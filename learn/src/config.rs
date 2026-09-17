use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct GameConfig {
    pub grid: GridConfig,
    pub snake: SnakeConfig,
    pub items: ItemConfig,
    pub agent: AgentConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GridConfig {
    pub width: i32,
    pub height: i32,
    pub time_limit_seconds: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SnakeConfig {
    pub initial_length: usize,
    pub normal_speed_interval_sec: f32,
    pub boost_speed_interval_sec: f32,
    pub boost_duration_sec: f32,
    pub boost_cooldown_sec: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ItemConfig {
    pub apple_respawn_delay_sec: f32,
    pub special_respawn_delay_sec: f32,
    pub jam_block_delay_sec: f32,
    pub max_apples: usize,
    pub max_poison: usize,
    pub normal_apple_score: i32,
    pub gold_apple_score: i32,
    pub poison_apple_score: i32,
    pub normal_apple_grow: i32,
    pub gold_apple_grow: i32,
    pub poison_apple_grow: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    pub learning_rate: f64,
    pub epsilon: f64,
    pub num_actions: i64,
    pub input_channels: i64,
    pub random_action_bound: i64,
}

impl GameConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: GameConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    // 秒数を0.1s単位のステップ数に変換するヘルパー関数
    pub fn sec_to_steps(sec: f32) -> u16 {
        (sec / 0.1) as u16
    }
}