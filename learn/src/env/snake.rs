#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Snake {
    pub body: Vec<Position>,
    pub dir: Direction,
    pub is_alive: bool,
}

impl Snake {
    pub fn new(start_x: i32, start_y: i32) -> Self {
        Self {
            body: vec![Position { x: start_x, y: start_y }],
            dir: Direction::Right,
            is_alive: true,
        }
    }

    pub fn move_forward(&mut self, grow: bool) {
        if !self.is_alive { return; }
        let head = &self.body[0];
        let new_head = match self.dir {
            Direction::Up => Position { x: head.x, y: head.y - 1 },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x - 1, y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };
        self.body.insert(0, new_head);
        if !grow {
            self.body.pop();
        }
    }
}