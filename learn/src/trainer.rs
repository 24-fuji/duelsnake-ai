//! 自己対戦 Double DQN。複数の対戦を同時に進め、両プレイヤーを同じネットワークで動かして両方の経験から学習する

use crate::config::Config;
use crate::env::game::{Action, EndReason, GameEnv, GameResult, NUM_PLAYERS};
use crate::env::observation::{self, GRID_CHANNELS, VECTOR_FEATURES};
use crate::env::rules::Rules;
use crate::export::{self, ModelFile, TrainingInfo};
use crate::force::ForceField;
use crate::model::{NetworkSpec, QNetwork};
use crate::monitor::{approx_count, hms, thousands, StatsRow, StatusLine, TrainLog};
use crate::replay::{NStepBuilder, Obs, ReplayBuffer};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tch::nn::OptimizerConfig;
use tch::{nn, Device, Kind, Reduction, Tensor};

pub struct OutputPaths {
    pub recent_model: PathBuf,
    pub backup_dir: PathBuf,
}

/// 1プレイヤー分の判断の流れ
struct Stream {
    /// 直前の判断 (観測と行動)。次の判断か試合終了で遷移として確定する
    last: Option<(Obs, i64)>,
    /// 直前の判断以降に得た報酬
    reward: f32,
    /// この試合で引力・斥力から得た報酬の合計
    force_reward: f32,
    nstep: NStepBuilder,
}

struct Slot {
    env: GameEnv,
    streams: [Stream; NUM_PLAYERS],
}

impl Slot {
    fn new(rules: Rules, seed: u64, n_step: usize, gamma: f32) -> Self {
        let stream = || Stream {
            last: None,
            reward: 0.0,
            force_reward: 0.0,
            nstep: NStepBuilder::new(n_step, gamma),
        };
        Self {
            env: GameEnv::new(rules, seed),
            streams: [stream(), stream()],
        }
    }
}

/// ログ出力の間隔ごとに集計する値
struct WindowStats {
    started: Instant,
    games: u64,
    deaths: u64,
    draws: u64,
    score_sum: i64,
    length_sum: usize,
    force_reward_sum: f64,
    ticks_sum: u64,
    loss_sum: f64,
    q_sum: f64,
    loss_count: u64,
    /// ランダムでない行動の回数 (Action::ALL の順)
    action_counts: [u64; Action::COUNT],
}

impl WindowStats {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            games: 0,
            deaths: 0,
            draws: 0,
            score_sum: 0,
            length_sum: 0,
            force_reward_sum: 0.0,
            ticks_sum: 0,
            loss_sum: 0.0,
            q_sum: 0.0,
            loss_count: 0,
            action_counts: [0; Action::COUNT],
        }
    }

    fn record_game(&mut self, slot: &Slot, result: GameResult) {
        let env = &slot.env;
        self.games += 1;
        self.deaths += (result.reason == EndReason::Death) as u64;
        self.draws += result.winner.is_none() as u64;
        for (field, stream) in env.fields.iter().zip(&slot.streams) {
            self.score_sum += field.snake.score as i64;
            self.length_sum += field.snake.len();
            self.force_reward_sum += stream.force_reward as f64;
        }
        self.ticks_sum += env.tick as u64;
    }
}

pub struct Trainer {
    config: Config,
    rules: Rules,
    force: ForceField,
    spec: NetworkSpec,
    device: Device,
    online_vs: nn::VarStore,
    online: QNetwork,
    target_vs: nn::VarStore,
    target: QNetwork,
    optimizer: nn::Optimizer,
    replay: ReplayBuffer,
    rng: StdRng,
    slots: Vec<Slot>,
    info: TrainingInfo,
    /// 前回の更新以降にリプレイへ入った遷移数
    since_update: usize,
    grid_buf: Vec<f32>,
    vector_buf: Vec<f32>,
}

