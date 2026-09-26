use super::field::{Item, ItemType};
use super::game::{Action, EndReason, GameEnv};
use super::observation::{encode, GRID_CHANNELS, VECTOR_FEATURES};
use super::rules::Rules;
use super::snake::{Direction, HeldItem, Position};
use crate::config::Config;
use std::collections::VecDeque;
use std::path::Path;

/// config.yaml のルール。盤面の座標などは 16x16 で書いているので、学習する盤面の大きさによらず 16x16 にする
fn rules() -> Rules {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yaml");
    let mut rules = Rules::from_config(&Config::load(&path).unwrap().game);
    (rules.width, rules.height) = (16, 16);
    rules
}

/// アイテムの無い盤面で始める
fn empty_env_with(rules: Rules) -> GameEnv {
    let mut env = GameEnv::new(rules, 0);
    for field in &mut env.fields {
        field.items.clear();
    }
    env
}

fn empty_env() -> GameEnv {
    empty_env_with(rules())
}

fn pos(x: i32, y: i32) -> Position {
    Position { x, y }
}

/// プレイヤー0が次に移動するティックまで進め、そのティックでは `action` を入力する
fn move_once(env: &mut GameEnv, action: Option<Action>) {
    while !env.needs_decision(0) {
        env.step([None, None]);
    }
    env.step([action, None]);
}

fn set_body(env: &mut GameEnv, player: usize, body: &[Position], dir: Direction) {
    let snake = &mut env.fields[player].snake;
    snake.body = body.iter().copied().collect::<VecDeque<_>>();
    snake.dir = dir;
    snake.last_moved_dir = dir;
}

#[test]
fn seconds_are_rounded_to_ticks() {
    let r = rules();
    assert_eq!(r.time_limit_ticks, 300);
    assert_eq!(r.normal_interval_ticks, 2);
    assert_eq!(r.boost_interval_ticks, 1);
    assert_eq!(r.boost_duration_ticks, 50);
    assert_eq!(r.boost_cooldown_ticks, 75);
    // 0.6 / 0.1 を切り捨てると 5 になってしまう
    assert_eq!(r.apple_respawn_ticks, 6);
    assert_eq!(r.special_respawn_ticks, 10);
    assert_eq!(r.jam_delay_ticks, 2);
}

#[test]
fn initial_state_matches_rules() {
    let env = GameEnv::new(rules(), 1);
    for field in &env.fields {
        let body: Vec<_> = field.snake.body.iter().copied().collect();
        assert_eq!(body, vec![pos(8, 8), pos(8, 9), pos(8, 10)]);
        assert_eq!(field.snake.dir, Direction::Up);
        let count = |kind| field.items.iter().filter(|i| i.kind == kind).count();
        assert_eq!(count(ItemType::NormalApple), 3);
        assert_eq!(count(ItemType::PoisonApple), 2);
        assert_eq!(count(ItemType::GoldApple), 1);
        assert_eq!(count(ItemType::BlockClear), 1);
        assert_eq!(count(ItemType::BlockJam), 1);
        assert!(field
            .items
            .iter()
            .all(|i| !field.snake.body.contains(&i.pos)));
    }
}

#[test]
fn snake_moves_every_normal_interval_and_decides_just_before() {
    let mut env = empty_env();
    assert!(!env.needs_decision(0));
    env.step([None, None]);
    assert_eq!(env.fields[0].snake.head(), pos(8, 8));
    assert!(env.needs_decision(0));
    env.step([None, None]);
    assert_eq!(env.fields[0].snake.head(), pos(8, 7));
    assert_eq!(env.fields[0].snake.len(), 3);
}

#[test]
fn reversing_against_last_move_is_ignored() {
    let mut env = empty_env();
    // 同じ移動間隔の中で 左 → 下 と入力しても、下 (直前の移動の逆) は無視される
    env.step([Some(Action::Left), None]);
    env.step([Some(Action::Down), None]);
    assert!(env.fields[0].snake.alive);
    assert_eq!(env.fields[0].snake.head(), pos(7, 8));
}

#[test]
fn normal_apple_scores_and_grows_on_next_move() {
    let mut env = empty_env();
    env.fields[0].items.push(Item {
        pos: pos(8, 7),
        kind: ItemType::NormalApple,
    });
    move_once(&mut env, None);
    let snake = &env.fields[0].snake;
    assert_eq!((snake.score, snake.len(), snake.pending_growth), (1, 3, 1));
    assert!(env.fields[0].items.is_empty());
    assert_eq!(env.fields[0].eaten, Some(ItemType::NormalApple));

    // 取ったことが残るのは、取ったティックだけ
    env.step([None, None]);
    assert_eq!(env.fields[0].eaten, None);

    move_once(&mut env, None);
    assert_eq!(env.fields[0].snake.len(), 4);

    // 食べてから 0.6 秒 (6 ティック) 後に再出現する。ここまでに 2 ティック経過済み
    for _ in 0..3 {
        env.step([None, None]);
    }
    assert!(env.fields[0].items.is_empty());
    env.step([None, None]);
    assert_eq!(env.fields[0].items.len(), 1);
}

