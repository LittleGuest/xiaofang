#![no_std]
#![no_main]
#![feature(slice_flatten)]
#![allow(unused)]

use core::{mem::MaybeUninit, ops::RangeBounds};

use alloc::vec::Vec;
use bagua::BaGua;
use buzzer::Buzzer;
use cube_man::CubeManGame;
use cube_rand::CubeRng;
use dice::Dice;
use embassy_time::Timer;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    Pixel,
};
use embedded_storage::{ReadStorage, Storage};
use esp_hal::{i2c::I2C, rng::Rng, Blocking};
use esp_storage::FlashStorage;
use face::Face;
use ledc::LedControl;
use log::info;
use maze::Maze;
use mpu6050_dmp::{
    accel::{AccelF32, AccelFullScale},
    sensor::Mpu6050,
};
use snake::SnakeGame;

use timers::Timers;
use ui::Ui;

use crate::sokoban::Sokoban;

#[macro_use]
extern crate alloc;

pub mod bagua;
pub mod battery;
pub mod buzzer;
pub mod cube_man;
pub mod dice;
pub mod face;
pub mod ledc;
pub mod mapping;
pub mod maze;
pub mod snake;
pub mod sokoban;
pub mod timers;
pub mod ui;

pub static mut RNG: MaybeUninit<Rng> = MaybeUninit::uninit();

struct PositionVec(Vec<Position>);

/// 左上角为坐标原点,横x,纵y
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Position {
    x: i32,
    y: i32,
}
impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0 as i32,
            y: value.1 as i32,
        }
    }
}

impl FromIterator<(usize, usize)> for PositionVec {
    fn from_iter<T: IntoIterator<Item = (usize, usize)>>(iter: T) -> Self {
        todo!()
    }
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
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
        pos
    }

    fn random(x: i32, y: i32) -> Self {
        unsafe {
            Self {
                x: CubeRng(RNG.assume_init_mut().random() as u64).random(0, x as u32) as i32,
                y: CubeRng(RNG.assume_init_mut().random() as u64).random(0, y as u32) as i32,
            }
        }
    }

    // fn random_range(x: impl RangeBounds<i8>, y: impl RangeBounds<i8>) -> Self {
    //     Self {
    //         x: Rng.random_range(x),
    //         y: Rng.random_range(y),
    //     }
    // }

    fn random_range_usize(x: impl RangeBounds<usize>, y: impl RangeBounds<usize>) -> Self {
        unsafe {
            Self {
                x: CubeRng(RNG.assume_init_mut().random() as u64).random_range(x) as i32,
                y: CubeRng(RNG.assume_init_mut().random() as u64).random_range(y) as i32,
            }
        }
    }
}

impl From<Position> for Pixel<Rgb888> {
    fn from(p: Position) -> Self {
        Self((p.x, p.y).into(), BinaryColor::On.into())
    }
}

impl From<&Position> for Pixel<Rgb888> {
    fn from(p: &Position) -> Self {
        Self((p.x, p.y).into(), BinaryColor::On.into())
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

impl core::fmt::Display for Gd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Gd::None => f.write_str("None"),
            Gd::Up => f.write_str("Up"),
            Gd::Right => f.write_str("Right"),
            Gd::Down => f.write_str("Down"),
            Gd::Left => f.write_str("Left"),
        }
    }
}

impl From<Direction> for Gd {
    fn from(v: Direction) -> Self {
        match v {
            Direction::Up => Self::Up,
            Direction::Right => Self::Right,
            Direction::Down => Self::Down,
            Direction::Left => Self::Left,
        }
    }
}

/// 小方
pub struct App<'d, T>
where
    T: esp_hal::i2c::Instance,
{
    /// 界面
    uis: Vec<Ui>,
    /// 当前界面的索引
    ui_current_idx: i8,
    /// 表情
    face: Face,
    gd: Gd,

    mpu6050: Mpu6050<I2C<'d, T, Blocking>>,
    ledc: LedControl<'d>,
    buzzer: Buzzer<'d>,
}