impl Trainer {
    pub fn new(
        config: Config,
        device: Device,
        seed: u64,
        resume: Option<ModelFile>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        tch::manual_seed(seed as i64);
        let mut rng = StdRng::seed_from_u64(seed);
        let rules = Rules::from_config(&config.game);
        let spec = NetworkSpec::new(&config.model, &rules);
        let train = &config.train;

        let online_vs = nn::VarStore::new(device);
        let mut online = QNetwork::new(&online_vs.root(), &spec);
        let mut target_vs = nn::VarStore::new(device);
        let target = QNetwork::new(&target_vs.root(), &spec);

        let mut info = TrainingInfo::default();
        if let Some(model) = resume {
            model.load_into(&mut online, &spec)?;
            info = model.training;
        }
        target_vs.copy(&online_vs)?;
        let optimizer = nn::Adam::default().build(&online_vs, train.learning_rate)?;

        let grid_shape = [GRID_CHANNELS as i64, spec.height, spec.width];
        let replay = ReplayBuffer::new(train.replay_capacity, grid_shape, VECTOR_FEATURES);
        let gamma = train.gamma as f32;
        let slots = (0..train.num_envs)
            .map(|_| Slot::new(rules, rng.gen(), train.n_step, gamma))
            .collect();

        Ok(Self {
            force: ForceField::new(&config.reward.force),
            config,
            rules,
            spec,
            device,
            online_vs,
            online,
            target_vs,
            target,
            optimizer,
            replay,
            rng,
            slots,
            info,
            since_update: 0,
            grid_buf: Vec::new(),
            vector_buf: Vec::new(),
        })
    }

    pub fn spec(&self) -> &NetworkSpec {
        &self.spec
    }

    pub fn info(&self) -> TrainingInfo {
        self.info
    }

    fn grid_len(&self) -> usize {
        GRID_CHANNELS * self.rules.cell_count()
    }

    fn epsilon(&self) -> f64 {
        let t = &self.config.train;
        let progress =
            (self.info.decisions as f64 / t.epsilon_decay_decisions.max(1) as f64).min(1.0);
        t.epsilon_start + (t.epsilon_end - t.epsilon_start) * progress
    }

    pub fn run(
        &mut self,
        running: &AtomicBool,
        max_games: Option<u64>,
        paths: &OutputPaths,
        log: &mut TrainLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let started = Instant::now();
        let session_start = self.info.games;
        let save_interval = self.config.train.save_interval_games;
        let log_interval = Duration::from_secs(self.config.train.log_interval_seconds);
        let mut stats = WindowStats::new();
        let mut status = StatusLine::new();

        while running.load(Ordering::SeqCst)
            && max_games.is_none_or(|m| self.info.games - session_start < m)
        {
            let games_before = self.info.games;
            self.tick(&mut stats);
            self.train_if_due(&mut stats);

            if stats.games > 0 && stats.started.elapsed() >= log_interval {
                log.stats(&self.stats_row(&stats))?;
                stats = WindowStats::new();
            }
            if self.info.games / save_interval > games_before / save_interval {
                self.save(&paths.recent_model)?;
                let message = format!(
                    "途中経過を保存しました ({} 試合): {}",
                    thousands(self.info.games),
                    paths.recent_model.display()
                );
                status.message(&message);
                log.event(&message)?;
            }
            if status.due() {
                status.show(self.status_text(started, session_start));
            }
        }

        if !running.load(Ordering::SeqCst) {
            let message =
                "中断しました。モデルを保存しています... (もう一度 Ctrl+C で保存せずに終了)";
            status.message(message);
            log.event(message)?;
        }
        if stats.games > 0 {
            log.stats(&self.stats_row(&stats))?;
        }
        status.clear();

        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup = paths.backup_dir.join(format!(
            "snake-model-{}-{timestamp}.json",
            export::size_label(&self.config.game.grid)
        ));
        self.save(&backup)?;
        self.save(&paths.recent_model)?;
        let messages = [
            format!(
                "この実行で {} 試合 (合計 {} 試合) を {} で学習しました",
                thousands(self.info.games - session_start),
                thousands(self.info.games),
                hms(started.elapsed())
            ),
            format!("バックアップを保存しました: {}", backup.display()),
            format!("最新モデルを更新しました: {}", paths.recent_model.display()),
        ];
        for message in &messages {
            println!("{message}");
            log.event(message)?;
        }
        Ok(())
    }

    /// 最下行に出す状況。試合数は同時対戦の都合で端数が意味を持たないので概数にする
    fn status_text(&self, started: Instant, session_start: u64) -> String {
        let elapsed = started.elapsed();
        let phase = if self.replay.len() < self.config.train.learning_starts {
            "経験を収集中"
        } else {
            "学習中"
        };
        let games_per_sec =
            (self.info.games - session_start) as f64 / elapsed.as_secs_f64().max(1e-9);
        format!(
            "{phase} | 約 {} 試合 | ε {:.2} | {:.1} 試合/秒 | 経過 {}",
            approx_count(self.info.games),
            self.epsilon(),
            games_per_sec,
            hms(elapsed)
        )
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        ModelFile::new(&self.config, &self.online, &self.spec, self.info).write(path)
    }