#[test]
fn gold_apple_grows_by_three() {
    let mut env = empty_env();
    env.fields[0].items.push(Item {
        pos: pos(8, 7),
        kind: ItemType::GoldApple,
    });
    for _ in 0..4 {
        move_once(&mut env, None);
    }
    let snake = &env.fields[0].snake;
    assert_eq!((snake.score, snake.len()), (3, 6));
}

#[test]
fn poison_apple_shrinks_immediately_and_score_stays_non_negative() {
    let mut env = empty_env();
    env.fields[0].items.push(Item {
        pos: pos(8, 7),
        kind: ItemType::PoisonApple,
    });
    move_once(&mut env, None);
    let snake = &env.fields[0].snake;
    assert_eq!((snake.score, snake.len()), (0, 1));
    assert!(snake.alive);
}

#[test]
fn poison_cancels_pending_growth_first() {
    let mut env = empty_env();
    env.fields[0].snake.pending_growth = 3;
    env.fields[0].items.push(Item {
        pos: pos(8, 7),
        kind: ItemType::PoisonApple,
    });
    move_once(&mut env, None);
    let snake = &env.fields[0].snake;
    // 移動で伸びる予定を 1 消費し、残り 2 を毒で打ち消す
    assert_eq!((snake.len(), snake.pending_growth), (4, 0));
}

#[test]
fn snake_can_move_into_its_tail_cell_unless_growing() {
    // 2x2 で輪になった体。頭 (8,8) から下に進むと今の尻尾 (8,9) に入る
    let body = [pos(8, 8), pos(9, 8), pos(9, 9), pos(8, 9)];

    let mut env = empty_env();
    set_body(&mut env, 0, &body, Direction::Left);
    env.fields[0].snake.dir = Direction::Down;
    move_once(&mut env, None);
    assert!(env.fields[0].snake.alive);
    assert_eq!(env.fields[0].snake.head(), pos(8, 9));

    let mut env = empty_env();
    set_body(&mut env, 0, &body, Direction::Left);
    env.fields[0].snake.dir = Direction::Down;
    env.fields[0].snake.pending_growth = 1;
    move_once(&mut env, None);
    assert!(!env.fields[0].snake.alive);
}

#[test]
fn hitting_wall_is_immediate_loss() {
    let mut env = empty_env();
    set_body(
        &mut env,
        0,
        &[pos(8, 0), pos(8, 1), pos(8, 2)],
        Direction::Up,
    );
    env.fields[0].snake.score = 10;
    move_once(&mut env, None);
    let result = env.result.expect("試合が終わっていない");
    assert_eq!(result.winner, Some(1));
    assert_eq!(result.reason, EndReason::Death);
}

#[test]
fn hitting_obstacle_is_immediate_loss() {
    let mut env = empty_env();
    env.fields[1].obstacles.push(pos(8, 7));
    move_once(&mut env, None);
    let result = env.result.expect("試合が終わっていない");
    assert_eq!(result.winner, Some(0));
}

#[test]
fn simultaneous_deaths_are_decided_by_score() {
    for (scores, winner) in [([2, 5], Some(1)), ([4, 4], None)] {
        let mut env = empty_env();
        for p in 0..2 {
            set_body(&mut env, p, &[pos(3, 0), pos(3, 1)], Direction::Up);
            env.fields[p].snake.score = scores[p];
        }
        move_once(&mut env, None);
        let result = env.result.expect("試合が終わっていない");
        assert_eq!(result.winner, winner);
        assert_eq!(result.reason, EndReason::Death);
    }
}

/// 盤面のすべてのマスを、左上から 1 行ごとに折り返してたどる順に並べる
fn serpentine(width: i32, height: i32) -> Vec<Position> {
    (0..height)
        .flat_map(|y| (0..width).map(move |i| pos(if y % 2 == 0 { i } else { width - 1 - i }, y)))
        .collect()
}

/// 5 × 5 の盤面で、`players` のヘビがあと 1 マスで盤面を埋める状態にする。
/// 最初のマスにはお邪魔ブロックを置き、最後のマスだけを空け、頭はその手前で右を向く
fn env_one_cell_before_filled(players: &[usize], growth: u32) -> GameEnv {
    let mut r = rules();
    (r.width, r.height) = (5, 5);
    let mut env = empty_env_with(r);
    let path = serpentine(5, 5);
    let body: Vec<Position> = path[1..24].iter().rev().copied().collect();
    for &p in players {
        env.fields[p].obstacles.push(path[0]);
        set_body(&mut env, p, &body, Direction::Right);
        env.fields[p].snake.pending_growth = growth;
    }
    env
}

#[test]
fn filling_all_cells_except_obstacles_wins() {
    let mut env = env_one_cell_before_filled(&[0], 1);
    env.fields[1].snake.score = 10;
    move_once(&mut env, None);
    let result = env.result.expect("試合が終わっていない");
    assert_eq!(result.winner, Some(0));
    assert_eq!(result.reason, EndReason::Filled);

    // 伸びる予定が無ければ尻尾が退くので、埋まらずに続く
    let mut env = env_one_cell_before_filled(&[0], 0);
    move_once(&mut env, None);
    assert!(!env.is_over());
}

