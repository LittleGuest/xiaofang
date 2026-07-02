#![doc = include_str!("../../../rfcs/008_dodge_cube.md")]

use crate::{buzzer, rng, Ad, App, CubeRng, Point};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use embassy_time::Timer;
use embedded_graphics_core::{
    pixelcolor::Rgb888,
    prelude::WebColors,
    Pixel,
};

/// 躲避方块
#[derive(Debug)]
pub struct DodgeCubeGame {
    width: i32,
    height: i32,
    player_pos: Point,
    /// 障碍物行，从上(y=0)到下
    rows: VecDeque<CubeRow>,
    /// 帧间隔 ms
    waiting_time: u64,
    /// 得分
    score: u8,
    /// 最高分
    pub highest: u8,
    game_over: bool,
}

/// 一行障碍物，8列，true=障碍物
#[derive(Debug, Clone)]
struct CubeRow {
    data: [bool; 8],
}

impl CubeRow {
    fn empty() -> Self {
        Self { data: [false; 8] }
    }

    /// 随机生成一行障碍物，保留 gap_count 个连续空位
    fn random(gap_count: usize) -> Self {
        let mut data = [true; 8];
        let start = unsafe {
            CubeRng(rng().random() as u64).random_range(0..=(8 - gap_count))
        };
        for i in start..start + gap_count {
            if i < 8 {
                data[i] = false;
            }
        }
        Self { data }
    }
}

impl Default for DodgeCubeGame {
    fn default() -> Self {
        Self::new()
    }
}

impl DodgeCubeGame {
    pub fn new() -> Self {
        let mut rows = VecDeque::with_capacity(8);
        // 初始化空行
        for _ in 0..8 {
            rows.push_back(CubeRow::empty());
        }

        Self {
            width: 8,
            height: 8,
            player_pos: Point::new(3, 7),
            rows,
            waiting_time: 600,
            score: 0,
            highest: 0,
            game_over: false,
        }
    }

    pub async fn run(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.ad = Ad::default();

        loop {
            Timer::after_millis(self.waiting_time).await;

            if self.game_over {
                unsafe { buzzer().dodge_cube_die().await };
                app.ledc.draw_score(self.score);
                Timer::after_millis(1500).await;
                if self.score > self.highest {
                    self.highest = self.score;
                    app.face.break_record_animate(&mut app.ledc).await;
                }
                Timer::after_millis(500).await;
                break;
            }

            // 1. 障碍物行向下滚动
            self.rows.pop_front();

            // 2. 根据难度生成新行（从顶部进入）
            let gap = if self.score < 10 { 2 } else { 1 };
            let should_generate = unsafe { CubeRng(rng().random() as u64).random_range(1..=3) };
            if should_generate > 1 {
                self.rows.push_back(CubeRow::random(gap));
            } else {
                self.rows.push_back(CubeRow::empty());
            }

            // 3. 读取加速度方向并移动玩家
            app.acc_direction();
            app.check_pause().await;
            self.r#move(&app.ad);
            unsafe { buzzer().dodge_cube_move().await };

            // 4. 碰撞检测：玩家位置是否有障碍物
            let py = self.player_pos.y as usize;
            let px = self.player_pos.x as usize;
            if py < self.rows.len() && self.rows[py].data[px] {
                self.game_over = true;
                continue;
            }

            // 5. 得分
            self.calc_score();
            if self.score % 10 == 0 {
                unsafe { buzzer().dodge_cube_score().await };
            }

            // 6. 难度递增
            if self.waiting_time > 300 {
                self.waiting_time -= 2;
            }

            self.draw(app);
        }
    }

    fn r#move(&mut self, gd: &Ad) {
        match gd {
            Ad::Front => {
                if self.player_pos.y > 0 {
                    self.player_pos.y -= 1;
                }
            }
            Ad::Back => {
                if self.player_pos.y < self.height - 1 {
                    self.player_pos.y += 1;
                }
            }
            Ad::Left => {
                if self.player_pos.x > 0 {
                    self.player_pos.x -= 1;
                }
            }
            Ad::Right => {
                if self.player_pos.x < self.width - 1 {
                    self.player_pos.x += 1;
                }
            }
            _ => {}
        }
    }

    fn calc_score(&mut self) {
        self.score += 1;
    }

    fn draw(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        let mut pixels = Vec::new();

        // 绘制障碍物行
        for (y, row) in self.rows.iter().enumerate() {
            for (x, &occupied) in row.data.iter().enumerate() {
                if occupied {
                    pixels.push(Pixel(
                        (x as i32, y as i32).into(),
                        Rgb888::CSS_CYAN,
                    ));
                }
            }
        }

        // 绘制玩家（红色）
        let pp = self.player_pos;
        pixels.push(Pixel((pp.x, pp.y).into(), Rgb888::CSS_RED));

        app.ledc.write_pixels(pixels);
    }
}
