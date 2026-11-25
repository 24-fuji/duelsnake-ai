//! 2つの学習済みモデルを対戦させて強さを比べる。学習前に取っておいた旧モデルと、学習後の新モデルの比較に使う
//!
//! どちらも ε = 0 (常に Q 値が最大の行動) で動かす。同じシードの盤面で先手 (プレイヤー 0) と後手を
//! 入れ替えて 2 試合ずつ行い、盤面の運とプレイヤー番号の偏りを打ち消す

use crate::config::{ConvLayerConfig, GameConfig, ModelConfig};
use crate::env::game::{Action, EndReason, GameEnv, GameResult, NUM_PLAYERS};
use crate::env::observation::{self, GRID_CHANNELS, VECTOR_FEATURES};
use crate::env::rules::Rules;
use crate::export::{self, ModelFile, TrainingInfo};
use crate::model::{NetworkSpec, QNetwork};
use crate::monitor::{hms, thousands, StatusLine};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tch::{nn, Device, Tensor};

/// 対戦させるモデルの数 (旧モデルと新モデル)
pub const CONTENDERS: usize = 2;
pub const OLD: usize = 0;
pub const NEW: usize = 1;
const LABELS: [&str; CONTENDERS] = ["旧モデル", "新モデル"];

/// 対戦させる1モデル
pub struct Contender {
    pub path: PathBuf,
    pub training: TrainingInfo,
    pub created_at: String,
    /// モデルを学習したときのゲームルール。対戦は config.yaml のルールで行う
    pub game: GameConfig,
    spec: NetworkSpec,
    net: QNetwork,
    _vs: nn::VarStore,
}

impl Contender {
    /// モデル JSON を読む。層の構成は config.yaml ではなくモデル自身から決めるので、構成を変える前のモデルとも比べられる
    pub fn load(
        path: &Path,
        rules: &Rules,
        device: Device,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let model = ModelFile::read(path)?;
        let size = export::size_label(&model.game.grid);
        if (model.game.grid.width, model.game.grid.height) != (rules.width, rules.height) {
            return Err(format!(
                "モデル {} の盤面 {size} が config.yaml の盤面 {}x{} と異なります",
                path.display(),
                rules.width,
                rules.height
            )
            .into());
        }
        let Some((_, hidden)) = model.network.dense.split_last() else {
            return Err(format!("モデル {} に全結合層がありません", path.display()).into());
        };
        let Some((grid_channels, vector_features)) = model.input_sizes() else {
            return Err(format!(
                "モデル {} の観測の仕様がこのコードの観測と合わないので、対戦させられません",
                path.display()
            )
            .into());
        };
        let layers = ModelConfig {
            conv_layers: model
                .network
                .conv
                .iter()
                .map(|c| ConvLayerConfig {
                    channels: c.out_channels,
                    stride: c.stride,
                })
                .collect(),
            hidden_layers: hidden.iter().map(|d| d.out_features).collect(),
        };
        let spec = NetworkSpec::with_inputs(&layers, rules, grid_channels, vector_features);
        let vs = nn::VarStore::new(device);
        let mut net = QNetwork::new(&vs.root(), &spec);
        model
            .load_into(&mut net, &spec)
            .map_err(|e| format!("モデル {} を読み込めません: {e}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
            training: model.training,
            created_at: model.created_at,
            game: model.game,
            spec,
            net,
            _vs: vs,
        })
    }

    /// 1局面あたりの入力の長さ (盤面, 状態ベクトル)。観測の先頭からこの長さだけを使う
    fn input_lens(&self) -> (usize, usize) {
        let s = &self.spec;
        (
            (s.grid_channels * s.height * s.width) as usize,
            s.vector_features as usize,
        )
    }

    /// 入力の説明 (例: 盤面 17 チャネル / 状態 22 個)
    pub fn input_label(&self) -> String {
        let s = &self.spec;
        let note = if (s.grid_channels as usize, s.vector_features as usize)
            == (GRID_CHANNELS, VECTOR_FEATURES)
        {
            ""
        } else {
            "、観測の先頭部分だけを使う古いモデル"
        };
        format!(
            "入力: 盤面 {} チャネル / 状態 {} 個{note}",
            s.grid_channels, s.vector_features
        )
    }

