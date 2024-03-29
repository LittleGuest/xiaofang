use alloc::collections::LinkedList;
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use crate::{App, Direction, Gd, Position};

type Food = Position;

#[derive(Debug)]
pub struct SnakeGame {
    width: i8,
    height: i8,
    snake: Snake,
    food: Food,
    /// ms
    waiting_time: u32,
    score: u8,
    game_over: bool,
}

impl SnakeGame {
    pub fn new() -> Self {
        let width = 8;
        let height = 8;
        Self {
            width,
            height,
            snake: Snake::new(Position::new(5, 5)),
            food: Food::random(width, height),
            waiting_time: 600,
            score: 0,
            game_over: false,
        }
    }

    pub fn run<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        app.gd = Gd::default();

        loop {
            if self.game_over {
                // TODO 历史最高分动画,音乐
                app.ledc.draw_score(self.score);
                app.delay.delay_ms(3000_u32);
                break;
            }
            app.gravity_direction();
            self.r#move(&app.gd);
            // TODO 移动音效,得分音效和画面效果,死亡音效
            self.draw(app);

            app.delay.delay_ms(self.waiting_time);
        }
    }

    fn r#move(&mut self, gd: &Gd) {
        match gd {
            Gd::None => {}
            Gd::Up => self.snake.set_direction(Direction::Up),
            Gd::Right => self.snake.set_direction(Direction::Right),
            Gd::Down => self.snake.set_direction(Direction::Down),
            Gd::Left => self.snake.set_direction(Direction::Left),
        };

        let next_head = self.snake.next_head_pos();
        if self.food.eq(&next_head) {
            self.snake.grow(self.food);
            self.calc_score();
            self.create_food();
        } else if self.outside(next_head) || self.snake.overlapping() {
            self.game_over = true;
        } else {
            self.snake.r#move();
        }
    }

    fn calc_score(&mut self) {
        self.score += 1;
    }

    fn outside(&self, next_head: Position) -> bool {
        next_head.x < 0
            || next_head.y < 0
            || next_head.x >= self.width
            || next_head.y >= self.height
    }

    fn create_food(&mut self) {
        self.food = loop {
            let food = Food::random(self.width, self.height);
            if self.snake.body.iter().any(|s| s.eq(&food)) {
                continue;
            } else {
                break food;
            }
        };
    }

    pub fn draw<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        let ledc = &mut app.ledc;
        ledc.clear();
        // ledc.clear_work();
        let mut tmp = self.snake.as_bytes();
        for (i, s) in tmp.iter_mut().enumerate() {
            if i == self.food.y as usize {
                *s |= 1 << (7 - self.food.x);
            }
        }
        ledc.write_bytes(tmp);
    }
}

#[derive(Debug)]
struct Snake {
    direction: Direction,
    head: Position,
    body: LinkedList<Position>,
}

impl Snake {
    fn new(head: Position) -> Self {
        let mut body = LinkedList::new();
        body.push_back(head);
        let (x, y) = (head.x, head.y);
        body.push_back(Position { x, y: y + 1 });

        Self {
            direction: Direction::Up,
            head,
            body,
        }
    }

    fn set_direction(&mut self, dir: Direction) {
        if dir == self.direction.opposite() {
            return;
        }
        self.direction = dir;
    }

    fn grow(&mut self, food: Food) {
        self.head = food;
        self.body.push_front(food);
    }

    fn r#move(&mut self) {
        let next_head = self.next_head_pos();
        self.body.push_front(next_head);
        self.body.pop_back();
        self.head = next_head;
    }

    fn next_head_pos(&self) -> Position {
        self.head.next(self.direction)
    }

    fn overlapping(&self) -> bool {
        self.body.iter().skip(1).any(|pos| pos.eq(&self.head))
    }

    fn as_bytes(&self) -> [u8; 8] {
        let mut bs = [0; 8];
        for y in 0..8 {
            let mut tmp = 0;
            for x in 0..8 {
                if self.body.iter().any(|p| p.x == x && p.y == y) {
                    tmp |= 1 << (7 - x);
                }
            }
            bs[y as usize] = tmp;
        }
        bs
    }
}
