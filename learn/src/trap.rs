//! 衝突した試合の分析。死んだ手を選んだときに、まだ助かる道があったか (うっかり) か、
//! もうどの手でも助からなかったか (詰み) を判定する
//!
//! 「助かる道がある」とは、今の体がすべて退くまで (長さ + 伸びる予定の手数だけ) 動き続けられる経路があることとする。
//! 体は尻尾から順に空いていくので、その順番も考えて探す。次のものは考えない近似である。
//! - 経路の途中でアイテムを取ったときの効果 (リンゴで伸びる・毒リンゴで縮む・ブロック消去を拾う)
//! - これから相手に置かれるお邪魔ブロックと、手持ちのブロック消去を使うこと

use crate::env::field::Field;
use crate::env::rules::Rules;
use crate::env::snake::{Direction, Position};

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

/// 1回の判定で調べる局面の数の上限。越えたら「判定できず」にする
const SEARCH_BUDGET: usize = 200_000;

/// 衝突したもの
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    Wall,
    Body,
    Jam,
}

impl DeathCause {
    pub fn index(self) -> usize {
        self as usize
    }
}

/// 死んだ直後のフィールドから、何に衝突したかを調べる。死んだ移動では頭が進まないので、頭の 1 マス先が衝突したマス
pub fn death_cause(field: &Field, rules: &Rules) -> DeathCause {
    let next = field.snake.next_head();
    if !Field::in_bounds(next, rules) {
        DeathCause::Wall
    } else if field.obstacles.contains(&next) {
        DeathCause::Jam
    } else {
        DeathCause::Body
    }
}

/// 衝突した試合の死に方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathKind {
    /// 死んだ手を選んだときに、まだ助かる道があった
    Blunder,
    /// 死んだ手を選んだときには、どの手でも助からなかった。`moves` 手前の判断で詰みに入った
    Trapped { moves: u32 },
    /// 探索が上限を越えて判定できなかった
    Unknown,
}

/// 判定に要る、行動を決めたときのフィールドの様子
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// 先頭が頭
    pub body: Vec<Position>,
    pub pending_growth: u32,
    pub last_moved_dir: Direction,
    pub obstacles: Vec<Position>,
}