    /// 観測をまとめて推論し、Q 値が最大の行動を返す
    fn best_actions(&self, grids: &[f32], vectors: &[f32], device: Device) -> Vec<usize> {
        let s = &self.spec;
        let n = (vectors.len() as i64) / s.vector_features;
        let grid = Tensor::from_slice(grids)
            .view([n, s.grid_channels, s.height, s.width])
            .to(device);
        let vector = Tensor::from_slice(vectors)
            .view([n, s.vector_features])
            .to(device);
        let best = tch::no_grad(|| self.net.forward(&grid, &vector).argmax(1, false));
        Vec::<i64>::try_from(best.to(Device::Cpu))
            .expect("行動の取り出しに失敗")
            .into_iter()
            .map(|a| a as usize)
            .collect()
    }
}

/// 対戦中の1試合
struct Match {
    env: GameEnv,
    /// seats[p] はプレイヤー p を動かすモデル (OLD か NEW)
    seats: [usize; NUM_PLAYERS],
}

/// 対戦結果の集計。添字はモデル (OLD / NEW)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Tally {
    pub games: u64,
    pub wins: [u64; CONTENDERS],
    pub draws: u64,
    /// 勝ったときの決着の仕方 [盤面を埋めた, 相手が衝突した, 時間切れ]
    pub win_reasons: [[u64; 3]; CONTENDERS],
    pub deaths: [u64; CONTENDERS],
    pub score_sum: [i64; CONTENDERS],
    pub length_sum: [u64; CONTENDERS],
    pub ticks_sum: u64,
}

fn reason_index(reason: EndReason) -> usize {
    match reason {
        EndReason::Filled => 0,
        EndReason::Death => 1,
        EndReason::TimeUp => 2,
    }
}

impl Tally {
    fn record(&mut self, game: &Match, result: GameResult) {
        self.games += 1;
        self.ticks_sum += game.env.tick as u64;
        match result.winner {
            Some(p) => {
                let c = game.seats[p];
                self.wins[c] += 1;
                self.win_reasons[c][reason_index(result.reason)] += 1;
            }
            None => self.draws += 1,
        }
        for (field, &c) in game.env.fields.iter().zip(&game.seats) {
            self.deaths[c] += (!field.snake.alive) as u64;
            self.score_sum[c] += field.snake.score as i64;
            self.length_sum[c] += field.snake.len() as u64;
        }
    }

    /// 新モデルの勝ち点の割合 (引き分けは 0.5 勝) と、その 95% 信頼区間の半幅
    pub fn new_win_rate(&self) -> (f64, f64) {
        if self.games == 0 {
            return (f64::NAN, f64::NAN);
        }
        let n = self.games as f64;
        let (win, draw) = (self.wins[NEW] as f64, self.draws as f64);
        let mean = (win + 0.5 * draw) / n;
        // 1試合の勝ち点 (1, 0.5, 0) の分散から標準誤差を求める
        let variance = (win + 0.25 * draw) / n - mean * mean;
        (mean, 1.96 * (variance.max(0.0) / n).sqrt())
    }
}

/// 試合を組む順番。偶数番目の試合で新しいシードを引き、奇数番目は同じシードで席を入れ替える
struct Schedule {
    rng: StdRng,
    rules: Rules,
    games: u64,
    started: u64,
    pair_seed: u64,
}

impl Schedule {
    fn next(&mut self) -> Option<Match> {
        if self.started >= self.games {
            return None;
        }
        if self.started.is_multiple_of(2) {
            self.pair_seed = self.rng.gen();
        }
        let mut seats = [OLD; NUM_PLAYERS];
        seats[(self.started % 2) as usize] = NEW;
        self.started += 1;
        Some(Match {
            env: GameEnv::new(self.rules, self.pair_seed),
            seats,
        })
    }
}

