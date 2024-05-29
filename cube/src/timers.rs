#![doc = include_str!("../../rfcs/004_timer.md")]

use crate::{App, CubeRng, BUZZER, RNG};
use alloc::vec::Vec;
use embassy_time::Timer;
use embedded_graphics::geometry::Point;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    Pixel,
};

/// 沙漏
#[derive(Debug, Clone)]
pub struct Timers {
    pixels: Vec<TimerPixel>,
}

impl core::default::Default for Timers {
    fn default() -> Self {
        let mut pixels = Vec::<TimerPixel>::with_capacity(32);
        for y in 0..4 {
            for x in 0..8 {
                pixels.push(TimerPixel::new(Point::new(x, y), 0.3));
            }
        }
        Self { pixels }
    }
}

impl Timers {
    fn init<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        app.gravity_direction();
        app.ledc.write_pixels(self.pixels());

        // Timer::after_millis(1000).await;
        // // 闪烁三次配音效后开始
        // (0..3).for_each(|_| {
        //     // TODO: 音效
        //     app.ledc.set_brightness(0x01);
        // Timer::after_millis(50).await;
        //     app.ledc.set_brightness(0x00);
        // Timer::after_millis(50).await;
        // });
        //
        // Timer::after_millis(1000).await;
    }

    fn pixels(&mut self) -> Vec<Pixel<Rgb888>> {
        self.pixels.iter().map(|p| p.pixel).collect::<Vec<_>>()
    }

    /// 在某一列找
    fn last(&self, rx: i32) -> Option<usize> {
        let last = self
            .pixels
            .iter()
            .filter(|p| p.pixel.0.x == rx)
            .max_by_key(|p| p.pixel.0.y)?;
        self.pixels.iter().position(|p| p == last)
    }

    pub async fn run<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<'_, T>) {
        self.init(app);

        let mut rxs = vec![0, 1, 2, 3, 4, 5, 6, 7];

        loop {
            if self.pixels.is_empty() {
                break;
            }

            // 随机一列掉下
            let rx = unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64).random(0, rxs.len() as u32)
            } as usize;
            let Some(index) = self.last(rxs[rx]) else {
                rxs.remove(rx);
                continue;
            };

            Timer::after_millis(1000).await;
            let mut pixel = self.pixels.remove(index);
            pixel.blink(app).await;
            pixel.r#move(app).await;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TimerPixel {
    pixel: Pixel<Rgb888>,
    speed: f32,
}

impl TimerPixel {
    fn new(pos: Point, speed: f32) -> Self {
        Self {
            pixel: Pixel(pos, BinaryColor::On.into()),
            speed,
        }
    }

    /// 闪烁一下选中的像素,
    async fn blink<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<'_, T>) {
        for _ in 0..3 {
            self.pixel.1 = BinaryColor::from(self.pixel.1).invert().into();
            app.ledc.write_pixel(self.pixel);
            Timer::after_millis(100).await;
            unsafe { BUZZER.assume_init_mut().timer_pixel_blinky().await };
        }
    }

    /// 执行像素的下落过程
    async fn r#move<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<'_, T>) {
        self.pixel.1 = BinaryColor::On.into();
        self.pixel.0.y += 4;
        Timer::after_millis(500).await;
        app.ledc.write_pixel(self.pixel);
    }
}
