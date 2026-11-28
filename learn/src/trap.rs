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
/// 推論時の安全装置 (`safety`) が 1 つの手を調べる局面の数の上限。
/// Web 版でも毎回の判断で同じ探索をするので、死に方の分析より小さく抑える
pub const SAFETY_BUDGET: usize = 2_000;

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
    let mut search = Search::new(snapshot, width, height, SEARCH_BUDGET);
    let head = search.cell(snapshot.body[0]);
    Escape::from(search.run(head, snapshot.last_moved_dir, 0))
}

/// 最初の一手を `first` に進んだうえで、今の体がすべて退くまで動き続けられる経路があるか。
/// 最初の一手で衝突するなら Trapped。調べる局面は `budget` までで、越えたら Unknown
pub fn escape_by(
    snapshot: &Snapshot,
    width: i32,
    height: i32,
    first: Direction,
    budget: usize,
) -> Escape {
    let mut search = Search::new(snapshot, width, height, budget);
    let head = search.cell(snapshot.body[0]);
    if first == snapshot.last_moved_dir.opposite() {
        return Escape::Trapped;
    }
    let Some(next) = search
        .neighbor(head, first)
        .filter(|&n| search.can_enter(n, 1))
    else {
        return Escape::Trapped;
    };
    if search.traces_body(next) {
        return Escape::Open;
    }
    search.on_path[next] = true;
    Escape::from(search.run(next, first, 1))
}

impl Escape {
    fn from(found: Option<bool>) -> Self {
        match found {
            Some(true) => Escape::Open,
            Some(false) => Escape::Trapped,
            None => Escape::Unknown,
        }
    }
}

/// 体でないマスの `body_index`
const NOT_BODY: usize = usize::MAX;

/// 経路の探索。マスは y * 幅 + x の番号で扱う
struct Search {
    width: i32,
    height: i32,
    goal: u32,
    /// free_at[i] は、マス i に入れるようになる移動の番号 (1 手目が 1)。0 は最初から空き
    free_at: Vec<u32>,
    /// 今の体のマスの番号 (先頭が頭)
    body: Vec<usize>,
    /// body_index[i] は、マス i が頭から何番目の体か。体でなければ NOT_BODY
    body_index: Vec<usize>,
    /// 探索中の経路で通ったマス。経路は新しい体になり、goal 手までは退かない
    on_path: Vec<bool>,
    /// 行けるマスを数えるときに、数えたマスへ付ける印。数えるたびに `stamp` を 1 増やすので、消さずに使い回せる
    seen: Vec<u32>,
    stamp: u32,
    stack: Vec<usize>,
    nodes: usize,
    budget: usize,
}

impl Search {
    fn new(snapshot: &Snapshot, width: i32, height: i32, budget: usize) -> Self {
        let cells = (width * height) as usize;
        let len = snapshot.body.len() as u32;
        let growth = snapshot.pending_growth;
        let cell = |p: Position| (p.y * width + p.x) as usize;
        let mut free_at = vec![0u32; cells];
        for &pos in &snapshot.obstacles {
            free_at[cell(pos)] = u32::MAX;
        }
        // 頭から i 番目の体は、伸びる予定の分だけ尻尾が止まったあと、尻尾から順に退く
        let body: Vec<usize> = snapshot.body.iter().map(|&p| cell(p)).collect();
        let mut body_index = vec![NOT_BODY; cells];
        for (i, &c) in body.iter().enumerate() {
            free_at[c] = growth + len - i as u32;
            body_index[c] = i;
        }
        let mut goal = growth + len;
        // 伸びている途中で盤面が埋まれば勝ちなので、そこまで動ければよい
        let fill_at = (cells as u32)
            .saturating_sub(snapshot.obstacles.len() as u32)
            .saturating_sub(len);
        if fill_at <= growth {
            goal = goal.min(fill_at);
        }
        Self {
            width,
            height,
            goal,
            free_at,
            body,
            body_index,
            on_path: vec![false; cells],
            seen: vec![0; cells],
            stamp: 0,
            stack: Vec::with_capacity(cells),
            nodes: 0,
            budget,
        }
    }