    /// 全対戦を1ティック進める。移動直前のプレイヤーだけがまとめて推論して行動を決める
    fn tick(&mut self, stats: &mut WindowStats) {
        let (g, v) = (self.grid_len(), VECTOR_FEATURES);
        let requests: Vec<(usize, usize)> = self
            .slots
            .iter()
            .enumerate()
            .flat_map(|(s, slot)| {
                (0..NUM_PLAYERS)
                    .filter(|&p| slot.env.needs_decision(p))
                    .map(move |p| (s, p))
            })
            .collect();

        let n = requests.len();
        self.grid_buf.resize(n * g, 0.0);
        self.vector_buf.resize(n * v, 0.0);
        for (k, &(s, p)) in requests.iter().enumerate() {
            observation::encode(
                &self.slots[s].env,
                p,
                &mut self.grid_buf[k * g..(k + 1) * g],
                &mut self.vector_buf[k * v..(k + 1) * v],
            );
        }
        let actions = self.select_actions(n, stats);
        self.info.decisions += n as u64;

        let mut env_actions = vec![[None; NUM_PLAYERS]; self.slots.len()];
        for (k, &(s, p)) in requests.iter().enumerate() {
            let obs = Obs::quantize(
                &self.grid_buf[k * g..(k + 1) * g],
                &self.vector_buf[k * v..(k + 1) * v],
            );
            let stream = &mut self.slots[s].streams[p];
            if let Some((prev_obs, prev_action)) = stream.last.take() {
                self.since_update += stream.nstep.push(
                    prev_obs,
                    prev_action,
                    stream.reward,
                    Some(&obs),
                    &mut self.replay,
                );
            }
            stream.reward = 0.0;
            stream.last = Some((obs, actions[k]));
            env_actions[s][p] = Some(Action::from_index(actions[k] as usize));
        }

        let reward_cfg = &self.config.reward;
        let (n_step, gamma) = (self.config.train.n_step, self.config.train.gamma as f32);
        for (slot, actions) in self.slots.iter_mut().zip(env_actions) {
            // 引力・斥力の仕事は、動く前の頭・アイテム・お邪魔ブロックの位置から求める
            let before = self.force.is_active().then(|| {
                slot.env
                    .fields
                    .each_ref()
                    .map(|f| (f.snake.head(), f.items.clone(), f.obstacles.clone()))
            });
            slot.env.step(actions);
            for (p, stream) in slot.streams.iter_mut().enumerate() {
                let field = &slot.env.fields[p];
                if let Some(kind) = field.eaten {
                    stream.reward += reward_cfg.pickup.reward(kind);
                }
                if let Some(before) = &before {
                    let (head, items, obstacles) = &before[p];
                    let work = self.force.work(items, obstacles, *head, field.snake.head());
                    stream.reward += work;
                    stream.force_reward += work;
                }
            }

            let Some(result) = slot.env.result else {
                continue;
            };
            for (p, stream) in slot.streams.iter_mut().enumerate() {
                stream.reward += match result.winner {
                    Some(w) if w == p => reward_cfg.win,
                    Some(_) => reward_cfg.lose,
                    None => reward_cfg.draw,
                };
                if let Some((obs, action)) = stream.last.take() {
                    self.since_update +=
                        stream
                            .nstep
                            .push(obs, action, stream.reward, None, &mut self.replay);
                }
            }
            stats.record_game(slot, result);
            self.info.games += 1;
            *slot = Slot::new(self.rules, self.rng.gen(), n_step, gamma);
        }
    }