#[test]
fn simultaneous_fills_are_decided_by_score() {
    for (scores, winner) in [([2, 5], Some(1)), ([4, 4], None)] {
        let mut env = env_one_cell_before_filled(&[0, 1], 1);
        for p in 0..2 {
            env.fields[p].snake.score = scores[p];
        }
        move_once(&mut env, None);
        let result = env.result.expect("試合が終わっていない");
        assert_eq!(result.winner, winner);
        assert_eq!(result.reason, EndReason::Filled);
    }
}

#[test]
fn time_up_is_decided_by_score() {
    let mut r = rules();
    r.normal_interval_ticks = 10_000; // 動かないようにする
    let mut env = empty_env_with(r);
    env.fields[0].snake.score = 3;
    for _ in 0..299 {
        env.step([None, None]);
    }
    assert!(!env.is_over());
    env.step([None, None]);
    let result = env.result.expect("試合が終わっていない");
    assert_eq!(result.winner, Some(0));
    assert_eq!(result.reason, EndReason::TimeUp);
}

#[test]
fn jam_block_arrives_after_delay_away_from_opponent_head() {
    for seed in 0..20 {
        let mut env = GameEnv::new(rules(), seed);
        env.fields[0].snake.held_item = Some(HeldItem::BlockJam);
        env.step([Some(Action::UseItem), None]);
        assert!(env.fields[0].snake.held_item.is_none());
        env.step([None, None]);
        assert!(env.fields[1].obstacles.is_empty());
        env.step([None, None]);
        assert_eq!(env.fields[1].obstacles.len(), 1);
        let block = env.fields[1].obstacles[0];
        assert!(block.manhattan(env.fields[1].snake.head()) > 2);
        assert!(!env.fields[1].items.iter().any(|i| i.pos == block));
    }
}

#[test]
fn block_clear_removes_own_obstacles_only() {
    let mut env = empty_env();
    env.fields[0].obstacles = vec![pos(0, 0), pos(1, 1)];
    env.fields[1].obstacles = vec![pos(0, 0)];
    env.fields[0].snake.held_item = Some(HeldItem::BlockClear);
    env.step([Some(Action::UseItem), None]);
    assert!(env.fields[0].obstacles.is_empty());
    assert_eq!(env.fields[1].obstacles.len(), 1);
}

#[test]
fn boost_moves_every_tick_then_cools_down() {
    let mut r = rules();
    r.boost_duration_ticks = 6;
    r.boost_cooldown_ticks = 4;
    let mut env = empty_env_with(r);
    set_body(&mut env, 0, &[pos(0, 15)], Direction::Right);

    move_once(&mut env, Some(Action::Boost));
    assert_eq!(env.fields[0].snake.head(), pos(1, 15));
    let mut xs = Vec::new();
    for _ in 0..8 {
        env.step([None, None]);
        xs.push(env.fields[0].snake.head().x);
    }
    // ブースト中は毎ティック、終了後は 2 ティックに1回動く
    assert_eq!(xs, vec![2, 3, 4, 5, 6, 6, 7, 7]);
    // クールダウンはブースト終了のティックから 4 ティック
    assert!(!env.fields[0].snake.boost_ready());
    env.step([None, None]);
    assert!(env.fields[0].snake.boost_ready());
}

#[test]
fn observation_encodes_own_view() {
    let mut env = empty_env();
    env.fields[0].items.push(Item {
        pos: pos(2, 3),
        kind: ItemType::GoldApple,
    });
    env.fields[0].snake.score = 5;
    env.fields[1].snake.held_item = Some(HeldItem::BlockJam);

    let plane = 16 * 16;
    let mut grid = vec![0.0; GRID_CHANNELS * plane];
    let mut vector = vec![0.0; VECTOR_FEATURES];
    encode(&env, 0, &mut grid, &mut vector);
    let at = |c: usize, x: usize, y: usize| grid[c * plane + y * 16 + x];

    assert_eq!(at(0, 8, 8), 1.0);
    assert_eq!(at(1, 8, 9), 1.0); // 首
    assert_eq!(at(1, 8, 10), 0.5); // 尻尾
    assert_eq!(at(3, 2, 3), 1.0);
    assert_eq!(grid[8 * plane..].iter().sum::<f32>(), plane as f32);
    assert_eq!(&vector[0..4], &[1.0, 0.0, 0.0, 0.0]);
    assert_eq!(vector[8], 1.0); // ブースト可能
    assert_eq!(vector[9], 1.0); // 残り時間
    assert_eq!(vector[10], 0.5); // スコア差 5 / 10
    assert_eq!(vector[11], 1.0); // 相手がお邪魔を所持

    encode(&env, 1, &mut grid, &mut vector);
    assert_eq!(vector[10], -0.5);
    assert_eq!(vector[11], 0.0);
}