/// 旧モデル (contenders[OLD]) と新モデル (contenders[NEW]) を `games` 試合対戦させる。
/// `envs` 試合ずつ同時に進めて推論をまとめる
pub fn play(
    contenders: &[Contender; CONTENDERS],
    rules: Rules,
    games: u64,
    envs: usize,
    seed: u64,
    device: Device,
) -> Tally {
    let mut schedule = Schedule {
        rng: StdRng::seed_from_u64(seed),
        rules,
        games,
        started: 0,
        pair_seed: 0,
    };
    let mut matches: Vec<Match> = (0..envs.max(1)).map_while(|_| schedule.next()).collect();
    let mut tally = Tally::default();
    let mut full_grid = vec![0.0; GRID_CHANNELS * rules.cell_count()];
    let mut full_vector = vec![0.0; VECTOR_FEATURES];
    let (mut grids, mut vectors) = (Vec::new(), Vec::new());
    let mut status = StatusLine::new();
    let clock = Instant::now();

    while !matches.is_empty() {
        let mut actions = vec![[None; NUM_PLAYERS]; matches.len()];
        for (c, contender) in contenders.iter().enumerate() {
            let requests: Vec<(usize, usize)> = matches
                .iter()
                .enumerate()
                .flat_map(|(m, game)| {
                    (0..NUM_PLAYERS)
                        .filter(move |&p| game.seats[p] == c && game.env.needs_decision(p))
                        .map(move |p| (m, p))
                })
                .collect();
            if requests.is_empty() {
                continue;
            }
            // 観測をすべて書いてから、このモデルが使う先頭部分だけを集める
            let (cg, cv) = contender.input_lens();
            grids.clear();
            vectors.clear();
            for &(m, p) in &requests {
                observation::encode(&matches[m].env, p, &mut full_grid, &mut full_vector);
                grids.extend_from_slice(&full_grid[..cg]);
                vectors.extend_from_slice(&full_vector[..cv]);
            }
            let best = contender.best_actions(&grids, &vectors, device);
            for (&(m, p), a) in requests.iter().zip(best) {
                actions[m][p] = Some(Action::from_index(a));
            }
        }
        for (game, action) in matches.iter_mut().zip(actions) {
            game.env.step(action);
        }

        // 終わった試合を集計し、次の試合に差し替える。もう組む試合が無ければ取り除く
        let mut m = 0;
        while m < matches.len() {
            let Some(result) = matches[m].env.result else {
                m += 1;
                continue;
            };
            tally.record(&matches[m], result);
            match schedule.next() {
                Some(game) => {
                    matches[m] = game;
                    m += 1;
                }
                None => {
                    matches.swap_remove(m);
                }
            }
        }

        if status.due() {
            status.show(format!(
                "対戦中 | {} / {} 試合 | 経過 {}",
                thousands(tally.games),
                thousands(games),
                hms(clock.elapsed())
            ));
        }
    }
    status.clear();
    tally
}

