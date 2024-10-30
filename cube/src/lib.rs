#![no_std]
#![no_main]

use crate::{dodge_cube::DodgeCubeGame, sokoban::Sokoban};
use alloc::vec::Vec;
use bagua::BaGua;
use buzzer::Buzzer;
use core::mem::MaybeUninit;
use cube_man::CubeManGame;
use cube_rand::CubeRng;
use dice::Dice;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_graphics_core::pixelcolor::Rgb888;
use embedded_storage::{ReadStorage, Storage};
use esp_hal::{rng::Rng, Blocking};
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
pub mod wifi_ap;

pub type Color = Rgb888;
pub static mut RNG: MaybeUninit<Rng> = MaybeUninit::uninit();
pub static mut BUZZER: MaybeUninit<Buzzer> = MaybeUninit::uninit();
pub static mut LEDCTL: MaybeUninit<LedControl> = MaybeUninit::uninit();

/// 物体移动方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    /// 反方向
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Self::Down,
            Direction::Right => Self::Left,
            Direction::Down => Self::Up,
            Direction::Left => Self::Right,
        }
    }
}

/// 加速度方向
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Ad {
    #[default]
    None,
    Front,
    Right,
    Back,
    Left,
    Up,
    Down,
}

impl core::fmt::Display for Ad {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Ad::None => f.write_str("None"),
            Ad::Front => f.write_str("Up"),
            Ad::Right => f.write_str("Right"),
            Ad::Back => f.write_str("Down"),
            Ad::Left => f.write_str("Left"),
            Ad::Up => f.write_str("Up"),
            Ad::Down => f.write_str("Down"),
        }
    }
}

impl From<Direction> for Ad {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Up => Self::Front,
            Direction::Right => Self::Right,
            Direction::Down => Self::Back,
            Direction::Left => Self::Left,
        }
    }
}

/// 坐标
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl From<Point> for embedded_graphics_core::geometry::Point {
    fn from(p: Point) -> Self {
        Self { x: p.x, y: p.y }
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
    ad: Ad,

    mpu6050: Mpu6050<esp_hal::i2c::I2c<'d, T, Blocking>>,
    ledc: LedControl<'d>,
    spawner: Spawner,
}

impl<'d, T> App<'d, T>
where
    T: esp_hal::i2c::Instance,
{
    pub fn accel(&mut self) -> AccelF32 {
        self.mpu6050.accel().unwrap().scaled(AccelFullScale::G2)
    }

    /// 加速度方向
    pub fn acc_direction(&mut self) {
        let accel = self.accel();
        let ax = accel.x();
        let ay = accel.y();
        let az = accel.z();

        let ax_abs = if ax <= 0.0 { 0.0 - ax } else { ax };
        let ay_abs = if ay <= 0.0 { 0.0 - ay } else { ay };
        let az_abs = if az <= 0.0 { 0.0 - az } else { az };
        if ax_abs > 0.5 || ay_abs > 0.5 {
            if ax_abs > ay_abs {
                if ax < -0.5 {
                    self.ad = Ad::Right;
                }
                if ax > 0.5 {
                    self.ad = Ad::Left;
                }
            }

            if ax_abs < ay_abs {
                if ay < -0.5 {
                    self.ad = Ad::Front
                }
                if ay > 0.5 {
                    self.ad = Ad::Back;
                }
            }
        } else if az_abs >= 1.0 {
            self.ad = Ad::Down;
        } else {
            self.ad = Ad::None;
        }
    }

    /// 退出
    pub fn quit(&self) -> bool {
        Ad::Down.eq(&self.ad)
    }

    pub fn new(
        mpu6050: Mpu6050<esp_hal::i2c::I2c<'d, T, Blocking>>,
        mut ledc: LedControl<'d>,
        spawner: Spawner,
    ) -> Self {
        ledc.set_brightness(0x01);

        App {
            uis: Ui::uis().into(),
            ui_current_idx: 0,
            face: Face::default(),
            ad: Ad::default(),

            mpu6050,
            ledc,
            spawner,
        }
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

            self.acc_direction();

            if self.ad == Ad::default() {
                self.ledc
                    .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                continue;
            }

            match self.ad {
                // 向上进入对应的界面
                Ad::Front => {
                    unsafe { BUZZER.assume_init_mut().menu_confirm().await };
                    match self.uis[self.ui_current_idx as usize] {
                        Ui::Timer => Timers::default().run(&mut self).await,
                        Ui::MusicSpectrum => {
                            // 麦克风采集信息，通过fft转换
                        }
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
                Ad::Right => {
                    self.ui_current_idx += 1;
                    if self.ui_current_idx >= self.uis.len() as i8 {
                        self.ui_current_idx = 0;
                    }
                    self.ledc
                        .write_bytes(self.uis[self.ui_current_idx as usize].ui());
                    unsafe { BUZZER.assume_init_mut().menu_select().await };
                }
                Ad::Left => {
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
