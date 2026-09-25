use super::field::ItemType;
use crate::config::GameConfig;

/// アイテムを食べたときのスコアと長さの増減
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppleEffect {
    pub score: i32,
    pub grow: i32,
}

/// ゲーム中に参照するルール値。秒数はすべてティック数に変換済み
#[derive(Debug, Clone, Copy)]
pub struct Rules {
    pub width: i32,
    pub height: i32,
    pub time_limit_ticks: u32,
    pub initial_length: usize,
    pub normal_interval_ticks: u32,
    pub boost_interval_ticks: u32,
    pub boost_duration_ticks: u32,
    pub boost_cooldown_ticks: u32,
    pub apple_respawn_ticks: u32,
    pub special_respawn_ticks: u32,
    pub jam_delay_ticks: u32,
    pub jam_safe_distance: i32,
    pub normal_apples: usize,
    pub poison_apples: usize,
    pub normal_apple: AppleEffect,
    pub gold_apple: AppleEffect,
    pub poison_apple: AppleEffect,
}

impl Rules {
    pub fn from_config(cfg: &GameConfig) -> Self {
        let tick = cfg.grid.tick_seconds;
        // 0.6 / 0.1 = 5.999... のような誤差で1ティックずれないよう四捨五入する
        let ticks = |sec: f32| (sec / tick).round().max(0.0) as u32;
        let items = &cfg.items;

        Self {
            width: cfg.grid.width,
            height: cfg.grid.height,
            time_limit_ticks: ticks(cfg.grid.time_limit_seconds).max(1),
            initial_length: cfg.snake.initial_length,
            normal_interval_ticks: ticks(cfg.snake.normal_speed_interval_sec).max(1),
            boost_interval_ticks: ticks(cfg.snake.boost_speed_interval_sec).max(1),
            boost_duration_ticks: ticks(cfg.snake.boost_duration_sec).max(1),
            boost_cooldown_ticks: ticks(cfg.snake.boost_cooldown_sec),
            apple_respawn_ticks: ticks(items.apple_respawn_delay_sec),
            special_respawn_ticks: ticks(items.special_respawn_delay_sec),
            jam_delay_ticks: ticks(items.jam_block_delay_sec),
            jam_safe_distance: items.jam_block_safe_distance,
            normal_apples: items.max_apples,
            poison_apples: items.max_poison,
            normal_apple: AppleEffect {
                score: items.normal_apple_score,
                grow: items.normal_apple_grow,
            },
            gold_apple: AppleEffect {
                score: items.gold_apple_score,
                grow: items.gold_apple_grow,
            },
            poison_apple: AppleEffect {
                score: items.poison_apple_score,
                grow: items.poison_apple_grow,
            },
        }
    }

    pub fn respawn_ticks(&self, item: ItemType) -> u32 {
        match item {
            ItemType::NormalApple | ItemType::PoisonApple => self.apple_respawn_ticks,
            ItemType::GoldApple | ItemType::BlockClear | ItemType::BlockJam => {
                self.special_respawn_ticks
            }
        }
    }

    pub fn cell_count(&self) -> usize {
        (self.width * self.height) as usize
    }
}
