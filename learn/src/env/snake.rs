use super::rules::Rules;
use std::collections::VecDeque;

/// 盤面上の座標。左上が (0, 0) で、x は右、y は下に向かって増える
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn step(self, dir: Direction) -> Position {
        let (dx, dy) = dir.delta();
        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn manhattan(self, other: Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    /// 観測ベクトルの one-hot で使う並び (up, down, left, right)
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldItem {
    BlockClear,
    BlockJam,
}

#[derive(Debug, Clone)]
pub struct Snake {
    /// 先頭が頭
    pub body: VecDeque<Position>,
    /// 次の移動で進む方向
    pub dir: Direction,
    /// 直近の移動で実際に進んだ方向。逆走の判定はこちらで行う
    pub last_moved_dir: Direction,
    pub alive: bool,
    pub score: i32,
    pub held_item: Option<HeldItem>,
    /// これから伸びる残りマス数。0 より大きい間は移動しても尻尾が縮まない
    pub pending_growth: u32,
    /// 次の移動までの残りティック数。1 なら次のティックで移動する
    pub move_cooldown: u32,
    pub boost_remaining: u32,
    pub boost_cooldown: u32,
}

impl Snake {
    /// 頭を `head` に置き、胴体を下向きに伸ばした状態で上向きに生成する
    pub fn new(head: Position, rules: &Rules) -> Self {
        let body = (0..rules.initial_length as i32)
            .map(|i| Position {
                x: head.x,
                y: head.y + i,
            })
            .collect();

        Self {
            body,
            dir: Direction::Up,
            last_moved_dir: Direction::Up,
            alive: true,
            score: 0,
            held_item: None,
            pending_growth: 0,
            move_cooldown: rules.normal_interval_ticks,
            boost_remaining: 0,
            boost_cooldown: 0,
        }
    }

    pub fn head(&self) -> Position {
        self.body[0]
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }

    /// 直前の進行方向と真逆の入力は無視する
    pub fn turn(&mut self, dir: Direction) {
        if dir != self.last_moved_dir.opposite() {
            self.dir = dir;
        }
    }

    pub fn boost_ready(&self) -> bool {
        self.boost_remaining == 0 && self.boost_cooldown == 0
    }

    pub fn try_boost(&mut self, rules: &Rules) {
        if self.boost_ready() {
            self.boost_remaining = rules.boost_duration_ticks;
        }
    }

    /// 次のティックで移動するか (= AI が行動を決めるタイミングか)
    pub fn moves_next_tick(&self) -> bool {
        self.alive && self.move_cooldown == 1
    }

    /// ブーストと移動のタイマーを1ティック進め、このティックで移動するなら true を返す
    pub fn advance_timers(&mut self, rules: &Rules) -> bool {
        if self.boost_remaining > 0 {
            self.boost_remaining -= 1;
            if self.boost_remaining == 0 {
                self.boost_cooldown = rules.boost_cooldown_ticks;
            }
        } else if self.boost_cooldown > 0 {
            self.boost_cooldown -= 1;
        }

        self.move_cooldown -= 1;
        if self.move_cooldown > 0 {
            return false;
        }
        self.move_cooldown = if self.boost_remaining > 0 {
            rules.boost_interval_ticks
        } else {
            rules.normal_interval_ticks
        };
        true
    }

    pub fn next_head(&self) -> Position {
        self.head().step(self.dir)
    }

    /// `pos` に進むと自分の体にぶつかるか。尻尾はこの移動で退くので伸びている途中でなければ除外する
    pub fn collides_with_self(&self, pos: Position) -> bool {
        let tail_moves = self.pending_growth == 0;
        let checked = if tail_moves {
            self.body.len() - 1
        } else {
            self.body.len()
        };
        self.body.iter().take(checked).any(|&p| p == pos)
    }

    pub fn advance(&mut self, next: Position) {
        self.body.push_front(next);
        if self.pending_growth > 0 {
            self.pending_growth -= 1;
        } else {
            self.body.pop_back();
        }
        self.last_moved_dir = self.dir;
    }

    /// 正なら以降の移動で伸び、負なら伸びる予定を打ち消したうえで尻尾を即座に削る (最短1マス)
    pub fn apply_growth(&mut self, grow: i32) {
        if grow >= 0 {
            self.pending_growth += grow as u32;
            return;
        }
        let mut shrink = grow.unsigned_abs();
        let cancelled = shrink.min(self.pending_growth);
        self.pending_growth -= cancelled;
        shrink -= cancelled;
        for _ in 0..shrink {
            if self.body.len() <= 1 {
                break;
            }
            self.body.pop_back();
        }
    }
}
