use super::snake::*;
use crate::config::GameConfig;
use rand::Rng;

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
    pub item_type: ItemType,
}

#[derive(Debug, Clone)]
pub struct PendingSpawn {
    pub item_type: ItemType,
    pub timer: u16,
}

#[derive(Debug, Clone)]
pub struct PendingJamBlock {
    pub timer: u16,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub snake: Snake,
    pub items: Vec<Item>,
    pub obstacle_blocks: Vec<Position>,
    pub pending_spawns: Vec<PendingSpawn>,
    pub pending_jams: Vec<PendingJamBlock>,
}

impl Field {
    pub fn new(start_x: i32, start_y: i32, config: &GameConfig) -> Self {
        let mut field = Self {
            snake: Snake::new(start_x, start_y, &config.snake),
            items: Vec::new(),
            obstacle_blocks: Vec::new(),
            pending_spawns: Vec::new(),
            pending_jams: Vec::new(),
        };

        for _ in 0..config.items.max_apples {
            field.spawn_item_immediately(ItemType::NormalApple, config);
        }
        for _ in 0..config.items.max_poison {
            field.spawn_item_immediately(ItemType::PoisonApple, config);
        }
        field.spawn_item_immediately(ItemType::GoldApple, config);
        field.spawn_item_immediately(ItemType::BlockClear, config);
        field.spawn_item_immediately(ItemType::BlockJam, config);

        field
    }

    pub fn spawn_item_immediately(&mut self, item_type: ItemType, config: &GameConfig) {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let x = rng.gen_range(0..config.grid.width);
            let y = rng.gen_range(0..config.grid.height);
            let pos = Position { x, y };

            if !self.snake.body.contains(&pos)
                && !self.obstacle_blocks.contains(&pos)
                && !self.items.iter().any(|i| i.pos == pos)
            {
                self.items.push(Item { pos, item_type });
                break;
            }
        }
    }

    pub fn spawn_jam_block(&mut self, config: &GameConfig) {
        let mut rng = rand::thread_rng();
        let head = *self.snake.body.front().unwrap();

        let next_head = match self.snake.dir {
            Direction::Up => Position { x: head.x, y: head.y - 1 },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x - 1, y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };

        for _ in 0..100 {
            let x = rng.gen_range(0..config.grid.width);
            let y = rng.gen_range(0..config.grid.height);
            let pos = Position { x, y };

            if pos != head
                && pos != next_head
                && !self.snake.body.contains(&pos)
                && !self.obstacle_blocks.contains(&pos)
                && !self.items.iter().any(|i| i.pos == pos)
            {
                self.obstacle_blocks.push(pos);
                break;
            }
        }
    }
}

pub struct GameEnv {
    pub player_field: Field,
    pub ai_field: Field,
    pub time_remaining_steps: u16,
    pub config: GameConfig,
}

impl GameEnv {
    pub fn new(config: GameConfig) -> Self {
        let steps = GameConfig::sec_to_steps(config.grid.time_limit_seconds);
        let p_x = config.grid.width / 2;
        let p_y = config.grid.height / 2;

        Self {
            player_field: Field::new(p_x, p_y, &config),
            ai_field: Field::new(p_x, p_y, &config),
            time_remaining_steps: steps,
            config,
        }
    }

    pub fn step(
        &mut self,
        player_dir: Direction,
        use_player_item: bool,
        ai_dir: Direction,
        use_ai_item: bool,
    ) {
        if self.time_remaining_steps == 0 {
            return;
        }
        self.time_remaining_steps -= 1;

        if use_player_item {
            Self::use_held_item(&mut self.player_field, &mut self.ai_field, &self.config);
        }
        if use_ai_item {
            Self::use_held_item(&mut self.ai_field, &mut self.player_field, &self.config);
        }

        self.player_field.snake.set_direction(player_dir);
        self.ai_field.snake.set_direction(ai_dir);

        let cfg = self.config.clone();
        self.tick_field(true, &cfg);
        self.tick_field(false, &cfg);
    }

