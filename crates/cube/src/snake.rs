#![doc = include_str!("../../../rfcs/003_snake.md")]

use alloc::collections::LinkedList;

use cube_rand::CubeRng;
use embassy_time::Timer;
use embedded_graphics::{
    Pixel,
    geometry::Point,
    pixelcolor::{Rgb888, WebColors},
};
use esp_hal::rng::Rng;

use crate::{Ad, App, Direction, buzzer};

#[derive(Debug)]
pub struct SnakeGame {
    width: i32,
    height: i32,
    snake: Snake,
    food: Food,
    /// ms
    waiting_time: u64,
    /// 得分
    score: u8,
    /// 最高分
    pub highest: u8,
    game_over: bool,
    /// 吃食物闪烁状态
    eat_flash: bool,
}

impl SnakeGame {
    pub fn new(rng: &mut Rng) -> Self {
        let width = 8;
        let height = 8;

        Self {
            width,
            height,
            snake: Snake::new(Point::new(5, 5)),
            food: Food::random(width, height, rng),
            waiting_time: 600,
            score: 0,
            highest: 0,
            game_over: false,
            eat_flash: false,
        }
    }

    pub async fn run(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.ad = Ad::default();

        loop {
            Timer::after_millis(self.waiting_time).await;

            if self.game_over {
                buzzer::snake_die().await;
                app.ledc.draw_score(self.score);
                Timer::after_millis(1500).await;
                if self.score > self.highest {
                    self.highest = self.score;
                    app.face.break_record_animate(&mut app.ledc).await;
                }
                Timer::after_millis(500).await;
                break;
            }
            app.acc_direction();
            app.check_pause().await;

            self.r#move(&app.ad, &mut app.rng).await;

            self.draw(app);
        }
    }

    async fn r#move(&mut self, gd: &Ad, rng: &mut Rng) {
        match gd {
            Ad::Front => self.snake.set_direction(Direction::Up),
            Ad::Right => self.snake.set_direction(Direction::Right),
            Ad::Back => self.snake.set_direction(Direction::Down),
            Ad::Left => self.snake.set_direction(Direction::Left),
            _ => {}
        };

        let next_head = self.snake.next_head_pos();
        if self.food.pos.eq(&next_head) {
            buzzer::snake_score().await;
            self.eat_flash = true;

            self.snake.grow(self.food.clone());
            self.food.create_food(self.width, self.height, &self.snake.body, rng);
            self.calc_score();
            buzzer::snake_move().await;
        } else if self.outside(next_head) || self.snake.overlapping() {
            self.game_over = true;
        } else {
            self.snake.r#move();
            buzzer::snake_move().await;
        }
    }

    fn calc_score(&mut self) {
        self.score += 1;
    }

    fn outside(&self, next_head: Point) -> bool {
        next_head.x < 0 || next_head.y < 0 || next_head.x >= self.width || next_head.y >= self.height
    }

    pub fn draw(&mut self, app: &mut App<'_>) {
        let ledc = &mut app.ledc;

        if self.eat_flash {
            // 吃食物时全屏短暂闪绿
            ledc.clear_with_color(Rgb888::CSS_GREEN);
            self.eat_flash = false;
            return;
        }

        ledc.clear();
        // 蛇身
        let mut pixels = self.snake.body.clone();
        // 食物
        pixels.push_back(self.food.clone().into());
        ledc.write_pixels(pixels);
    }
}

/// 食物
#[derive(Debug, Clone, PartialEq, Eq)]
struct Food {
    pos: Point,
    color: Rgb888,
}

impl From<Food> for Pixel<Rgb888> {
    fn from(f: Food) -> Self {
        Pixel(f.pos, f.color)
    }
}

impl Food {
    fn random(width: i32, height: i32, rng: &mut Rng) -> Self {
        let food = {
            let x = CubeRng(rng.random() as u64).random(0, width as u32) as i32;
            let y = CubeRng(rng.random() as u64).random(0, height as u32) as i32;
            (x, y)
        };
        Self {
            pos: food.into(),
            color: Rgb888::CSS_RED,
        }
    }

    fn create_food(&self, width: i32, height: i32, snake_body: &LinkedList<Pixel<Rgb888>>, rng: &mut Rng) -> Self {
        loop {
            let food = Food::random(width, height, rng);
            if snake_body.iter().any(|s| s.0.eq(&food.pos)) {
                continue;
            } else {
                break food;
            }
        }
    }
}

/// 贪吃蛇
#[derive(Debug)]
struct Snake {
    direction: Direction,
    head: Point,
    body: LinkedList<Pixel<Rgb888>>,
}

impl Snake {
    fn new(head: Point) -> Self {
        let headp = Pixel(head, Rgb888::CSS_WHITE);
        let mut body = LinkedList::new();
        body.push_back(headp);
        body.push_back(Pixel((headp.0.x, headp.0.y + 1).into(), Rgb888::CSS_WHITE));

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

    fn grow(&mut self, mut food: Food) {
        self.head = food.pos;
        food.color = Rgb888::CSS_WHITE;
        self.body.push_front(food.into());
    }

    fn r#move(&mut self) {
        let nh = self.next_head_pos();
        self.body.push_front(Pixel(nh, Rgb888::CSS_WHITE));
        self.body.pop_back();
        self.head = nh;
    }

    fn next_head_pos(&self) -> Point {
        let mut pos = self.head;

        match self.direction {
            Direction::Up => {
                pos.y -= 1;
            }
            Direction::Right => pos.x += 1,
            Direction::Down => pos.y += 1,
            Direction::Left => pos.x -= 1,
        }
        pos
    }

    fn overlapping(&self) -> bool {
        self.body.iter().skip(1).any(|pos| pos.0.eq(&self.head))
    }
}
