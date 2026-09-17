use crate::config::SnakeConfig;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldItem {
    BlockClear,
    BlockJam,
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub body: VecDeque<Position>,
    pub dir: Direction,
    pub is_alive: bool,
    pub score: i32,
    pub held_item: Option<HeldItem>,
    pub move_cooldown: u8,
    pub speed_up_duration: u16,
    pub speed_up_cooldown: u16,
    pub normal_interval_steps: u8,
    pub boost_interval_steps: u8,
    pub boost_duration_steps: u16,
    pub boost_cooldown_steps: u16,
}

impl Snake {
    pub fn new(start_x: i32, start_y: i32, config: &SnakeConfig) -> Self {
        let mut body = VecDeque::new();
        for i in 0..config.initial_length {
            body.push_back(Position {
                x: start_x,
                y: start_y + (i as i32),
            });
        }

        let normal_steps = (config.normal_speed_interval_sec / 0.1) as u8;
        let boost_steps = (config.boost_speed_interval_sec / 0.1) as u8;

        Self {
            body,
            dir: Direction::Up,
            is_alive: true,
            score: 0,
            held_item: None,
            move_cooldown: normal_steps,
            speed_up_duration: 0,
            speed_up_cooldown: 0,
            normal_interval_steps: normal_steps,
            boost_interval_steps: boost_steps,
            boost_duration_steps: (config.boost_duration_sec / 0.1) as u16,
            boost_cooldown_steps: (config.boost_cooldown_sec / 0.1) as u16,
        }
    }

    pub fn set_direction(&mut self, new_dir: Direction) {
        if new_dir != self.dir.opposite() {
            self.dir = new_dir;
        }
    }

    pub fn activate_speed_up(&mut self) {
        if self.speed_up_cooldown == 0 && self.speed_up_duration == 0 {
            self.speed_up_duration = self.boost_duration_steps;
            self.speed_up_cooldown = self.boost_cooldown_steps;
        }
    }

    pub fn tick_step(&mut self) -> bool {
        if !self.is_alive {
            return false;
        }

        if self.speed_up_duration > 0 {
            self.speed_up_duration -= 1;
        } else if self.speed_up_cooldown > 0 {
            self.speed_up_cooldown -= 1;
        }

        self.move_cooldown -= 1;
        if self.move_cooldown == 0 {
            self.move_cooldown = if self.speed_up_duration > 0 {
                self.boost_interval_steps
            } else {
                self.normal_interval_steps
            };
            return true;
        }
        false
    }

    pub fn move_forward(&mut self, grow_amount: i32, grid_w: i32, grid_h: i32) {
        if !self.is_alive {
            return;
        }

        let head = self.body.front().unwrap();
        let new_head = match self.dir {
            Direction::Up => Position { x: head.x, y: head.y - 1 },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x - 1, y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };

        if new_head.x < 0 || new_head.x >= grid_w || new_head.y < 0 || new_head.y >= grid_h {
            self.is_alive = false;
            return;
        }

        if self.body.contains(&new_head) {
            self.is_alive = false;
            return;
        }

        self.body.push_front(new_head);

        if grow_amount <= 0 {
            let pop_count = 1 + grow_amount.abs();
            for _ in 0..pop_count {
                if self.body.len() > 1 {
                    self.body.pop_back();
                }
            }
        }
    }
}