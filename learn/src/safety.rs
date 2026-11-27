//! 推論時の安全装置 (docs/training/endgame.md の提案 1)。Web 版の web/src/ai/safety.ts と同じ手順
//!
//! 行動を選ぶ前に、行動ごとに次の移動の行き先を調べて 4 段階に分け、いちばん良い段階の中から Q 値が最大の行動を選ぶ。
//! 1. 助かる: 今の体がすべて退くまで動き続けられる経路がある (`trap::escape_by`)
//! 2. 判定できず: 探索が上限 (`trap::SAFETY_BUDGET`) を越えた
//! 3. 詰み: 動き続けられる経路が無い。この中では、行けるマスが多い手を選ぶ (相手が先に死ぬまで粘る)
//! 4. 衝突: 次の一手で衝突する
//!
//! ブースト・アイテム使用は向きを変えないので、今の向きに進む手として調べる。
//! ブロック消去を持っているときのアイテム使用は、お邪魔ブロックを消してから進む手として調べる。

use crate::env::field::Field;
use crate::env::game::Action;
use crate::env::rules::Rules;
use crate::env::snake::{Direction, HeldItem, Position};
use crate::trap::{self, Escape, Snapshot};

/// 行き先の段階。小さいほど良い
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    Open,
    Unknown,
    Trapped,
    Collide,
}

/// 1つの行動の行き先を調べた結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    pub class: Class,
    /// 進んだ先から、お邪魔ブロックと (次の移動で退く尻尾を除く) 体を避けて行けるマスの数。詰みの手どうしを比べるのに使う
    pub room: usize,
}

/// 安全装置が選んだ行動
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
    pub action: usize,
    /// Q 値が最大の行動を差し替えたか
    pub overridden: bool,
}

/// この行動で次に進む向きと、お邪魔ブロックを消してから進むか
fn move_of(field: &Field, action: Action) -> (Direction, bool) {
    let snake = &field.snake;
    let turn = |dir: Direction| {
        // 直前に進んだ方向の真逆は無視され、今の向きのまま進む
        if dir == snake.last_moved_dir.opposite() {
            snake.dir
        } else {
            dir
        }
    };
    match action {
        Action::Up => (turn(Direction::Up), false),
        Action::Down => (turn(Direction::Down), false),
        Action::Left => (turn(Direction::Left), false),
        Action::Right => (turn(Direction::Right), false),
        Action::Boost => (snake.dir, false),
        Action::UseItem => (
            snake.dir,
            snake.held_item == Some(HeldItem::BlockClear) && !field.obstacles.is_empty(),
        ),
    }
}

/// `dir` に進んだときの行き先を調べる。`cleared` ならお邪魔ブロックを消してから進む
pub fn judge(field: &Field, rules: &Rules, dir: Direction, cleared: bool) -> Verdict {
    let snake = &field.snake;
    let next = snake.head().step(dir);
    let blocked = !Field::in_bounds(next, rules)
        || (!cleared && field.obstacles.contains(&next))
        || snake.collides_with_self(next);
    if blocked {
        return Verdict {
            class: Class::Collide,
            room: 0,
        };
    }
    let mut snapshot = Snapshot::of(field);
    if cleared {
        snapshot.obstacles.clear();
    }
    let class = match trap::escape_by(
        &snapshot,
        rules.width,
        rules.height,
        dir,
        trap::SAFETY_BUDGET,
    ) {
        Escape::Open => Class::Open,
        Escape::Unknown => Class::Unknown,
        Escape::Trapped => Class::Trapped,
    };
    Verdict {
        class,
        room: room(&snapshot, rules, next),
    }
}

