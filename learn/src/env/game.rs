use super::snake::*;

// フィールドサイズを 16x16 に固定
pub const GRID_SIZE: i32 = 16;

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
    pub player_snake: Snake,
    pub ai_snake: Snake,
    pub time_remaining: f32, // 30秒
}

impl GameEnv {
    pub fn new() -> Self {
        Self {
            // 16x16の領域内に初期配置（プレイヤー: 左上付近, AI: 右下付近）
            player_snake: Snake::new(2, 2),
            ai_snake: Snake::new(GRID_SIZE - 3, GRID_SIZE - 3),
            time_remaining: 30.0,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.time_remaining -= dt;
        // ゲーム進行・壁衝突判定 (0 <= x < GRID_SIZE, 0 <= y < GRID_SIZE)
    }
}