    fn cell(&self, pos: Position) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    /// マス `i` の `dir` 側の隣。盤の外なら None
    fn neighbor(&self, i: usize, dir: Direction) -> Option<usize> {
        let (x, y) = ((i as i32) % self.width, (i as i32) / self.width);
        let (dx, dy) = dir.delta();
        let (nx, ny) = (x + dx, y + dy);
        (nx >= 0 && nx < self.width && ny >= 0 && ny < self.height)
            .then(|| (ny * self.width + nx) as usize)
    }

    /// `moves` 手目の移動でマス `i` に入れるか
    fn can_enter(&self, i: usize, moves: u32) -> bool {
        !self.on_path[i] && self.free_at[i] <= moves
    }

    /// 入ったマス `i` が今の体のマスなら、そこから体を頭までなぞれば goal 手まで動き続けられるか。
    /// 頭から b 番目の体のマスに、そのマスが空いた手以降に入れたなら、b - 1 番目、b - 2 番目 … も
    /// ちょうど空いた手に入れる。なぞるマスがまだ経路で通っていなければ、頭のマスに着く手は goal 以上になる
    fn traces_body(&self, i: usize) -> bool {
        let b = self.body_index[i];
        b != NOT_BODY && self.body[..b].iter().all(|&c| !self.on_path[c])
    }

    /// `moves` 手動いて頭がマス `head` にあるところから、goal 手まで動き続けられるか。上限を越えたら None
    fn run(&mut self, head: usize, last_dir: Direction, moves: u32) -> Option<bool> {
        if moves >= self.goal {
            return Some(true);
        }
        self.nodes += 1;
        if self.nodes > self.budget {
            return None;
        }
        // 経路は同じマスを通れないので、行けるマスが残りの手数より少なければ届かない
        if !self.has_room(head, (self.goal - moves) as usize) {
            return Some(false);
        }
        for dir in DIRECTIONS {
            // 直前に進んだ方向の真逆は無視される
            if dir == last_dir.opposite() {
                continue;
            }
            let Some(next) = self.neighbor(head, dir) else {
                continue;
            };
            if !self.can_enter(next, moves + 1) {
                continue;
            }
            if self.traces_body(next) {
                return Some(true);
            }
            self.on_path[next] = true;
            let found = self.run(next, dir, moves + 1);
            self.on_path[next] = false;
            if found != Some(false) {
                return found;
            }
        }
        Some(false)
    }