/// `start` から、お邪魔ブロックと体 (次の移動で退く尻尾を除く) を避けて行けるマスの数 (`start` を含む)
fn room(snapshot: &Snapshot, rules: &Rules, start: Position) -> usize {
    let (w, h) = (rules.width, rules.height);
    let index = |p: Position| (p.y * w + p.x) as usize;
    let mut blocked = vec![false; rules.cell_count()];
    for &p in &snapshot.obstacles {
        blocked[index(p)] = true;
    }
    let tail_moves = snapshot.pending_growth == 0;
    let kept = snapshot.body.len() - usize::from(tail_moves);
    for &p in &snapshot.body[..kept] {
        blocked[index(p)] = true;
    }
    blocked[index(start)] = true;
    let mut stack = vec![start];
    let mut count = 1;
    while let Some(p) = stack.pop() {
        for dir in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let n = p.step(dir);
            if n.x < 0 || n.x >= w || n.y < 0 || n.y >= h || blocked[index(n)] {
                continue;
            }
            blocked[index(n)] = true;
            count += 1;
            stack.push(n);
        }
    }
    count
}

/// Q 値 `q` (Action::ALL の順) から、安全装置を通して行動を選ぶ
pub fn choose(field: &Field, rules: &Rules, q: &[f32]) -> Choice {
    let greedy = argmax(q);
    // 同じ行き先は 1 回だけ調べる (向き 4 つ × お邪魔ブロックを消すかどうか)
    let mut cache: Vec<((Direction, bool), Verdict)> = Vec::with_capacity(4);
    let mut verdict = |action: Action| {
        let key = move_of(field, action);
        if let Some(&(_, v)) = cache.iter().find(|(k, _)| *k == key) {
            return v;
        }
        let v = judge(field, rules, key.0, key.1);
        cache.push((key, v));
        v
    };
    let verdicts: Vec<Verdict> = Action::ALL.iter().map(|&a| verdict(a)).collect();
    let mut best = greedy;
    for a in 0..q.len() {
        if better(verdicts[a], q[a], verdicts[best], q[best]) {
            best = a;
        }
    }
    Choice {
        action: best,
        overridden: best != greedy,
    }
}

/// 行動 a (行き先 va, Q 値 qa) が行動 b より良いか
fn better(va: Verdict, qa: f32, vb: Verdict, qb: f32) -> bool {
    if va.class != vb.class {
        return va.class < vb.class;
    }
    if va.class == Class::Trapped && va.room != vb.room {
        return va.room > vb.room;
    }
    qa > qb
}

