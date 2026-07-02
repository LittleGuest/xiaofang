#![doc = include_str!("../../../rfcs/004_timer.md")]

use crate::{App, CubeRng, buzzer, rng};
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
    async fn init(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.acc_direction();
        app.ledc.write_pixels(self.pixels());
        Timer::after_millis(1000).await;

        // 闪烁三次配音效后开始
        for _ in 0..3 {
            unsafe { buzzer().timer_pixel_blinky().await };
            app.ledc.set_brightness(0x01);
            Timer::after_millis(100).await;
            app.ledc.set_brightness(0x00);
            Timer::after_millis(100).await;
        }
        app.ledc.set_brightness(0x01);
        Timer::after_millis(500).await;
    }

    fn pixels(&self) -> Vec<Pixel<Rgb888>> {
        self.pixels.iter().map(|p| p.pixel).collect::<Vec<_>>()
    }

    /// 在某一列找最底部的像素
    fn last(&self, rx: i32) -> Option<usize> {
        let last = self
            .pixels
            .iter()
            .filter(|p| p.pixel.0.x == rx)
            .max_by_key(|p| p.pixel.0.y)?;
        self.pixels.iter().position(|p| p == last)
    }

    pub async fn run(&mut self, app: &mut App<'_>) {
        self.init(app).await;

        let mut rxs = alloc::vec![0, 1, 2, 3, 4, 5, 6, 7];

        loop {
            if self.pixels.is_empty() {
                // 所有像素落完，播放结束音效
                unsafe { buzzer().timers_over().await };
                Timer::after_millis(1000).await;
                break;
            }

            // 下甩暂停
            app.acc_direction();
            app.check_pause().await;

            // 随机一列掉下
            let rx = unsafe {
                CubeRng(rng().random() as u64).random(0, rxs.len() as u32)
            } as usize;
            let Some(index) = self.last(rxs[rx]) else {
                rxs.remove(rx);
                continue;
            };

            Timer::after_millis(800).await;
            let mut pixel = self.pixels.remove(index);
            pixel.blink(app).await;

            // 下落动画：逐行移动到下半部分
            let target_y = pixel.pixel.0.y + 4;
            while pixel.pixel.0.y < target_y {
                pixel.pixel.0.y += 1;
                // 重绘：所有剩余上半像素 + 当前的下落像素
                app.ledc.clear();
                app.ledc.write_pixels(self.pixels());
                app.ledc.write_pixel(pixel.pixel);
                Timer::after_millis(80).await;
            }

            // 下落完成，反弹音效
            unsafe { buzzer().timer_pixel_rebound().await };
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

    /// 闪烁一下选中的像素
    async fn blink(&mut self, app: &mut App<'_>) {
        for _ in 0..3 {
            self.pixel.1 = BinaryColor::from(self.pixel.1).invert().into();
            app.ledc.write_pixel(self.pixel);
            Timer::after_millis(100).await;
            unsafe { buzzer().timer_pixel_blinky().await };
        }
    }
}