    /// `head` から、経路とお邪魔ブロックを避けて行けるマス (goal 手までに空かない体のマスも除く) が `need` 以上あるか。
    /// `need` に届いた時点で数えるのをやめる
    fn has_room(&mut self, head: usize, need: usize) -> bool {
        if need == 0 {
            return true;
        }
        self.stamp += 1;
        self.stack.clear();
        self.stack.push(head);
        let mut count = 0;
        while let Some(i) = self.stack.pop() {
            for dir in DIRECTIONS {
                let Some(next) = self.neighbor(i, dir) else {
                    continue;
                };
                if self.seen[next] == self.stamp
                    || self.on_path[next]
                    || self.free_at[next] > self.goal
                {
                    continue;
                }
                self.seen[next] = self.stamp;
                count += 1;
                if count >= need {
                    return true;
                }
                self.stack.push(next);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::env::game::GameEnv;
    use crate::env::snake::Snake;
    use rand::Rng;
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
    fn escape_by_checks_the_given_first_move() {
        // 上に進むと 2 マスの袋小路 (長さ 5 の体が退くまで動けない)。右に進めば広い
        let s = snapshot(
            &[(0, 2), (0, 3), (0, 4), (1, 4), (2, 4)],
            Direction::Up,
            0,
            &[(1, 1), (1, 0)],
        );
        assert_eq!(escape_by(&s, 5, 5, Direction::Up, 1000), Escape::Trapped);
        assert_eq!(escape_by(&s, 5, 5, Direction::Right, 1000), Escape::Open);
        // 壁・逆走は最初の一手で衝突する
        assert_eq!(escape_by(&s, 5, 5, Direction::Left, 1000), Escape::Trapped);
        assert_eq!(escape_by(&s, 5, 5, Direction::Down, 1000), Escape::Trapped);
        // どれかの手で助かれば、escape も助かると判定する
        assert_eq!(escape(&s, 5, 5), Escape::Open);
        // 上限を越えたら判定できず
        assert_eq!(escape_by(&s, 5, 5, Direction::Right, 1), Escape::Unknown);
    }

    /// 枝刈りも近道もしない総当たりで、goal 手まで動き続けられるか
    fn brute_force(s: &Snapshot, width: i32, height: i32, first: Option<Direction>) -> bool {
        let search = Search::new(s, width, height, usize::MAX);
        fn go(
            search: &mut Search,
            head: usize,
            last: Direction,
            moves: u32,
            only: Option<Direction>,
        ) -> bool {
            if moves >= search.goal {
                return true;
            }
            for dir in DIRECTIONS {
                if dir == last.opposite() || only.is_some_and(|d| d != dir) {
                    continue;
                }
                let Some(next) = search.neighbor(head, dir) else {
                    continue;
                };
                if !search.can_enter(next, moves + 1) {
                    continue;
                }
                search.on_path[next] = true;
                let found = go(search, next, dir, moves + 1, None);
                search.on_path[next] = false;
                if found {
                    return true;
                }
            }
            false
        }
        let mut search = search;
        let head = search.cell(s.body[0]);
        go(&mut search, head, s.last_moved_dir, 0, first)
    }

    /// 乱数で歩かせたヘビと、空いたマスに置いたお邪魔ブロック
    fn random_snapshot(rng: &mut impl rand::Rng, width: i32, height: i32) -> Option<Snapshot> {
        let len = rng.gen_range(2..=(width * height - 2) as usize);
        let mut walk = vec![pos(rng.gen_range(0..width), rng.gen_range(0..height))];
        let mut dirs = Vec::new();
        while walk.len() < len {
            let last = *walk.last().unwrap();
            let options: Vec<Direction> = DIRECTIONS
                .into_iter()
                .filter(|&d| {
                    let n = last.step(d);
                    n.x >= 0 && n.x < width && n.y >= 0 && n.y < height && !walk.contains(&n)
                })
                .collect();
            if options.is_empty() {
                break;
            }
            let d = options[rng.gen_range(0..options.len())];
            walk.push(last.step(d));
            dirs.push(d);
        }
        let last_moved_dir = *dirs.last()?;
        walk.reverse();
        let mut obstacles = Vec::new();
        for _ in 0..rng.gen_range(0..3) {
            let p = pos(rng.gen_range(0..width), rng.gen_range(0..height));
            if !walk.contains(&p) && !obstacles.contains(&p) {
                obstacles.push(p);
            }
        }
        // 盤面がもう埋まっている局面は、試合が終わっているので調べない
        if walk.len() + obstacles.len() >= (width * height) as usize {
            return None;
        }
        Some(Snapshot {
            body: walk,
            pending_growth: rng.gen_range(0..3),
            last_moved_dir,
            obstacles,
        })
    }

    #[test]
    fn matches_brute_force_on_small_boards() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let (mut open, mut trapped) = (0, 0);
        for _ in 0..3000 {
            let (w, h) = (rng.gen_range(3..=5), rng.gen_range(3..=5));
            let Some(s) = random_snapshot(&mut rng, w, h) else {
                continue;
            };
            let expected = brute_force(&s, w, h, None);
            let actual = escape(&s, w, h);
            assert_eq!(actual == Escape::Open, expected, "{s:?} {w}x{h}");
            assert_ne!(actual, Escape::Unknown);
            if expected {
                open += 1;
            } else {
                trapped += 1;
            }
            for dir in DIRECTIONS {
                let expected = brute_force(&s, w, h, Some(dir));
                let actual = escape_by(&s, w, h, dir, usize::MAX);
                assert_eq!(actual == Escape::Open, expected, "{s:?} {w}x{h} {dir:?}");
            }
        }
        // 助かる局面と詰みの局面の両方を十分に調べている
        assert!(open > 300 && trapped > 300, "{open} {trapped}");
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