impl Snapshot {
    pub fn of(field: &Field) -> Self {
        Self {
            body: field.snake.body.iter().copied().collect(),
            pending_growth: field.snake.pending_growth,
            last_moved_dir: field.snake.last_moved_dir,
            obstacles: field.obstacles.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escape {
    /// 助かる道がある
    Open,
    /// どの手でも助からない
    Trapped,
    /// 探索が上限を越えた
    Unknown,
}

/// `history` は1匹のヘビが行動を決めたときの様子を古い順に並べたもので、最後が死んだ手を選んだとき
pub fn classify(history: &[Snapshot], width: i32, height: i32) -> DeathKind {
    let Some(last) = history.last() else {
        return DeathKind::Unknown;
    };
    match escape(last, width, height) {
        Escape::Open => DeathKind::Blunder,
        Escape::Unknown => DeathKind::Unknown,
        Escape::Trapped => {
            // さかのぼって、まだ助かる道があった最後の判断を探す。その判断で選んだ手が詰みに入った手
            let mut moves = 1;
            for snapshot in history.iter().rev().skip(1) {
                if escape(snapshot, width, height) != Escape::Trapped {
                    break;
                }
                moves += 1;
            }
            DeathKind::Trapped { moves }
        }
    }
}

/// 今の体がすべて退くまで動き続けられる経路があるか
pub fn escape(snapshot: &Snapshot, width: i32, height: i32) -> Escape {
    let cells = (width * height) as usize;
    let len = snapshot.body.len() as u32;
    let growth = snapshot.pending_growth;
    // free_at[i] は、マス i に入れるようになる移動の番号 (1 手目が 1)。0 は最初から空き
    let mut free_at = vec![0u32; cells];
    for &pos in &snapshot.obstacles {
        free_at[index(pos, width)] = u32::MAX;
    }
    // 頭から i 番目の体は、伸びる予定の分だけ尻尾が止まったあと、尻尾から順に退く
    for (i, &pos) in snapshot.body.iter().enumerate() {
        free_at[index(pos, width)] = growth + len - i as u32;
    }
    let mut goal = growth + len;
    // 伸びている途中で盤面が埋まれば勝ちなので、そこまで動ければよい
    let fill_at = (cells as u32)
        .saturating_sub(snapshot.obstacles.len() as u32)
        .saturating_sub(len);
    if fill_at <= growth {
        goal = goal.min(fill_at);
    }

    let mut search = Search {
        width,
        height,
        goal,
        free_at,
        on_path: vec![false; cells],
        seen: vec![false; cells],
        queue: Vec::with_capacity(cells),
        nodes: 0,
    };
    match search.run(snapshot.body[0], snapshot.last_moved_dir, 0) {
        Some(true) => Escape::Open,
        Some(false) => Escape::Trapped,
        None => Escape::Unknown,
    }
}

fn index(pos: Position, width: i32) -> usize {
    (pos.y * width + pos.x) as usize
}

struct Search {
    width: i32,
    height: i32,
    goal: u32,
    free_at: Vec<u32>,
    /// 探索中の経路で通ったマス。経路は新しい体になり、goal 手までは退かない
    on_path: Vec<bool>,
    seen: Vec<bool>,
    queue: Vec<Position>,
    nodes: usize,
}

impl Search {
    fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }

    /// `moves` 手動いて頭が `head` にあるところから、goal 手まで動き続けられるか。上限を越えたら None
    fn run(&mut self, head: Position, last_dir: Direction, moves: u32) -> Option<bool> {
        if moves >= self.goal {
            return Some(true);
        }
        self.nodes += 1;
        if self.nodes > SEARCH_BUDGET {
            return None;
        }
        // 経路は同じマスを通れないので、行けるマスが残りの手数より少なければ届かない
        if self.reachable(head) < (self.goal - moves) as usize {
            return Some(false);
        }
        for dir in DIRECTIONS {
            // 直前に進んだ方向の真逆は無視される
            if dir == last_dir.opposite() {
                continue;
            }
            let next = head.step(dir);
            if !self.in_bounds(next) {
                continue;
            }
            let i = index(next, self.width);
            if self.on_path[i] || self.free_at[i] > moves + 1 {
                continue;
            }
            self.on_path[i] = true;
            let found = self.run(next, dir, moves + 1);
            self.on_path[i] = false;
            if found != Some(false) {
                return found;
            }
        }
        Some(false)
    }

    /// `head` から、経路とお邪魔ブロックを避けて行けるマスの数 (goal 手までに空かない体のマスも除く)
    fn reachable(&mut self, head: Position) -> usize {
        self.seen.fill(false);
        self.queue.clear();
        self.queue.push(head);
        let mut count = 0;
        while let Some(pos) = self.queue.pop() {
            for dir in DIRECTIONS {
                let next = pos.step(dir);
                if !self.in_bounds(next) {
                    continue;
                }
                let i = index(next, self.width);
                if self.seen[i] || self.on_path[i] || self.free_at[i] > self.goal {
                    continue;
                }
                self.seen[i] = true;
                count += 1;
                self.queue.push(next);
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::env::game::GameEnv;
    use crate::env::snake::Snake;
    use std::path::Path;

    fn pos(x: i32, y: i32) -> Position {
        Position { x, y }
    }

    fn snapshot(
        body: &[(i32, i32)],
        last: Direction,
        growth: u32,
        jams: &[(i32, i32)],
    ) -> Snapshot {
        Snapshot {
            body: body.iter().map(|&(x, y)| pos(x, y)).collect(),
            pending_growth: growth,
            last_moved_dir: last,
            obstacles: jams.iter().map(|&(x, y)| pos(x, y)).collect(),
        }
    }

    #[test]
    fn open_board_is_not_trapped() {
        let s = snapshot(&[(2, 2), (2, 3), (2, 4)], Direction::Up, 0, &[]);
        assert_eq!(escape(&s, 5, 5), Escape::Open);
    }

    #[test]
    fn no_safe_move_is_trapped() {
        // 左上の角で上を向き、右はお邪魔ブロック。下は逆走なので選べない
        let s = snapshot(&[(0, 0), (0, 1), (0, 2)], Direction::Up, 0, &[(1, 0)]);
        assert_eq!(escape(&s, 5, 5), Escape::Trapped);
    }

    #[test]
    fn pocket_smaller_than_the_body_is_trapped() {
        // 次の一手は安全だが、その先は 2 マスしかなく、長さ 5 の体が退くまで動けない
        let s = snapshot(
            &[(0, 1), (0, 2), (0, 3), (0, 4), (1, 4)],
            Direction::Up,
            0,
            &[(1, 1), (2, 0)],
        );
        assert_eq!(escape(&s, 5, 5), Escape::Trapped);
    }

    #[test]
    fn following_the_tail_escapes() {
        // 2x2 の盤面で空きは 1 マスだけだが、尻尾を追いかければ回り続けられる
        let s = snapshot(&[(0, 0), (1, 0), (1, 1)], Direction::Left, 0, &[]);
        assert_eq!(escape(&s, 2, 2), Escape::Open);
    }

    #[test]
    fn growing_into_the_last_cell_fills_the_board() {
        // 伸びる予定があると尻尾は退かないが、最後の空きマスに入れば盤面が埋まって勝ち
        let s = snapshot(&[(0, 0), (1, 0), (1, 1)], Direction::Left, 1, &[]);
        assert_eq!(escape(&s, 2, 2), Escape::Open);
        // 伸びる予定が 2 なら、1 手目で盤面が埋まる
        let s = snapshot(&[(0, 0), (1, 0), (1, 1)], Direction::Left, 2, &[]);
        assert_eq!(escape(&s, 2, 2), Escape::Open);
    }

    #[test]
    fn counts_moves_since_the_trap_was_entered() {
        let open = snapshot(&[(2, 2), (2, 3), (2, 4)], Direction::Up, 0, &[]);
        let trapped = snapshot(&[(0, 0), (0, 1), (0, 2)], Direction::Up, 0, &[(1, 0)]);
        assert_eq!(
            classify(std::slice::from_ref(&open), 5, 5),
            DeathKind::Blunder
        );
        assert_eq!(
            classify(&[open.clone(), trapped.clone()], 5, 5),
            DeathKind::Trapped { moves: 1 }
        );
        assert_eq!(
            classify(&[open, trapped.clone(), trapped.clone(), trapped], 5, 5),
            DeathKind::Trapped { moves: 3 }
        );
        assert_eq!(classify(&[], 5, 5), DeathKind::Unknown);
    }

    #[test]
    fn finds_what_the_snake_hit() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yaml");
        let rules = Rules::from_config(&Config::load(&path).unwrap().game);
        let mut env = GameEnv::new(rules, 1);
        // 上端で上を向いているので、最初の移動で壁にぶつかる
        env.fields[0].snake = Snake::new(pos(2, 0), &rules);
        while env.fields[0].snake.alive {
            env.step([None, None]);
        }
        assert_eq!(death_cause(&env.fields[0], &rules), DeathCause::Wall);
    }
}
