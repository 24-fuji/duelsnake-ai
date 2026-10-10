use crate::env::field::ItemType;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// config.yaml 全体
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub game: GameConfig,
    pub model: ModelConfig,
    pub reward: RewardConfig,
    pub train: TrainConfig,
    pub export: ExportConfig,
}

/// ゲームルール。Web 版と共通で、モデル JSON にもそのまま書き出す
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    pub grid: GridConfig,
    pub snake: SnakeConfig,
    pub items: ItemConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridConfig {
    pub width: i32,
    pub height: i32,
    pub tick_seconds: f32,
    pub time_limit_seconds: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnakeConfig {
    pub initial_length: usize,
    pub normal_speed_interval_sec: f32,
    pub boost_speed_interval_sec: f32,
    pub boost_duration_sec: f32,
    pub boost_cooldown_sec: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemConfig {
    pub apple_respawn_delay_sec: f32,
    pub special_respawn_delay_sec: f32,
    pub jam_block_delay_sec: f32,
    pub jam_block_safe_distance: i32,
    pub max_apples: usize,
    pub max_poison: usize,
    pub normal_apple_score: i32,
    pub gold_apple_score: i32,
    pub poison_apple_score: i32,
    pub normal_apple_grow: i32,
    pub gold_apple_grow: i32,
    pub poison_apple_grow: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    pub conv_layers: Vec<ConvLayerConfig>,
    pub hidden_layers: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConvLayerConfig {
    pub channels: i64,
    pub stride: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardConfig {
    pub win: f32,
    pub lose: f32,
    pub draw: f32,
    pub death_penalty: DeathPenaltyConfig,
    pub pickup: PickupRewardConfig,
    pub force: ForceConfig,
}

/// 衝突して死んだときの追加の罰。それまでにアイテムで稼いだ報酬に比例して重くする。
/// 勝ち負けの報酬 (lose) が一定だと、たくさん稼いだあとなら死んでも収支がプラスになり、盤面が埋まる終盤に無理をして死ぬ動きが出る
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeathPenaltyConfig {
    /// 死んだ試合でアイテムから得た報酬の合計のうち、この割合を罰として引く。1 なら稼ぎをすべて打ち消す
    pub score_ratio: f32,
    /// 割合を 0 からここまで上げていく判断回数。序盤は稼ぎが罰で消えて学習が進まなくなるのを避ける。0 なら最初から全力
    pub ramp_decisions: u64,
}

impl DeathPenaltyConfig {
    /// 死んだときに報酬へ足す値 (0 以下)。`earned` はその試合でアイテムから得た報酬の合計、`decisions` は通算の判断回数
    pub fn penalty(&self, earned: f32, decisions: u64) -> f32 {
        let ramp = if self.ramp_decisions == 0 {
            1.0
        } else {
            (decisions as f32 / self.ramp_decisions as f32).min(1.0)
        };
        -self.score_ratio * ramp * earned.max(0.0)
    }
}

/// アイテムを取ったときの報酬
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PickupRewardConfig {
    pub normal_apple: f32,
    pub gold_apple: f32,
    pub poison_apple: f32,
    pub block_clear: f32,
    pub block_jam: f32,
}

impl PickupRewardConfig {
    pub fn reward(&self, kind: ItemType) -> f32 {
        match kind {
            ItemType::NormalApple => self.normal_apple,
            ItemType::GoldApple => self.gold_apple,
            ItemType::PoisonApple => self.poison_apple,
            ItemType::BlockClear => self.block_clear,
            ItemType::BlockJam => self.block_jam,
        }
    }
}

/// 引力・斥力による報酬の補助。強さはすべて 0 以上で、0 ならその力は働かない
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForceConfig {
    /// 引力・斥力が届く最大のマンハッタン距離
    pub attraction_range: u32,
    pub repulsion_range: u32,
    pub normal_apple_attraction: f32,
    pub gold_apple_attraction: f32,
    pub poison_apple_repulsion: f32,
    pub jam_block_repulsion: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainConfig {
    pub num_envs: usize,
    pub learning_rate: f64,
    pub gamma: f64,
    pub n_step: usize,
    pub batch_size: usize,
    pub replay_capacity: usize,
    pub learning_starts: usize,
    pub transitions_per_update: usize,
    pub target_update_interval: u64,
    pub epsilon_start: f64,
    pub epsilon_end: f64,
    pub epsilon_decay_decisions: u64,
    pub grad_clip_norm: f64,
    pub log_interval_seconds: u64,
    pub save_interval_games: u64,
    pub commit_interval_games: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportConfig {
    pub difficulty: Vec<DifficultyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DifficultyConfig {
    pub name: String,
    pub random_action_rate: f32,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("設定ファイル {} を読めません: {e}", path.display()))?;
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| format!("設定ファイル {} の形式が不正です: {e}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        let grid = &self.game.grid;
        let snake = &self.game.snake;
        let train = &self.train;
        let check = |ok: bool, msg: &str| if ok { Ok(()) } else { Err(msg.to_string()) };

        check(
            grid.width >= 5 && grid.height >= 5,
            "grid の幅と高さは 5 以上にしてください",
        )?;
        check(
            grid.tick_seconds > 0.0,
            "tick_seconds は正の値にしてください",
        )?;
        check(
            snake.initial_length >= 1 && snake.initial_length as i32 <= grid.height / 2,
            "initial_length は 1 以上、盤面の高さの半分以下にしてください",
        )?;
        check(
            snake.boost_duration_sec >= grid.tick_seconds,
            "boost_duration_sec は 1 ティック以上にしてください",
        )?;
        check(
            !self.model.conv_layers.is_empty(),
            "conv_layers を 1 つ以上指定してください",
        )?;
        check(
            self.model
                .conv_layers
                .iter()
                .all(|c| c.channels > 0 && c.stride > 0)
                && self.model.hidden_layers.iter().all(|&h| h > 0),
            "層のチャネル数・ストライド・ユニット数は正の値にしてください",
        )?;
        let force = &self.reward.force;
        check(
            [
                force.normal_apple_attraction,
                force.gold_apple_attraction,
                force.poison_apple_repulsion,
                force.jam_block_repulsion,
            ]
            .iter()
            .all(|&s| s >= 0.0),
            "reward.force の強さは 0 以上にしてください",
        )?;
        check(
            self.reward.death_penalty.score_ratio >= 0.0,
            "reward.death_penalty.score_ratio は 0 以上にしてください",
        )?;
        check(train.num_envs >= 1, "num_envs は 1 以上にしてください")?;
        check(train.n_step >= 1, "n_step は 1 以上にしてください")?;
        check(
            train.batch_size <= train.learning_starts
                && train.learning_starts <= train.replay_capacity,
            "batch_size <= learning_starts <= replay_capacity にしてください",
        )?;
        check(
            train.transitions_per_update >= 1 && train.target_update_interval >= 1,
            "transitions_per_update と target_update_interval は 1 以上にしてください",
        )?;
        check(
            train.log_interval_seconds >= 1
                && train.save_interval_games >= 1
                && train.commit_interval_games >= 1,
            "log_interval_seconds・save_interval_games・commit_interval_games は 1 以上にしてください",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod death_penalty_tests {
    use super::DeathPenaltyConfig;

    fn cfg(score_ratio: f32, ramp_decisions: u64) -> DeathPenaltyConfig {
        DeathPenaltyConfig {
            score_ratio,
            ramp_decisions,
        }
    }

    #[test]
    fn penalty_grows_with_earnings_and_ramps_up() {
        let c = cfg(0.5, 100);
        assert_eq!(c.penalty(40.0, 0), 0.0);
        assert_eq!(c.penalty(40.0, 50), -10.0);
        assert_eq!(c.penalty(40.0, 100), -20.0);
        assert_eq!(c.penalty(40.0, 10_000), -20.0);
    }

    #[test]
    fn no_penalty_without_earnings_and_no_ramp_when_zero() {
        assert_eq!(cfg(1.0, 0).penalty(-8.0, 0), 0.0);
        assert_eq!(cfg(1.0, 0).penalty(8.0, 0), -8.0);
    }
}
