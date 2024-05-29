#![no_std]
#![no_main]
#![feature(slice_flatten)]
#![feature(extract_if)]
#![allow(unused)]

use core::{borrow::BorrowMut, mem::MaybeUninit};

use alloc::vec::Vec;
use bagua::BaGua;
use buzzer::Buzzer;
use cube_man::CubeManGame;
use cube_rand::CubeRng;
use dice::Dice;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_graphics_core::pixelcolor::Rgb888;
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

use static_cell::{ConstStaticCell, StaticCell};
use timers::Timers;
use ui::Ui;

use crate::{dodge_cube::DodgeCubeGame, sokoban::Sokoban};

#[macro_use]
extern crate alloc;

pub mod bagua;
pub mod battery;
pub mod buzzer;
pub mod cube_man;
pub mod dice;
pub mod dodge_cube;
pub mod face;
pub mod ledc;
pub mod map;
pub mod mapping;
pub mod maze;
pub mod player;
pub mod snake;
pub mod sokoban;
pub mod timers;
pub mod ui;

pub type CubeColor = Rgb888;
pub static mut RNG: MaybeUninit<Rng> = MaybeUninit::uninit();
pub static mut BUZZER: MaybeUninit<Buzzer> = MaybeUninit::uninit();
pub static mut LEDCTL: MaybeUninit<LedControl> = MaybeUninit::uninit();

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
pub enum Gd {
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
    spawner: Spawner,
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
        spawner: Spawner,
    ) -> Self {
        ledc.set_brightness(0x01);

        App {
            uis: Ui::uis().into(),
            ui_current_idx: 0,
            face: Face::default(),
            gd: Gd::default(),

            mpu6050,
            ledc,
            spawner,
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
            Timer::after_millis(500).await;

            self.gravity_direction();

            if self.gd == Gd::default() {
                self.ledc
                    .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                continue;
            }

            match self.gd {
                // 向上进入对应的界面
                Gd::Up => {
                    unsafe { BUZZER.assume_init_mut().menu_confirm().await };
                    match self.uis[self.ui_current_idx as usize] {
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
                        Ui::Sokoban => Sokoban::new().run(&mut self).await,
                        Ui::DodgeCube => DodgeCubeGame::new().run(&mut self).await,
                        Ui::Sound => unsafe { BUZZER.assume_init_mut().change() },
                    }
                }
                Gd::Right => {
                    self.ui_current_idx += 1;
                    if self.ui_current_idx >= self.uis.len() as i8 {
                        self.ui_current_idx = 0;
                    }
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                    unsafe { BUZZER.assume_init_mut().menu_select().await };
                }
                Gd::Left => {
                    self.ui_current_idx -= 1;
                    if self.ui_current_idx < 0 {
                        self.ui_current_idx = self.uis.len() as i8 - 1;
                    }
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                    unsafe { BUZZER.assume_init_mut().menu_select().await };
                }
                _ => {
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                }
            }
        }
    }
}
