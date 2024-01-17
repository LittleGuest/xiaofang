#![no_std]
#![no_main]
#![feature(slice_flatten)]

use core::{cell::OnceCell, mem::MaybeUninit, ops::RangeBounds};

use alloc::vec::{IntoIter, Vec};
use bagua::BaGua;
use cube_rand::CubeRng;
use dice::Dice;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    Pixel,
};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use face::Face;
use hal::{i2c::I2C, Delay};
use ledc::LedControl;
use maze::Maze;
use mpu6050_dmp::{
    accel::{AccelF32, AccelFullScale},
    sensor::Mpu6050,
};
use snake::SnakeGame;
use static_cell::StaticCell;
use timer::Timer;
use ui::Ui;

#[macro_use]
extern crate alloc;

mod bagua;
mod battery;
mod dice;
mod face;
pub mod ledc;
mod mapping;
mod maze;
mod snake;
mod timer;
mod ui;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub static mut Rng: MaybeUninit<hal::Rng> = MaybeUninit::uninit();

pub fn init() {
    const HEAP_SIZE: usize = 64 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

/// 左上角为坐标原点,横x,纵y
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Position {
    x: i8,
    y: i8,
}
impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0 as i8,
            y: value.1 as i8,
        }
    }
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
        pos
    }

    fn random(x: i8, y: i8) -> Self {
        unsafe {
            Self {
                x: CubeRng(Rng.assume_init_mut().random() as u64).random(0, x as u32) as i8,
                y: CubeRng(Rng.assume_init_mut().random() as u64).random(0, y as u32) as i8,
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
                x: CubeRng(Rng.assume_init_mut().random() as u64).random_range(x) as i8,
                y: CubeRng(Rng.assume_init_mut().random() as u64).random_range(y) as i8,
            }
        }
    }
}

impl From<Position> for Pixel<Rgb888> {
    fn from(p: Position) -> Self {
        Self((p.x as i32, p.y as i32).into(), BinaryColor::On.into())
    }
}

// impl FromIterator<Position> for Iterator<Item = Pixel<Rgb888>> {}
// impl From<Vec<Position>> for Vec<Pixel<Rgb888>> {
//     fn from(value: Vec<Position>) -> Self {
//         todo!()
//     }
// }

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
    T: hal::i2c::Instance,
{
    /// 蜂鸣器开关
    buzzer: bool,
    /// 界面
    uis: Vec<Ui>,
    /// 当前界面的索引
    ui_current_idx: i8,
    /// 表情
    face: Face,
    gd: Gd,

    mpu6050: Mpu6050<I2C<'d, T>>,
    ledc: LedControl<'d>,
    delay: Delay,
}

impl<'d, T> App<'d, T>
where
    T: hal::i2c::Instance,
{
    fn gravity_direction(&mut self) {
        let accel = self.accel();
        let ax = accel.x();
        let ay = accel.y();

        // let ax_abs = ax.abs();
        // let ay_abs = ay.abs();
        let ax_abs = if ax <= 0.0 { 0.0 - ax } else { ax };
        let ay_abs = if ay <= 0.0 { 0.0 - ay } else { ay };
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

    pub fn new(delay: Delay, mpu6050: Mpu6050<I2C<'d, T>>, mut ledc: LedControl<'d>) -> Self {
        ledc.set_brightness(0x01);

        App {
            buzzer: true,
            uis: Ui::uis(),
            ui_current_idx: 0,
            face: Face::new(),
            gd: Gd::default(),

            mpu6050,
            ledc,
            delay,
        }
    }

    pub fn accel(&mut self) -> AccelF32 {
        self.mpu6050.accel().unwrap().scaled(AccelFullScale::G2)
    }

    pub fn run(mut self) -> ! {
        loop {
            self.delay.delay_ms(600_u32);

            self.gravity_direction();

            if self.gd == Gd::default() {
                self.ledc
                    .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                continue;
            }

            match self.gd {
                Gd::None => {
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());

                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                }
                Gd::Up => {
                    // 向上进入对应的界面
                    let ui = &self.uis[self.ui_current_idx as usize];
                    match ui {
                        Ui::Timer => Timer::run(&mut self),
                        Ui::Dice => Dice::run(&mut self),
                        Ui::Snake => SnakeGame::new().run(&mut self),
                        Ui::BaGua => BaGua::run(&mut self),
                        Ui::Maze => Maze::<11, 11>::new().run(&mut self),
                        Ui::Sound => {}
                    }
                }
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
                _ => {}
            }
        }
    }
}
