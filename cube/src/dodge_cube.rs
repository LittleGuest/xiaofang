#![doc = include_str!("../../rfcs/008_dodge_cube.md")]

use alloc::collections::LinkedList;
use cube_rand::CubeRng;
use embassy_time::Timer;
use embedded_graphics::{
    geometry::Point,
    pixelcolor::{Rgb888, WebColors},
    Pixel,
};

use crate::{ledc::LedControl, player::Player, App, Direction, Gd, RNG};

#[derive(Debug)]
pub struct DodgeCubeGame {
    width: i32,
    height: i32,
    player: Player,
    /// ms
    waiting_time: u64,
    /// 得分
    score: u8,
    /// 最高分
    pub highest: u8,
    game_over: bool,
}

impl Default for DodgeCubeGame {
    fn default() -> Self {
        Self::new()
    }
}

impl DodgeCubeGame {
    pub fn new() -> Self {
        let width = 8;
        let height = 8;

        Self {
            width,
            height,
            player: Player::new(Point::new(3, 7)),
            waiting_time: 600,
            score: 0,
            highest: 0,
            game_over: false,
        }
    }

    pub async fn run<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<'_, T>) {
        app.ledc.clear();
        app.gd = Gd::default();

        loop {
            Timer::after_millis(self.waiting_time).await;

            if self.game_over {
                app.ledc.draw_score(self.score);
                Timer::after_millis(1500).await;
                if self.score > self.highest {
                    self.highest = self.score;
                    app.face.break_record_animate(&mut app.ledc).await;
                }
                Timer::after_millis(500).await;
                break;
            }
            app.gravity_direction();
            self.r#move(&app.gd);
            // TODO: 移动音效,得分音效和画面效果,死亡音效
            self.draw(&mut app.ledc);
        }
    }

    fn r#move(&mut self, gd: &Gd) {}

    fn calc_score(&mut self) {
        self.score += 1;
    }

    fn outside(&self, pos: Point) -> bool {
        pos.x < 0 || pos.y < 0 || pos.x >= self.width || pos.y >= self.height
    }

    pub fn draw(&mut self, ledc: &mut LedControl<'_>) {
        ledc.clear();
        // let mut pixels = self.snake.body.clone();
        // ledc.write_pixels(pixels);
    }
}

#[derive(Debug)]
struct Cube {
    body: LinkedList<Pixel<Rgb888>>,
}

impl Cube {}
