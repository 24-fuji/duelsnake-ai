use super::snake::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemType {
    NormalApple,  // +1
    GoldApple,    // +3
    PoisonApple,  // -2
    Block,        // ぶつかるとゲームオーバー
    BlockClear,   // お邪魔ブロック解除
    BlockJam,     // 相手に妨害ブロック配置
}

pub struct GameEnv {
    pub grid_size: i32,
    pub player_snake: Snake,
    pub ai_snake: Snake,
    pub time_remaining: f32, // 30秒
}

impl GameEnv {
    pub fn new(grid_size: i32) -> Self {
        Self {
            grid_size,
            player_snake: Snake::new(2, 2),
            ai_snake: Snake::new(grid_size - 3, grid_size - 3),
            time_remaining: 30.0,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.time_remaining -= dt;
        // ゲーム進行・アイテム判定・判定処理
    }
}