impl<'d, T> App<'d, T>
where
    T: esp_hal::i2c::Instance,
{
    fn gravity_direction(&mut self) {
        let accel = self.accel();
        let ax = accel.x();
        let ay = accel.y();

        let ax_abs = if ax <= 0.0 { 0.0 - ax } else { ax };
        let ay_abs = if ay <= 0.0 { 0.0 - ay } else { ay };
        if ax_abs > 0.5 || ay_abs > 0.5 {
            if ax_abs > ay_abs {
                if ax < -0.5 {
                    self.gd = Gd::Right;
                }
                if ax > 0.5 {
                    self.gd = Gd::Left;
                }
            }

            if ax_abs < ay_abs {
                if ay < -0.5 {
                    self.gd = Gd::Up;
                }
                if ay > 0.5 {
                    self.gd = Gd::Down;
                }
            }
        } else {
            self.gd = Gd::None;
        }
    }

    pub fn new(
        mpu6050: Mpu6050<I2C<'d, T, Blocking>>,
        mut ledc: LedControl<'d>,
        buzzer: Buzzer<'d>,
    ) -> Self {
        ledc.set_brightness(0x01);

        App {
            uis: Ui::uis(),
            ui_current_idx: 0,
            face: Face::default(),
            gd: Gd::default(),

            mpu6050,
            ledc,
            buzzer,
        }
    }

    pub fn accel(&mut self) -> AccelF32 {
        self.mpu6050.accel().unwrap().scaled(AccelFullScale::G2)
    }

    pub async fn run(mut self) -> ! {
        let flash_addr = 0x9100;
        let mut flash = FlashStorage::new();
        let mut flash_data = [0u8; 8];
        flash.read(flash_addr, &mut flash_data).ok();
        info!(
            "Read flash data from {:x}:  {:02x?}",
            flash_addr,
            &flash_data[..8]
        );

        loop {
            Timer::after_millis(600).await;

            self.gravity_direction();

            if self.gd == Gd::default() {
                self.ledc
                    .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                continue;
            }

            match self.gd {
                // 向上进入对应的界面
                Gd::Up => match self.uis[self.ui_current_idx as usize] {
                    Ui::Timer => Timers::default().run(&mut self).await,
                    Ui::Dice => Dice.run(&mut self).await,
                    Ui::Snake => {
                        let mut snake = SnakeGame::new();
                        // 最高分从flash中获取
                        snake.highest = flash_data[0x00];
                        snake.run(&mut self).await;
                        // 游戏结束将最高分再次写入flash
                        flash_data[0x00] = snake.highest;
                        flash.write(flash_addr, &flash_data).ok();
                    }
                    Ui::BaGua => BaGua::run(&mut self).await,
                    Ui::Maze => {
                        let mut cr = unsafe {
                            CubeRng(RNG.assume_init_mut().random() as u64).random_range(19..=33)
                        };
                        if cr % 2 == 0 {
                            cr += 1;
                        }
                        Maze::new(cr, cr).run(&mut self).await;
                    }
                    Ui::CubeMan => {
                        let mut cm = CubeManGame::new();
                        // 最高分从flash中获取
                        cm.highest = flash_data[0x01];
                        cm.run(&mut self).await;
                        // 游戏结束将最高分再次写入flash
                        flash_data[0x01] = cm.highest;
                        flash.write(flash_addr, &flash_data).ok();
                    }
                    Ui::Sokoban => {
                        Sokoban::new().run(&mut self).await;
                    }
                    Ui::Sound => {}
                },
                Gd::Right => {
                    self.ui_current_idx += 1;
                    if self.ui_current_idx >= self.uis.len() as i8 {
                        self.ui_current_idx = 0;
                    }
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                }
                Gd::Left => {
                    self.ui_current_idx -= 1;
                    if self.ui_current_idx < 0 {
                        self.ui_current_idx = self.uis.len() as i8 - 1;
                    }
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                }
                _ => {
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                }
            }
        }
    }
}
