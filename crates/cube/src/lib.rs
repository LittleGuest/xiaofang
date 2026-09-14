#![no_std]
#![no_main]

use alloc::vec::Vec;

use bagua::BaGua;
use cube_man::CubeManGame;
use cube_rand::CubeRng;
use dice::Dice;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_graphics_core::pixelcolor::Rgb888;
use embedded_storage::{ReadStorage, Storage};
use esp_hal::{
    Blocking,
    analog::adc::{Adc, AdcCalLine, AdcPin},
    i2c::master::I2c,
    rng::Rng,
};
use esp_radio::esp_now::EspNow;
use esp_storage::FlashStorage;
use face::Face;
use ledc::LedControl;
use maze::Maze;
use mpu6050_dmp::{
    accel::{AccelF32, AccelFullScale},
    sensor::Mpu6050,
};
use snake::SnakeGame;
use timers::Timers;
use ui::Ui;

use crate::{dodge_cube::DodgeCubeGame, sokoban::Sokoban};

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
pub mod music_spectrum;
pub mod play_ball;
pub mod player;
pub mod snake;
pub mod sokoban;
pub mod timers;
pub mod ui;
pub mod wifi_ap;

pub type Color = Rgb888;

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
pub struct App<'d> {
    /// 界面
    uis: Vec<Ui>,
    /// 当前界面的索引
    ui_current_idx: i8,
    /// 表情
    face: Face,
    ad: Ad,
    /// 上一次成功读取的加速度, I2C 瞬时失败时回退使用, 避免热循环 panic
    last_accel: Option<AccelF32>,

    mpu6050: Mpu6050<I2c<'d, Blocking>>,
    ledc: LedControl<'d>,
    rng: Rng,
    flash: FlashStorage<'d>,

    /// 麦克风 ADC(音乐频谱采样)
    adc: Adc<'d, esp_hal::peripherals::ADC1<'d>, Blocking>,
    mic_pin: AdcPin<
        esp_hal::peripherals::GPIO0<'d>,
        esp_hal::peripherals::ADC1<'d>,
        AdcCalLine<esp_hal::peripherals::ADC1<'d>>,
    >,

    /// ESP-NOW 联机接口（对打球）
    esp_now: Option<EspNow<'d>>,
    /// 本机 MAC 地址
    my_mac: [u8; 6],

    #[allow(unused)]
    spawner: Spawner,
}

impl<'d> App<'d> {
    pub fn accel(&mut self) -> AccelF32 {
        match self.mpu6050.accel() {
            Ok(a) => {
                let a = a.scaled(AccelFullScale::G2);
                self.last_accel = Some(a);
                a
            }
            // I2C 总线瞬时出错: 沿用上一次成功值, 不 panic
            Err(_) => self.last_accel.unwrap_or(AccelF32::new(0.0, 0.0, 0.0)),
        }
    }

    /// 加速度方向（基于倾斜姿态）
    pub fn acc_direction(&mut self) {
        let accel = self.accel();
        let ax = accel.x();
        let ay = accel.y();

        let ax_abs = ax.abs();
        let ay_abs = ay.abs();

        if ax_abs > 0.5 || ay_abs > 0.5 {
            if ax_abs >= ay_abs {
                // X 轴倾斜更大，左右方向
                self.ad = if ax < 0.0 { Ad::Right } else { Ad::Left };
            } else {
                // Y 轴倾斜更大，前后方向
                self.ad = if ay < 0.0 { Ad::Front } else { Ad::Back };
            }
        } else {
            self.ad = Ad::None;
        }
    }

    /// 检测下甩手势（设备向地面加速）→ 暂停游戏
    /// 正常握持时 az ≈ 1g（重力），下甩时 az 骤降到 0.3g 以下
    /// 暂停后等待恢复正常握持再继续游戏
    pub async fn check_pause(&mut self) {
        let az = self.accel().z();
        if az < 0.3 {
            // 下甩手势触发，暂停游戏
            self.ledc.set_brightness(0x00); // 熄屏表示暂停
            // 等待设备恢复正常姿态
            loop {
                Timer::after_millis(100).await;
                let az = self.accel().z();
                // az 回到 0.7 以上表示设备恢复静止握持
                if az > 0.7 {
                    // 等用户稳定握持
                    Timer::after_millis(300).await;
                    self.ledc.set_brightness(0x01); // 恢复亮度
                    self.acc_direction();
                    break;
                }
            }
        }
    }

