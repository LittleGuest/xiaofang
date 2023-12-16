use bagua::BaGua;
use esp_idf_svc::hal::{delay::FreeRtos, i2c::I2cDriver};
use ledc::LedControl;
use log::info;
use max7219::connectors::Connector;
use mpu6050_dmp::{
    accel::{AccelF32, AccelFullScale},
    sensor::Mpu6050,
};
use rand::Rng;
use snake::SnakeGame;
use ui::Ui;

use crate::face::Face;

mod bagua;
mod battery;
mod dice;
mod face;
pub mod ledc;
mod mapping;
mod maze;
mod snake;
mod ui;

/// 左上角为坐标原点,横x,纵y
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Position {
    x: i8,
    y: i8,
}

impl Position {
    fn new(x: i8, y: i8) -> Self {
        Position { x, y }
    }
    fn r#move(&mut self, d: Direction) {
        match d {
            Direction::Up => {
                self.y -= 1;
            }
            Direction::Right => self.x += 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
        }
    }

    fn next(&self, d: Direction) -> Self {
        let mut pos = *self;

        match d {
            Direction::Up => {
                pos.y -= 1;
            }
            Direction::Right => pos.x += 1,
            Direction::Down => pos.y += 1,
            Direction::Left => pos.x -= 1,
        }

        info!("pos => {pos:?}");

        pos
    }

    fn random(x: i8, y: i8) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            x: rng.gen_range(0..x),
            y: rng.gen_range(0..y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Self::Down,
            Direction::Right => Self::Left,
            Direction::Down => Self::Up,
            Direction::Left => Self::Right,
        }
    }
}

/// 左上角为坐标原点,横x,纵y
/// 定义：* * ｜0 0 0｜0 0 0
///      无用 |x坐 标|y坐标
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
// struct PositionByte(u8);
//
// impl PositionByte {
//     fn r#move(&mut self, gd: Gd) {
//         match gd {
//             Gd::None => {}
//             Gd::Up => {
//                 self.0 -= 1;
//                 // self.tail_x = self.tail_xy >> 3;
//                 // self.tail_y = self.tail_xy & 0b00000111;
//             }
//             Gd::Right => self.0 += 1,
//             Gd::Down => self.0 += 1,
//             Gd::Left => self.0 -= 1,
//         }
//     }
// }

/// 重力方向
#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Gd {
    #[default]
    None,
    Up,
    Right,
    Down,
    Left,
}

pub fn delay_ms(dur: u32) {
    FreeRtos::delay_ms(dur);
}

pub fn delay_us(dur: u32) {
    FreeRtos::delay_us(dur);
}

/// 小方
pub struct App<'d, C> {
    /// 蜂鸣器开关
    buzzer: bool,
    /// 界面
    uis: [Ui; 8],
    /// 当前界面的索引
    ui_current_idx: i8,
    /// 表情
    face: Face,
    gd: Gd,

    mpu6050: Mpu6050<I2cDriver<'d>>,
    ledc: LedControl<C>,
}

impl<'d, C> App<'d, C>
where
    C: Connector,
{
    fn gravity_direction(&mut self) {
        let accel = self.accel();
        let ax = accel.x();
        let ay = accel.y();

        let ax_abs = ax.abs();
        let ay_abs = ay.abs();
        if ax_abs > 0.5 || ay_abs > 0.5 {
            if ax_abs > ay_abs {
                if ax < -0.5 {
                    self.ledc.gd = Gd::Right;
                    self.gd = Gd::Right;
                }
                if ax > 0.5 {
                    self.ledc.gd = Gd::Left;
                    self.gd = Gd::Left;
                }
            }

            if ax_abs < ay_abs {
                if ay < -0.5 {
                    self.ledc.gd = Gd::Up;
                    self.gd = Gd::Up;
                }
                if ay > 0.5 {
                    self.ledc.gd = Gd::Down;
                    self.gd = Gd::Down;
                }
            }
        } else {
            self.ledc.gd = Gd::None;
            self.gd = Gd::None;
        }
    }

    pub fn new(mpu6050: Mpu6050<I2cDriver<'d>>, mut ledc: LedControl<C>) -> Self {
        ledc.set_intensity(0x01);

        App {
            buzzer: true,
            uis: Ui::uis(),
            ui_current_idx: 0,
            face: Face::new(),
            gd: Gd::default(),

            mpu6050,
            ledc,
        }
    }

    pub fn accel(&mut self) -> AccelF32 {
        self.mpu6050.accel().unwrap().scaled(AccelFullScale::G2)
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        loop {
            delay_ms(800);

            self.gravity_direction();
            if self.gd == Gd::default() {
                self.ledc
                    .upload_raw(self.uis[self.ui_current_idx as usize].ui());
                continue;
            }

            match self.gd {
                Gd::None => {
                    self.ledc
                        .upload_raw(self.uis[self.ui_current_idx as usize].ui());

                    self.ledc
                        .upload_raw(self.uis[self.ui_current_idx as usize].ui());
                }
                Gd::Up => {
                    // 向上进入对应的界面
                    let ui = &self.uis[self.ui_current_idx as usize];
                    match ui {
                        Ui::Timer => {}
                        Ui::Dice => {}
                        Ui::Snake => {
                            SnakeGame::new().run(&mut self);
                        }
                        Ui::BaGua => {
                            BaGua::run(&mut self);
                        }
                        Ui::Maze => {}
                        Ui::Temp => {}
                        Ui::Sound => {}
                        Ui::Version => {}
                    }
                }
                Gd::Right => {
                    self.ui_current_idx += 1;
                    if self.ui_current_idx >= self.uis.len() as i8 {
                        self.ui_current_idx = 0;
                    }
                    self.ledc
                        .upload_raw(self.uis[self.ui_current_idx as usize].ui());
                }
                Gd::Left => {
                    self.ui_current_idx -= 1;
                    if self.ui_current_idx < 0 {
                        self.ui_current_idx = self.uis.len() as i8 - 1;
                    }
                    self.ledc
                        .upload_raw(self.uis[self.ui_current_idx as usize].ui());
                }
                _ => {}
            }
        }
    }
}
