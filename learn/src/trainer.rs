//! 自己対戦 Double DQN。複数の対戦を同時に進め、両プレイヤーを同じネットワークで動かして両方の経験から学習する

use crate::config::Config;
use crate::env::game::{Action, EndReason, GameEnv, GameResult, NUM_PLAYERS};
use crate::env::observation::{self, GRID_CHANNELS, VECTOR_FEATURES};
use crate::env::rules::Rules;
use crate::export::{ModelFile, TrainingInfo};
use crate::model::{NetworkSpec, QNetwork};
use crate::replay::{NStepBuilder, Obs, ReplayBuffer};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tch::nn::OptimizerConfig;
use tch::{nn, Device, Reduction, Tensor};

pub struct OutputPaths {
    pub recent_model: PathBuf,
    pub backup_dir: PathBuf,
    pub log_file: PathBuf,
}

/// 1プレイヤー分の判断の流れ
struct Stream {
    /// 直前の判断 (観測と行動)。次の判断か試合終了で遷移として確定する
    last: Option<(Obs, i64)>,
    /// 直前の判断以降に得た報酬
    reward: f32,
    score: i32,
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
            score: 0,
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
    ticks_sum: u64,
    loss_sum: f64,
    loss_count: u64,
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
            ticks_sum: 0,
            loss_sum: 0.0,
            loss_count: 0,
        }
    }

    fn record_game(&mut self, env: &GameEnv, result: GameResult) {
        self.games += 1;
        self.deaths += (result.reason == EndReason::Death) as u64;
        self.draws += result.winner.is_none() as u64;
        for field in &env.fields {
            self.score_sum += field.snake.score as i64;
            self.length_sum += field.snake.len();
        }
        self.ticks_sum += env.tick as u64;
    }
}

pub struct Trainer {
    config: Config,
    rules: Rules,
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
            if model.game != config.game {
                println!("注意: 再開元モデルと config.yaml のゲームルールが異なります。現在の config.yaml のルールで学習を続けます");
            }
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = paths.log_file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut log = BufWriter::new(File::create(&paths.log_file)?);
        writeln!(
            log,
            "games,decisions,updates,epsilon,loss,avg_score,avg_length,death_rate,draw_rate,avg_ticks,games_per_sec"
        )?;

        let session_start = self.info.games;
        let save_interval = self.config.train.save_interval_games;
        let mut stats = WindowStats::new();

        while running.load(Ordering::SeqCst)
            && max_games.is_none_or(|m| self.info.games - session_start < m)
        {
            let games_before = self.info.games;
            self.tick(&mut stats);
            self.train_if_due(&mut stats);

            if stats.games >= self.config.train.log_interval_games {
                self.report(&stats, &mut log)?;
                stats = WindowStats::new();
            }
            if self.info.games / save_interval > games_before / save_interval {
                self.save(&paths.recent_model)?;
                println!(
                    "  -> 途中経過を保存しました: {}",
                    paths.recent_model.display()
                );
            }
        }
        if stats.games > 0 {
            self.report(&stats, &mut log)?;
        }
        log.flush()?;

        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup = paths
            .backup_dir
            .join(format!("snake-model-{timestamp}.json"));
        self.save(&backup)?;
        self.save(&paths.recent_model)?;
        println!("バックアップを保存しました: {}", backup.display());
        println!("最新モデルを更新しました: {}", paths.recent_model.display());
        Ok(())
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
        let actions = self.select_actions(n);
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
            slot.env.step(actions);
            for (p, stream) in slot.streams.iter_mut().enumerate() {
                let score = slot.env.fields[p].snake.score;
                stream.reward += (score - stream.score) as f32 * reward_cfg.score_point;
                stream.score = score;
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
            stats.record_game(&slot.env, result);
            self.info.games += 1;
            *slot = Slot::new(self.rules, self.rng.gen(), n_step, gamma);
        }
    }

    /// ε-greedy。ランダムに決まらなかった分だけまとめて推論する
    fn select_actions(&mut self, n: usize) -> Vec<i64> {
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
            stats.loss_sum += self.train_step();
            stats.loss_count += 1;
        }
    }

    fn train_step(&mut self) -> f64 {
        let train = &self.config.train;
        let batch = self
            .replay
            .sample(train.batch_size, &mut self.rng, self.device);
        let bootstrap_discount = train.gamma.powi(train.n_step as i32);

        let q = self
            .online
            .forward(&batch.grid, &batch.vector)
            .gather(1, &batch.actions, false)
            .squeeze_dim(1);
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
        loss.double_value(&[])
    }

    fn report(&self, stats: &WindowStats, log: &mut impl Write) -> std::io::Result<()> {
        let games = stats.games.max(1) as f64;
        let snakes = games * NUM_PLAYERS as f64;
        let loss = if stats.loss_count > 0 {
            stats.loss_sum / stats.loss_count as f64
        } else {
            f64::NAN
        };
        let avg_score = stats.score_sum as f64 / snakes;
        let avg_length = stats.length_sum as f64 / snakes;
        let death_rate = stats.deaths as f64 / games;
        let draw_rate = stats.draws as f64 / games;
        let avg_ticks = stats.ticks_sum as f64 / games;
        let games_per_sec = stats.games as f64 / stats.started.elapsed().as_secs_f64();
        let epsilon = self.epsilon();

        println!(
            "[{:>8} 試合] ε {:.3} | loss {:.4} | スコア {:.2} | 長さ {:.1} | 死亡決着 {:>5.1}% | 引き分け {:>4.1}% | {:.0} ティック | {:.1} 試合/秒",
            self.info.games,
            epsilon,
            loss,
            avg_score,
            avg_length,
            death_rate * 100.0,
            draw_rate * 100.0,
            avg_ticks,
            games_per_sec,
        );
        writeln!(
            log,
            "{},{},{},{:.4},{:.6},{:.3},{:.2},{:.4},{:.4},{:.1},{:.2}",
            self.info.games,
            self.info.decisions,
            self.info.updates,
            epsilon,
            loss,
            avg_score,
            avg_length,
            death_rate,
            draw_rate,
            avg_ticks,
            games_per_sec,
        )?;
        log.flush()
    }
}