    fn use_held_item(user_field: &mut Field, target_field: &mut Field, config: &GameConfig) {
        if let Some(item) = user_field.snake.held_item.take() {
            match item {
                HeldItem::BlockClear => {
                    user_field.obstacle_blocks.clear();
                }
                HeldItem::BlockJam => {
                    let jam_steps = GameConfig::sec_to_steps(config.items.jam_block_delay_sec);
                    target_field.pending_jams.push(PendingJamBlock { timer: jam_steps });
                }
            }
        }
    }

    fn tick_field(&mut self, is_player: bool, config: &GameConfig) {
        let field = if is_player { &mut self.player_field } else { &mut self.ai_field };

        let mut ready_spawns = Vec::new();
        field.pending_spawns.retain_mut(|p| {
            if p.timer > 0 { p.timer -= 1; }
            if p.timer == 0 {
                ready_spawns.push(p.item_type);
                false
            } else { true }
        });
        for item_type in ready_spawns {
            field.spawn_item_immediately(item_type, config);
        }

        let mut ready_jams = 0;
        field.pending_jams.retain_mut(|j| {
            if j.timer > 0 { j.timer -= 1; }
            if j.timer == 0 {
                ready_jams += 1;
                false
            } else { true }
        });
        for _ in 0..ready_jams {
            field.spawn_jam_block(config);
        }

        if field.snake.tick_step() {
            Self::move_and_check_collisions(field, config);
        }
    }

    fn move_and_check_collisions(field: &mut Field, config: &GameConfig) {
        let head = *field.snake.body.front().unwrap();
        let next_head = match field.snake.dir {
            Direction::Up => Position { x: head.x, y: head.y - 1 },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x - 1, y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };

        if field.obstacle_blocks.contains(&next_head) {
            field.snake.is_alive = false;
            return;
        }

        let mut grow_amount = 0;
        let mut hit_idx = None;

        let apple_steps = GameConfig::sec_to_steps(config.items.apple_respawn_delay_sec);
        let special_steps = GameConfig::sec_to_steps(config.items.special_respawn_delay_sec);

        for (idx, item) in field.items.iter().enumerate() {
            if item.pos == next_head {
                hit_idx = Some(idx);
                match item.item_type {
                    ItemType::NormalApple => {
                        grow_amount = config.items.normal_apple_grow;
                        field.snake.score += config.items.normal_apple_score;
                        field.pending_spawns.push(PendingSpawn { item_type: ItemType::NormalApple, timer: apple_steps });
                    }
                    ItemType::GoldApple => {
                        grow_amount = config.items.gold_apple_grow;
                        field.snake.score += config.items.gold_apple_score;
                        field.pending_spawns.push(PendingSpawn { item_type: ItemType::GoldApple, timer: special_steps });
                    }
                    ItemType::PoisonApple => {
                        grow_amount = config.items.poison_apple_grow;
                        field.snake.score = (field.snake.score + config.items.poison_apple_score).max(0);
                        field.pending_spawns.push(PendingSpawn { item_type: ItemType::PoisonApple, timer: apple_steps });
                    }
                    ItemType::BlockClear => {
                        field.snake.held_item = Some(HeldItem::BlockClear);
                        field.pending_spawns.push(PendingSpawn { item_type: ItemType::BlockClear, timer: special_steps });
                    }
                    ItemType::BlockJam => {
                        field.snake.held_item = Some(HeldItem::BlockJam);
                        field.pending_spawns.push(PendingSpawn { item_type: ItemType::BlockJam, timer: special_steps });
                    }
                }
                break;
            }
        }

        if let Some(idx) = hit_idx {
            field.items.remove(idx);
        }

        field.snake.move_forward(grow_amount, config.grid.width, config.grid.height);
    }
}