    /// ε-greedy。ランダムに決まらなかった分だけまとめて推論する
    fn select_actions(&mut self, n: usize, stats: &mut WindowStats) -> Vec<i64> {
        let (g, v) = (self.grid_len(), VECTOR_FEATURES);
        let epsilon = self.epsilon();
        let mut actions = vec![0i64; n];
        let mut greedy = Vec::new();
        for (k, action) in actions.iter_mut().enumerate() {
            if self.rng.gen::<f64>() < epsilon {
                *action = self.rng.gen_range(0..Action::COUNT as i64);
            } else {
                greedy.push(k);
            }
        }
        if greedy.is_empty() {
            return actions;
        }

        let mut grids = Vec::with_capacity(greedy.len() * g);
        let mut vectors = Vec::with_capacity(greedy.len() * v);
        for &k in &greedy {
            grids.extend_from_slice(&self.grid_buf[k * g..(k + 1) * g]);
            vectors.extend_from_slice(&self.vector_buf[k * v..(k + 1) * v]);
        }
        let m = greedy.len() as i64;
        let grid = Tensor::from_slice(&grids)
            .view([m, GRID_CHANNELS as i64, self.spec.height, self.spec.width])
            .to(self.device);
        let vector = Tensor::from_slice(&vectors)
            .view([m, v as i64])
            .to(self.device);
        let best = tch::no_grad(|| self.online.forward(&grid, &vector).argmax(1, false));
        let best = Vec::<i64>::try_from(best.to(Device::Cpu)).expect("行動の取り出しに失敗");
        for (&k, a) in greedy.iter().zip(best) {
            actions[k] = a;
            stats.action_counts[a as usize] += 1;
        }
        actions
    }

    fn train_if_due(&mut self, stats: &mut WindowStats) {
        let per_update = self.config.train.transitions_per_update;
        if self.replay.len() < self.config.train.learning_starts {
            self.since_update = 0;
            return;
        }
        while self.since_update >= per_update {
            self.since_update -= per_update;
            let (loss, q_mean) = self.train_step();
            stats.loss_sum += loss;
            stats.q_sum += q_mean;
            stats.loss_count += 1;
        }
    }

    /// 1回更新し、(損失, 学習バッチでの max Q の平均) を返す
    fn train_step(&mut self) -> (f64, f64) {
        let train = &self.config.train;
        let batch = self
            .replay
            .sample(train.batch_size, &mut self.rng, self.device);
        let bootstrap_discount = train.gamma.powi(train.n_step as i32);

        let q_all = self.online.forward(&batch.grid, &batch.vector);
        let q_mean = tch::no_grad(|| {
            q_all
                .max_dim(1, false)
                .0
                .mean(Kind::Float)
                .double_value(&[])
        });
        let q = q_all.gather(1, &batch.actions, false).squeeze_dim(1);
        let target = tch::no_grad(|| {
            // Double DQN: 次の行動はオンライン側で選び、その価値はターゲット側で見積もる
            let next_action = self
                .online
                .forward(&batch.next_grid, &batch.next_vector)
                .argmax(1, true);
            let next_q = self
                .target
                .forward(&batch.next_grid, &batch.next_vector)
                .gather(1, &next_action, false)
                .squeeze_dim(1);
            &batch.returns + next_q * (1.0 - &batch.dones) * bootstrap_discount
        });
        let loss = q.smooth_l1_loss(&target, Reduction::Mean, 1.0);
        self.optimizer
            .backward_step_clip_norm(&loss, train.grad_clip_norm);

        self.info.updates += 1;
        if self
            .info
            .updates
            .is_multiple_of(train.target_update_interval)
        {
            self.target_vs
                .copy(&self.online_vs)
                .expect("ターゲットネットワークの同期に失敗");
        }
        (loss.double_value(&[]), q_mean)
    }

    fn stats_row(&self, stats: &WindowStats) -> StatsRow {
        let games = stats.games.max(1) as f64;
        let snakes = games * NUM_PLAYERS as f64;
        let (loss, q_mean) = if stats.loss_count > 0 {
            let n = stats.loss_count as f64;
            (stats.loss_sum / n, stats.q_sum / n)
        } else {
            (f64::NAN, f64::NAN)
        };
        let greedy = stats.action_counts.iter().sum::<u64>().max(1) as f64;
        StatsRow {
            games: self.info.games,
            window_games: stats.games,
            decisions: self.info.decisions,
            updates: self.info.updates,
            epsilon: self.epsilon(),
            loss,
            q_mean,
            avg_score: stats.score_sum as f64 / snakes,
            avg_length: stats.length_sum as f64 / snakes,
            avg_force_reward: stats.force_reward_sum / snakes,
            death_rate: stats.deaths as f64 / games,
            draw_rate: stats.draws as f64 / games,
            avg_ticks: stats.ticks_sum as f64 / games,
            games_per_sec: stats.games as f64 / stats.started.elapsed().as_secs_f64(),
            action_rates: stats.action_counts.map(|c| c as f64 / greedy),
        }
    }
}