    /// 从麦克风(ADC)采样一段音频,以 0 为中点归一化到 [-1,1]
    pub fn sample_audio(&mut self, buf: &mut [f32]) {
        let mid = 2048.0;
        for s in buf.iter_mut() {
            let raw = loop {
                match self.adc.read_oneshot(&mut self.mic_pin) {
                    Ok(v) => break v,
                    Err(nb::Error::WouldBlock) => continue,
                    Err(_) => break 2048,
                }
            };
            *s = (raw as f32 - mid) / mid;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mpu6050: Mpu6050<I2c<'d, Blocking>>,
        mut ledc: LedControl<'d>,
        spawner: Spawner,
        flash: FlashStorage<'d>,
        rng: Rng,
        adc: Adc<'d, esp_hal::peripherals::ADC1<'d>, Blocking>,
        mic_pin: AdcPin<
            esp_hal::peripherals::GPIO0<'d>,
            esp_hal::peripherals::ADC1<'d>,
            AdcCalLine<esp_hal::peripherals::ADC1<'d>>,
        >,
        esp_now: EspNow<'d>,
        my_mac: [u8; 6],
    ) -> Self {
        ledc.set_brightness(0x01);

        App {
            uis: Ui::uis().into(),
            ui_current_idx: 0,
            face: Face::default(),
            ad: Ad::default(),
            last_accel: None,

            mpu6050,
            ledc,
            rng,
            flash,
            adc,
            mic_pin,

            esp_now: Some(esp_now),
            my_mac,

            spawner,
        }
    }

    pub async fn run(mut self) -> ! {
        let flash_addr = 0x9100;
        let mut flash_data = [0u8; 8];
        self.flash.read(flash_addr, &mut flash_data).ok();
        // info!(
        //     "Read flash data from {:x}:  {:02x?}",
        //     flash_addr,
        //     &flash_data[..8]
        // );

        loop {
            Timer::after_millis(500).await;

            self.acc_direction();

            if self.ad == Ad::default() {
                self.ledc.write_bytes(self.uis[self.ui_current_idx as usize].ui());
                continue;
            }

            match self.ad {
                // 向上进入对应的界面
                Ad::Front => {
                    buzzer::menu_confirm().await;
                    match self.uis[self.ui_current_idx as usize] {
                        Ui::Timer => Timers::default().run(&mut self).await,
                        Ui::MusicSpectrum => {
                            music_spectrum::MusicSpectrum::run(&mut self).await;
                        }
                        Ui::Dice => Dice.run(&mut self).await,
                        Ui::Snake => {
                            let mut snake = SnakeGame::new(&mut self.rng);
                            // 最高分从flash中获取
                            snake.highest = flash_data[0x00];
                            snake.run(&mut self).await;
                            // 游戏结束将最高分再次写入flash
                            flash_data[0x00] = snake.highest;
                            self.flash.write(flash_addr, &flash_data).ok();
                        }
                        Ui::BaGua => BaGua::run(&mut self).await,
                        Ui::Maze => {
                            let mut cr = CubeRng(self.rng.random() as u64).random_range(19..=33);
                            if cr.is_multiple_of(2) {
                                cr += 1;
                            }
                            Maze::new(cr, cr, &mut self.rng).run(&mut self).await;
                        }
                        Ui::CubeMan => {
                            let mut cm = CubeManGame::new();
                            // 最高分从flash中获取
                            cm.highest = flash_data[0x01];
                            cm.run(&mut self).await;
                            // 游戏结束将最高分再次写入flash
                            flash_data[0x01] = cm.highest;
                            self.flash.write(flash_addr, &flash_data).ok();
                        }
                        Ui::Sokoban => Sokoban::new().run(&mut self).await,
                        Ui::PlayBall => play_ball::PlayBall::new().run(&mut self).await,
                        Ui::DodgeCube => {
                            let mut dc = DodgeCubeGame::new();
                            dc.highest = flash_data[0x02];
                            dc.run(&mut self).await;
                            flash_data[0x02] = dc.highest;
                            self.flash.write(flash_addr, &flash_data).ok();
                        }
                        Ui::Sound => buzzer::change(),
                    }
                }
                Ad::Right => {
                    self.ui_current_idx += 1;
                    if self.ui_current_idx >= self.uis.len() as i8 {
                        self.ui_current_idx = 0;
                    }
                    self.ledc.write_bytes(self.uis[self.ui_current_idx as usize].ui());
                    buzzer::menu_select().await;
                }
                Ad::Left => {
                    self.ui_current_idx -= 1;
                    if self.ui_current_idx < 0 {
                        self.ui_current_idx = self.uis.len() as i8 - 1;
                    }
                    self.ledc.write_bytes(self.uis[self.ui_current_idx as usize].ui());
                    buzzer::menu_select().await;
                }
                _ => {
                    self.ledc.write_bytes(self.uis[self.ui_current_idx as usize].ui());
                }
            }
        }
    }
}
