use super::rules::Rules;
use super::snake::{HeldItem, Position, Snake};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    NormalApple,
    GoldApple,
    PoisonApple,
    BlockClear,
    BlockJam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub pos: Position,
    pub kind: ItemType,
}

#[derive(Debug, Clone)]
pub struct PendingSpawn {
    pub kind: ItemType,
    pub ticks: u32,
}

/// プレイヤー1人分の盤面
#[derive(Debug, Clone)]
pub struct Field {
    pub snake: Snake,
    pub items: Vec<Item>,
    pub obstacles: Vec<Position>,
    /// 食べられたアイテムの再出現待ち
    pub pending_spawns: Vec<PendingSpawn>,
    /// 相手から送られたお邪魔ブロックの出現待ち (残りティック数)
    pub incoming_jams: Vec<u32>,
    /// 直前のティックに取ったアイテム。ゲームの進行には使わず、学習の報酬に使う
    pub eaten: Option<ItemType>,
}

impl Field {
    pub fn new(rules: &Rules, rng: &mut StdRng) -> Self {
        let start = Position {
            x: rules.width / 2,
            y: rules.height / 2,
        };
        let mut field = Self {
            snake: Snake::new(start, rules),
            items: Vec::new(),
            obstacles: Vec::new(),
            pending_spawns: Vec::new(),
            incoming_jams: Vec::new(),
            eaten: None,
        };

        let initial = std::iter::repeat_n(ItemType::NormalApple, rules.normal_apples)
            .chain(std::iter::repeat_n(
                ItemType::PoisonApple,
                rules.poison_apples,
            ))
            .chain([
                ItemType::GoldApple,
                ItemType::BlockClear,
                ItemType::BlockJam,
            ]);
        for kind in initial {
            field.spawn_item(kind, rules, rng);
        }
        field
    }

    /// 生きているヘビが、お邪魔ブロック以外のすべてのマスを埋めているか
    pub fn is_filled(&self, rules: &Rules) -> bool {
        self.snake.alive && self.snake.len() + self.obstacles.len() == rules.cell_count()
    }

    pub fn in_bounds(pos: Position, rules: &Rules) -> bool {
        pos.x >= 0 && pos.x < rules.width && pos.y >= 0 && pos.y < rules.height
    }

    /// ヘビ・アイテム・ブロックのどれも無いマスを列挙する
    fn empty_cells(&self, rules: &Rules) -> Vec<Position> {
        let w = rules.width;
        let mut occupied = vec![false; rules.cell_count()];
        let occupied_positions = self
            .snake
            .body
            .iter()
            .chain(self.items.iter().map(|i| &i.pos))
            .chain(self.obstacles.iter());
        for p in occupied_positions {
            occupied[(p.y * w + p.x) as usize] = true;
        }
        (0..rules.cell_count() as i32)
            .filter(|&i| !occupied[i as usize])
            .map(|i| Position { x: i % w, y: i / w })
            .collect()
    }

    /// 空きマスにアイテムを出す。空きが無ければ false
    pub fn spawn_item(&mut self, kind: ItemType, rules: &Rules, rng: &mut StdRng) -> bool {
        match self.empty_cells(rules).choose(rng) {
            Some(&pos) => {
                self.items.push(Item { pos, kind });
                true
            }
            None => false,
        }
    }

    /// 頭から安全距離より離れた空きマスにお邪魔ブロックを出す。置ける場所が無ければ false
    pub fn spawn_obstacle(&mut self, rules: &Rules, rng: &mut StdRng) -> bool {
        let head = self.snake.head();
        let candidates: Vec<Position> = self
            .empty_cells(rules)
            .into_iter()
            .filter(|p| p.manhattan(head) > rules.jam_safe_distance)
            .collect();
        match candidates.choose(rng) {
            Some(&pos) => {
                self.obstacles.push(pos);
                true
            }
            None => false,
        }
    }

    /// 出現待ちのタイマーを進め、時間になったものを出現させる
    pub fn process_pending(&mut self, rules: &Rules, rng: &mut StdRng) {
        let mut due_items = Vec::new();
        self.pending_spawns.retain_mut(|p| {
            p.ticks = p.ticks.saturating_sub(1);
            if p.ticks == 0 {
                due_items.push(p.kind);
            }
            p.ticks > 0
        });
        for kind in due_items {
            if !self.spawn_item(kind, rules, rng) {
                // 盤面が埋まっていたら次のティックで再挑戦
                self.pending_spawns.push(PendingSpawn { kind, ticks: 1 });
            }
        }

        let mut due_jams = 0;
        self.incoming_jams.retain_mut(|t| {
            *t = t.saturating_sub(1);
            if *t == 0 {
                due_jams += 1;
            }
            *t > 0
        });
        for _ in 0..due_jams {
            // 置き場所が無いお邪魔ブロックは消滅する
            self.spawn_obstacle(rules, rng);
        }
    }

    /// ヘビのタイマーを進め、移動するティックなら移動・衝突判定・アイテム取得まで行う
    pub fn update_snake(&mut self, rules: &Rules) {
        self.eaten = None;
        if !self.snake.alive || !self.snake.advance_timers(rules) {
            return;
        }

        let next = self.snake.next_head();
        if !Self::in_bounds(next, rules)
            || self.obstacles.contains(&next)
            || self.snake.collides_with_self(next)
        {
            self.snake.alive = false;
            return;
        }
        self.snake.advance(next);

        if let Some(idx) = self.items.iter().position(|i| i.pos == next) {
            let item = self.items.remove(idx);
            self.consume(item.kind, rules);
            self.eaten = Some(item.kind);
        }
    }

    fn consume(&mut self, kind: ItemType, rules: &Rules) {
        let snake = &mut self.snake;
        let apple = match kind {
            ItemType::NormalApple => Some(rules.normal_apple),
            ItemType::GoldApple => Some(rules.gold_apple),
            ItemType::PoisonApple => Some(rules.poison_apple),
            ItemType::BlockClear => {
                snake.held_item = Some(HeldItem::BlockClear);
                None
            }
            ItemType::BlockJam => {
                snake.held_item = Some(HeldItem::BlockJam);
                None
            }
        };
        if let Some(effect) = apple {
            snake.score = (snake.score + effect.score).max(0);
            snake.apply_growth(effect.grow);
        }
        self.pending_spawns.push(PendingSpawn {
            kind,
            ticks: rules.respawn_ticks(kind),
        });
    }
}
