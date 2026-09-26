use super::field::Field;
use super::rules::Rules;
use super::snake::{Direction, HeldItem};
use rand::rngs::StdRng;
use rand::SeedableRng;

pub const NUM_PLAYERS: usize = 2;

/// AI の行動。並び順がモデル出力 (Q 値) の並びになる
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Boost,
    UseItem,
}

impl Action {
    pub const ALL: [Action; 6] = [
        Action::Up,
        Action::Down,
        Action::Left,
        Action::Right,
        Action::Boost,
        Action::UseItem,
    ];
    pub const COUNT: usize = Self::ALL.len();

    pub fn from_index(index: usize) -> Action {
        Self::ALL[index]
    }

    pub fn name(self) -> &'static str {
        match self {
            Action::Up => "up",
            Action::Down => "down",
            Action::Left => "left",
            Action::Right => "right",
            Action::Boost => "boost",
            Action::UseItem => "use_item",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndReason {
    /// お邪魔ブロック以外のすべてのマスをヘビで埋めた
    Filled,
    Death,
    TimeUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameResult {
    /// None は引き分け
    pub winner: Option<usize>,
    pub reason: EndReason,
}

/// 2人対戦1試合分の環境
pub struct GameEnv {
    pub fields: [Field; NUM_PLAYERS],
    pub tick: u32,
    pub result: Option<GameResult>,
    rules: Rules,
    rng: StdRng,
}

impl GameEnv {
    pub fn new(rules: Rules, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let fields = [Field::new(&rules, &mut rng), Field::new(&rules, &mut rng)];
        Self {
            fields,
            tick: 0,
            result: None,
            rules,
            rng,
        }
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    pub fn remaining_ticks(&self) -> u32 {
        self.rules.time_limit_ticks.saturating_sub(self.tick)
    }

    /// このプレイヤーが次のティックの前に行動を決める必要があるか
    pub fn needs_decision(&self, player: usize) -> bool {
        !self.is_over() && self.fields[player].snake.moves_next_tick()
    }

    /// 1ティック進める。`actions[p]` が None のプレイヤーは何も入力しない
    pub fn step(&mut self, actions: [Option<Action>; NUM_PLAYERS]) {
        if self.is_over() {
            return;
        }
        self.tick += 1;

        for field in &mut self.fields {
            field.process_pending(&self.rules, &mut self.rng);
        }
        for (player, action) in actions.into_iter().enumerate() {
            if let Some(action) = action {
                self.apply_action(player, action);
            }
        }
        for field in &mut self.fields {
            field.update_snake(&self.rules);
        }

        self.result = self.judge();
    }

    fn apply_action(&mut self, player: usize, action: Action) {
        let snake = &mut self.fields[player].snake;
        match action {
            Action::Up => snake.turn(Direction::Up),
            Action::Down => snake.turn(Direction::Down),
            Action::Left => snake.turn(Direction::Left),
            Action::Right => snake.turn(Direction::Right),
            Action::Boost => snake.try_boost(&self.rules),
            Action::UseItem => match snake.held_item.take() {
                Some(HeldItem::BlockClear) => self.fields[player].obstacles.clear(),
                Some(HeldItem::BlockJam) => {
                    let opponent = 1 - player;
                    self.fields[opponent]
                        .incoming_jams
                        .push(self.rules.jam_delay_ticks);
                }
                None => {}
            },
        }
    }

    fn judge(&self) -> Option<GameResult> {
        let filled = self.fields.each_ref().map(|f| f.is_filled(&self.rules));
        let alive = self.fields.each_ref().map(|f| f.snake.alive);
        let (winner, reason) = match (filled, alive) {
            // 盤面を埋めたら、相手の生死やスコアに関わらず勝ち
            ([true, false], _) => (Some(0), EndReason::Filled),
            ([false, true], _) => (Some(1), EndReason::Filled),
            ([true, true], _) => (self.leader_by_score(), EndReason::Filled),
            (_, [true, false]) => (Some(0), EndReason::Death),
            (_, [false, true]) => (Some(1), EndReason::Death),
            (_, [false, false]) => (self.leader_by_score(), EndReason::Death),
            _ if self.tick >= self.rules.time_limit_ticks => {
                (self.leader_by_score(), EndReason::TimeUp)
            }
            _ => return None,
        };
        Some(GameResult { winner, reason })
    }

    fn leader_by_score(&self) -> Option<usize> {
        let (a, b) = (self.fields[0].snake.score, self.fields[1].snake.score);
        match a.cmp(&b) {
            std::cmp::Ordering::Greater => Some(0),
            std::cmp::Ordering::Less => Some(1),
            std::cmp::Ordering::Equal => None,
        }
    }
}