/// Q 値が最大の行動。同じ値なら先のもの
pub fn argmax(q: &[f32]) -> usize {
    let mut best = 0;
    for (i, &v) in q.iter().enumerate().skip(1) {
        if v > q[best] {
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::env::game::GameEnv;
    use std::path::Path;

    fn rules() -> Rules {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yaml");
        Rules::from_config(&Config::load(&path).unwrap().game)
    }

    fn pos(x: i32, y: i32) -> Position {
        Position { x, y }
    }

    /// アイテムの無いフィールドに、`body` (先頭が頭) のヘビを `dir` 向きで置く
    fn field(body: &[(i32, i32)], dir: Direction, jams: &[(i32, i32)]) -> (Field, Rules) {
        let rules = rules();
        let mut field = GameEnv::new(rules, 1).fields[0].clone();
        field.items.clear();
        field.snake.body = body.iter().map(|&(x, y)| pos(x, y)).collect();
        field.snake.dir = dir;
        field.snake.last_moved_dir = dir;
        field.obstacles = jams.iter().map(|&(x, y)| pos(x, y)).collect();
        (field, rules)
    }

    /// Q 値を行動の並び (up, down, left, right, boost, use_item) で作る
    fn q(values: [f32; 6]) -> [f32; 6] {
        values
    }

    #[test]
    fn keeps_the_greedy_action_when_it_is_safe() {
        let (f, r) = field(&[(4, 4), (4, 5), (4, 6)], Direction::Up, &[]);
        let choice = choose(&f, &r, &q([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
        assert_eq!(
            choice,
            Choice {
                action: 0,
                overridden: false
            }
        );
    }

    #[test]
    fn avoids_a_collision() {
        // 上端で上を向いている。上・ブースト・アイテム使用は壁にぶつかるので、Q 値が次に大きい安全な手にする
        let (f, r) = field(&[(4, 0), (4, 1), (4, 2)], Direction::Up, &[]);
        let choice = choose(&f, &r, &q([5.0, 9.0, 1.0, 2.0, 4.0, 3.0]));
        // 下は逆走で無視され、上に進んで衝突する
        assert_eq!(
            choice,
            Choice {
                action: 3,
                overridden: true
            }
        );
    }

    #[test]
    fn avoids_a_pocket_smaller_than_the_body() {
        // 上に進むと 2 マスの袋小路に入る (長さ 5)。右なら広い
        let (f, r) = field(
            &[(0, 2), (0, 3), (0, 4), (1, 4), (2, 4)],
            Direction::Up,
            &[(1, 1), (1, 0)],
        );
        assert_eq!(judge(&f, &r, Direction::Up, false).class, Class::Trapped);
        assert_eq!(judge(&f, &r, Direction::Up, false).room, 2);
        assert_eq!(judge(&f, &r, Direction::Right, false).class, Class::Open);
        let choice = choose(&f, &r, &q([9.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
        assert_eq!(
            choice,
            Choice {
                action: 3,
                overridden: true
            }
        );
    }

    #[test]
    fn prefers_the_larger_room_when_every_move_is_trapped() {
        // 頭 (2,1) は上向き (長さ 6)。上はお邪魔ブロック、下は逆走、左は 1 マス、右は 3 マスの袋小路で、どちらも詰み
        //   y=0: . X X X X .
        //   y=1: X . H . . X
        //   y=2: . X b . X .
        //   y=3: . . b X . .
        //   y=4: . . b b b .
        let (f, r) = field(
            &[(2, 1), (2, 2), (2, 3), (2, 4), (3, 4), (4, 4)],
            Direction::Up,
            &[
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (0, 1),
                (5, 1),
                (1, 2),
                (4, 2),
                (3, 3),
            ],
        );
        assert_eq!(judge(&f, &r, Direction::Up, false).class, Class::Collide);
        let left = judge(&f, &r, Direction::Left, false);
        let right = judge(&f, &r, Direction::Right, false);
        assert_eq!((left.class, left.room), (Class::Trapped, 1));
        assert_eq!((right.class, right.room), (Class::Trapped, 3));
        // Q 値は左が最大でも、長く粘れる右を選ぶ
        let choice = choose(&f, &r, &q([0.0, 0.0, 9.0, 1.0, 0.0, 0.0]));
        assert_eq!(
            choice,
            Choice {
                action: 3,
                overridden: true
            }
        );
    }

    #[test]
    fn using_block_clear_counts_as_moving_through_cleared_blocks() {
        // 上にお邪魔ブロックがある。ブロック消去を持っていれば、アイテム使用で消してから上に進める
        let (mut f, r) = field(&[(4, 4), (4, 5), (4, 6)], Direction::Up, &[(4, 3)]);
        f.snake.held_item = Some(HeldItem::BlockClear);
        assert_eq!(move_of(&f, Action::UseItem), (Direction::Up, true));
        assert_eq!(judge(&f, &r, Direction::Up, false).class, Class::Collide);
        assert_eq!(judge(&f, &r, Direction::Up, true).class, Class::Open);
        let choice = choose(&f, &r, &q([2.0, 0.0, 0.0, 0.0, 9.0, 5.0]));
        // ブーストはブロックにぶつかる。アイテム使用なら消してから進める
        assert_eq!(
            choice,
            Choice {
                action: 5,
                overridden: true
            }
        );
        // お邪魔を持っているだけならブロックは消えない
        f.snake.held_item = Some(HeldItem::BlockJam);
        assert_eq!(move_of(&f, Action::UseItem), (Direction::Up, false));
    }

    #[test]
    fn argmax_takes_the_first_of_ties() {
        assert_eq!(argmax(&[1.0, 3.0, 3.0, 0.0]), 1);
        assert_eq!(argmax(&[2.0]), 0);
    }
}
