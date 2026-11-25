//! AI の入力 (観測) の作り方。Web 側でも同じ手順で作る必要がある (RULES.md 参照)

use super::field::{Field, ItemType};
use super::game::GameEnv;
use super::snake::{HeldItem, Position};

/// 盤面チャネル。値は [チャネル][y][x] の順に並べる。
/// 0〜8 が自分のフィールド、9〜16 が相手のフィールド (自分と同じ並び)。
/// 相手の分は後から末尾に足したので、先頭 9 チャネルだけを使う古いモデルもそのまま動かせる (`input_sizes`)
pub const GRID_CHANNEL_NAMES: [&str; 17] = [
    "own_head",
    "own_body",
    "normal_apple",
    "gold_apple",
    "poison_apple",
    "block_clear_item",
    "block_jam_item",
    "obstacle",
    "in_bounds",
    "opp_head",
    "opp_body",
    "opp_normal_apple",
    "opp_gold_apple",
    "opp_poison_apple",
    "opp_block_clear_item",
    "opp_block_jam_item",
    "opp_obstacle",
];
pub const GRID_CHANNELS: usize = GRID_CHANNEL_NAMES.len();

/// 状態ベクトル。0〜12 が自分の状態と相手との比較、13〜21 が相手の状態 (後から末尾に足した)
pub const VECTOR_FEATURE_NAMES: [&str; 22] = [
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
    "opp_dir_up",
    "opp_dir_down",
    "opp_dir_left",
    "opp_dir_right",
    "opp_pending_growth",
    "opp_held_block_clear",
    "opp_boost_remaining",
    "opp_boost_ready",
    "opp_incoming_jams",
];
pub const VECTOR_FEATURES: usize = VECTOR_FEATURE_NAMES.len();

/// 自分のフィールドの最初のチャネル (own_head) と、相手のフィールドの最初のチャネル (opp_head)
const OWN_FIELD_CHANNEL: usize = 0;
const OPP_FIELD_CHANNEL: usize = 9;
/// すべてのマスが 1 のチャネル
const IN_BOUNDS_CHANNEL: usize = 8;

/// 状態ベクトルの正規化に使う定数。値 / 定数 を [0, 1] (score_diff は [-1, 1]) に切り詰める
pub const PENDING_GROWTH_SCALE: f32 = 5.0;
pub const SCORE_DIFF_SCALE: f32 = 10.0;
pub const INCOMING_JAMS_SCALE: f32 = 3.0;

/// フィールドの最初のチャネル (頭) から数えた、各アイテムのチャネルの位置
fn item_channel(kind: ItemType) -> usize {
    match kind {
        ItemType::NormalApple => 2,
        ItemType::GoldApple => 3,
        ItemType::PoisonApple => 4,
        ItemType::BlockClear => 5,
        ItemType::BlockJam => 6,
    }
}

/// モデルの観測の名前の並びが、この観測の先頭部分と一致すれば (盤面チャネル数, 状態ベクトルの長さ) を返す。
/// `encode` で書いた観測のうち、盤面は先頭のチャネル数分、状態ベクトルは先頭の長さ分をそのモデルに渡せばよい
pub fn input_sizes(grid_channels: &[String], vector_features: &[String]) -> Option<(usize, usize)> {
    let is_prefix = |names: &[String], all: &[&str]| {
        !names.is_empty() && names.len() <= all.len() && names.iter().zip(all).all(|(a, b)| a == b)
    };
    (is_prefix(grid_channels, &GRID_CHANNEL_NAMES)
        && is_prefix(vector_features, &VECTOR_FEATURE_NAMES))
    .then_some((grid_channels.len(), vector_features.len()))
}

/// 1つのフィールドを `first` チャネルから始まる 8 チャネル (頭・胴体・アイテム 5 種・お邪魔ブロック) に書き込む
fn encode_field(field: &Field, width: usize, plane: usize, first: usize, grid: &mut [f32]) {
    let cell = |channel: usize, p: Position| {
        (first + channel) * plane + p.y as usize * width + p.x as usize
    };
    let snake = &field.snake;
    grid[cell(0, snake.head())] = 1.0;
    // 胴体は首が 1.0、尻尾に向かって 1 / (長さ - 1) まで小さくなる
    let len = snake.len();
    for (i, &p) in snake.body.iter().enumerate().skip(1) {
        grid[cell(1, p)] = (len - i) as f32 / (len - 1) as f32;
    }
    for item in &field.items {
        grid[cell(item_channel(item.kind), item.pos)] = 1.0;
    }
    for &p in &field.obstacles {
        grid[cell(7, p)] = 1.0;
    }
}

fn flag(b: bool) -> f32 {
    if b {
        1.0
    } else {
        0.0
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

    grid.fill(0.0);
    grid[IN_BOUNDS_CHANNEL * plane..(IN_BOUNDS_CHANNEL + 1) * plane].fill(1.0);
    encode_field(field, w, plane, OWN_FIELD_CHANNEL, grid);
    encode_field(opponent, w, plane, OPP_FIELD_CHANNEL, grid);

    let snake = &field.snake;
    let opp = &opponent.snake;
    vector.fill(0.0);
    vector[snake.last_moved_dir.index()] = 1.0;
    vector[4] = (snake.pending_growth as f32 / PENDING_GROWTH_SCALE).min(1.0);
    vector[5] = flag(snake.held_item == Some(HeldItem::BlockClear));
    vector[6] = flag(snake.held_item == Some(HeldItem::BlockJam));
    vector[7] = snake.boost_remaining as f32 / rules.boost_duration_ticks as f32;
    vector[8] = flag(snake.boost_ready());
    vector[9] = env.remaining_ticks() as f32 / rules.time_limit_ticks as f32;
    vector[10] = ((snake.score - opp.score) as f32 / SCORE_DIFF_SCALE).clamp(-1.0, 1.0);
    vector[11] = flag(opp.held_item == Some(HeldItem::BlockJam));
    vector[12] = (field.incoming_jams.len() as f32 / INCOMING_JAMS_SCALE).min(1.0);

    // 相手の状態。相手がお邪魔を持っているかは 11 番にあるので、ここには入れない
    vector[13 + opp.last_moved_dir.index()] = 1.0;
    vector[17] = (opp.pending_growth as f32 / PENDING_GROWTH_SCALE).min(1.0);
    vector[18] = flag(opp.held_item == Some(HeldItem::BlockClear));
    vector[19] = opp.boost_remaining as f32 / rules.boost_duration_ticks as f32;
    vector[20] = flag(opp.boost_ready());
    vector[21] = (opponent.incoming_jams.len() as f32 / INCOMING_JAMS_SCALE).min(1.0);
}