/// 対戦結果を人が読む形の行にする
pub fn report(contenders: &[Contender; CONTENDERS], tally: &Tally) -> Vec<String> {
    let n = tally.games.max(1) as f64;
    let pct = |count: u64| format!("{:5.1}%", count as f64 / n * 100.0);
    let mut lines = Vec::new();
    for (label, c) in LABELS.iter().zip(contenders) {
        lines.push(format!(
            "{label}: {} (通算 {} 試合学習 / 書き出し {} / {})",
            c.path.display(),
            thousands(c.training.games),
            c.created_at,
            c.input_label()
        ));
    }
    lines.push(format!(
        "{} 試合 (同じ盤面で先手と後手を入れ替えて 2 試合ずつ)",
        thousands(tally.games)
    ));
    for c in [NEW, OLD] {
        let [filled, death, time_up] = tally.win_reasons[c];
        lines.push(format!(
            "  {}の勝ち: {:>6} ({}) 内訳: 盤面を埋めた {} / 相手が衝突 {} / 時間切れでスコア勝ち {}",
            LABELS[c],
            thousands(tally.wins[c]),
            pct(tally.wins[c]),
            thousands(filled),
            thousands(death),
            thousands(time_up)
        ));
    }
    lines.push(format!(
        "  引き分け      : {:>6} ({})",
        thousands(tally.draws),
        pct(tally.draws)
    ));
    let (rate, margin) = tally.new_win_rate();
    lines.push(format!(
        "新モデルの勝率 (引き分けは 0.5 勝): {:.1}% ± {:.1}% (95% 信頼区間)",
        rate * 100.0,
        margin * 100.0
    ));
    let verdict = if rate - margin > 0.5 {
        "新モデルのほうが強い"
    } else if rate + margin < 0.5 {
        "旧モデルのほうが強い"
    } else {
        "はっきりした差は無い (試合数を増やすと差が見えることがある)"
    };
    lines.push(format!("判定: {verdict}"));
    for c in [NEW, OLD] {
        lines.push(format!(
            "  {}: 平均スコア {:.2} / 平均の長さ {:.1} / 衝突した試合 {}",
            LABELS[c],
            tally.score_sum[c] as f64 / n,
            tally.length_sum[c] as f64 / n,
            pct(tally.deaths[c])
        ));
    }
    lines.push(format!(
        "  平均試合時間: {:.1} ティック",
        tally.ticks_sum as f64 / n
    ));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;

    fn config() -> Config {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yaml");
        Config::load(&path).unwrap()
    }

    /// 乱数の重みのモデルを書き出し、Contender として読み直す
    fn contender(config: &Config, dir: &Path, name: &str) -> Contender {
        let rules = Rules::from_config(&config.game);
        let spec = NetworkSpec::new(&config.model, &rules);
        let vs = nn::VarStore::new(Device::Cpu);
        let net = QNetwork::new(&vs.root(), &spec);
        let path = dir.join(name);
        ModelFile::new(config, &net, &spec, TrainingInfo::default())
            .write(&path)
            .unwrap();
        Contender::load(&path, &rules, Device::Cpu).unwrap()
    }

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "duelsnake-arena-test-{}-{}",
            std::process::id(),
            chrono::Local::now().format("%H%M%S%f")
        ))
    }

    #[test]
    fn plays_all_games_with_swapped_seats() {
        tch::manual_seed(3);
        let mut config = config();
        let dir = temp_dir();
        let old = contender(&config, &dir, "old.json");
        // 層の構成が違うモデルとも対戦できる
        config.model.hidden_layers = vec![32, 16];
        let new = contender(&config, &dir, "new.json");
        let rules = Rules::from_config(&config.game);
        let contenders = [old, new];

        let tally = play(&contenders, rules, 7, 3, 1, Device::Cpu);
        assert_eq!(tally.games, 7);
        assert_eq!(tally.wins[OLD] + tally.wins[NEW] + tally.draws, 7);
        for c in [OLD, NEW] {
            assert_eq!(tally.win_reasons[c].iter().sum::<u64>(), tally.wins[c]);
        }
        // 同じシードなら同じ結果になる
        assert_eq!(play(&contenders, rules, 7, 3, 1, Device::Cpu), tally);
        let (rate, margin) = tally.new_win_rate();
        assert!((0.0..=1.0).contains(&rate) && margin >= 0.0);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn plays_old_model_that_sees_only_its_own_field() {
        tch::manual_seed(5);
        let config = config();
        let dir = temp_dir();
        let rules = Rules::from_config(&config.game);
        // 相手の盤面を足す前の観測 (盤面 9 チャネル / 状態 13 個) で学習したモデル
        let spec = NetworkSpec::with_inputs(&config.model, &rules, 9, 13);
        let vs = nn::VarStore::new(Device::Cpu);
        let net = QNetwork::new(&vs.root(), &spec);
        let path = dir.join("legacy.json");
        ModelFile::new(&config, &net, &spec, TrainingInfo::default())
            .write(&path)
            .unwrap();
        let old = Contender::load(&path, &rules, Device::Cpu).unwrap();
        assert!(old.input_label().contains("盤面 9 チャネル / 状態 13 個"));
        let new = contender(&config, &dir, "new.json");

        let tally = play(&[old, new], rules, 6, 4, 1, Device::Cpu);
        assert_eq!(tally.games, 6);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn same_model_on_both_sides_is_even() {
        tch::manual_seed(4);
        let config = config();
        let dir = temp_dir();
        let rules = Rules::from_config(&config.game);
        let a = contender(&config, &dir, "a.json");
        let b = Contender::load(&a.path, &rules, Device::Cpu).unwrap();

        // 同じモデル同士なら、同じ盤面で席を入れ替えた2試合は勝ち負けが入れ替わるだけになる
        let tally = play(&[a, b], rules, 20, 4, 2, Device::Cpu);
        assert_eq!(tally.wins[OLD], tally.wins[NEW]);
        assert_eq!(tally.score_sum[OLD], tally.score_sum[NEW]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn win_rate_counts_draws_as_half() {
        let tally = Tally {
            games: 4,
            wins: [1, 2],
            draws: 1,
            ..Tally::default()
        };
        let (rate, margin) = tally.new_win_rate();
        assert!((rate - 0.625).abs() < 1e-12);
        assert!(margin > 0.0);
    }

    #[test]
    fn rejects_model_of_another_grid_size() {
        let config = config();
        let dir = temp_dir();
        let a = contender(&config, &dir, "a.json");
        let mut other = config.game.clone();
        other.grid.width += 2;
        let err = Contender::load(&a.path, &Rules::from_config(&other), Device::Cpu)
            .err()
            .unwrap();
        assert!(err.to_string().contains("盤面"), "{err}");
        fs::remove_dir_all(dir).unwrap();
    }
}
