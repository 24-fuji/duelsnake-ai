//! AI の入力 (観測) の作り方。Web 側でも同じ手順で作る必要がある (RULES.md 参照)

use super::field::ItemType;
use super::game::GameEnv;
use super::snake::HeldItem;

/// 盤面チャネル。値は [チャネル][y][x] の順に並べる
pub const GRID_CHANNEL_NAMES: [&str; 9] = [
    "own_head",
    "own_body",
    "normal_apple",
    "gold_apple",
    "poison_apple",
    "block_clear_item",
    "block_jam_item",
    "obstacle",
    "in_bounds",
];
pub const GRID_CHANNELS: usize = GRID_CHANNEL_NAMES.len();

pub const VECTOR_FEATURE_NAMES: [&str; 13] = [
    "dir_up",
    "dir_down",
    "dir_left",
    "dir_right",
    "pending_growth",
    "held_block_clear",
    "held_block_jam",
    "boost_remaining",
    "boost_ready",
    "time_remaining",
    "score_diff",
    "opponent_holds_jam",
    "incoming_jams",
];
pub const VECTOR_FEATURES: usize = VECTOR_FEATURE_NAMES.len();

/// 状態ベクトルの正規化に使う定数。値 / 定数 を [0, 1] (score_diff は [-1, 1]) に切り詰める
pub const PENDING_GROWTH_SCALE: f32 = 5.0;
pub const SCORE_DIFF_SCALE: f32 = 10.0;
pub const INCOMING_JAMS_SCALE: f32 = 3.0;

fn item_channel(kind: ItemType) -> usize {
    match kind {
        ItemType::NormalApple => 2,
        ItemType::GoldApple => 3,
        ItemType::PoisonApple => 4,
        ItemType::BlockClear => 5,
        ItemType::BlockJam => 6,
    }
}

/// `player` から見た観測を書き込む。`grid` は GRID_CHANNELS * 高さ * 幅、`vector` は VECTOR_FEATURES の長さ
pub fn encode(env: &GameEnv, player: usize, grid: &mut [f32], vector: &mut [f32]) {
    let rules = env.rules();
    let (w, h) = (rules.width as usize, rules.height as usize);
    let plane = w * h;
    debug_assert_eq!(grid.len(), GRID_CHANNELS * plane);
    debug_assert_eq!(vector.len(), VECTOR_FEATURES);

    let field = &env.fields[player];
    let opponent = &env.fields[1 - player];
    let snake = &field.snake;
    let cell = |channel: usize, x: i32, y: i32| channel * plane + y as usize * w + x as usize;

    grid.fill(0.0);
    grid[8 * plane..9 * plane].fill(1.0);

    let head = snake.head();
    grid[cell(0, head.x, head.y)] = 1.0;
    // 胴体は首が 1.0、尻尾に向かって 1 / (長さ - 1) まで小さくなる
    let len = snake.len();
    for (i, p) in snake.body.iter().enumerate().skip(1) {
        grid[cell(1, p.x, p.y)] = (len - i) as f32 / (len - 1) as f32;
    }
    for item in &field.items {
        grid[cell(item_channel(item.kind), item.pos.x, item.pos.y)] = 1.0;
    }
    for p in &field.obstacles {
        grid[cell(7, p.x, p.y)] = 1.0;
    }

    let flag = |b: bool| if b { 1.0 } else { 0.0 };
    vector.fill(0.0);
    vector[snake.last_moved_dir.index()] = 1.0;
    vector[4] = (snake.pending_growth as f32 / PENDING_GROWTH_SCALE).min(1.0);
    vector[5] = flag(snake.held_item == Some(HeldItem::BlockClear));
    vector[6] = flag(snake.held_item == Some(HeldItem::BlockJam));
    vector[7] = snake.boost_remaining as f32 / rules.boost_duration_ticks as f32;
    vector[8] = flag(snake.boost_ready());
    vector[9] = env.remaining_ticks() as f32 / rules.time_limit_ticks as f32;
    vector[10] = ((snake.score - opponent.snake.score) as f32 / SCORE_DIFF_SCALE).clamp(-1.0, 1.0);
    vector[11] = flag(opponent.snake.held_item == Some(HeldItem::BlockJam));
    vector[12] = (field.incoming_jams.len() as f32 / INCOMING_JAMS_SCALE).min(1.0);
}
