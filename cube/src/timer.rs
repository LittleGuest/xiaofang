use alloc::vec::Vec;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    Pixel,
};
use embedded_hal::delay::DelayNs;

use crate::{App, CubeRng, Position, RNG};

/// 沙漏
#[derive(Debug, Clone)]
pub struct Timer {
    pixels: Vec<TimerPixel>,
}

impl core::default::Default for Timer {
    fn default() -> Self {
        let mut pixels = Vec::<TimerPixel>::with_capacity(32);
        for y in 0..4 {
            for x in 0..8 {
                pixels.push(TimerPixel::new(Position::new(x, y), 0.3));
            }
        }
        Self { pixels }
    }
}

impl Timer {
    fn init<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        // app.ledc.clear_work();
        app.gravity_direction();
        app.ledc.write_pixels(self.pixels());

        // app.delay.delay_ms(1000_u32);
        // // 闪烁三次配音效后开始
        // (0..3).for_each(|_| {
        //     // TODO 音效
        //     app.ledc.set_brightness(0x01);
        //     app.delay.delay_ms(50_u32);
        //     app.ledc.set_brightness(0x00);
        //     app.delay.delay_ms(50_u32);
        // });
        //
        // app.delay.delay_ms(1000_u32);
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

    pub fn run<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
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

            app.delay.delay_ms(1000_u32);
            let mut pixel = self.pixels.remove(index);
            pixel.blink(app);
            pixel.r#move(app);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TimerPixel {
    pixel: Pixel<Rgb888>,
    #[allow(unused)]
    speed: f32,
}

impl TimerPixel {
    fn new(pos: Position, speed: f32) -> Self {
        Self {
            pixel: Pixel((pos.x as i32, pos.y as i32).into(), BinaryColor::On.into()),
            speed,
        }
    }

    /// 闪烁一下选中的像素,
    fn blink<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        (0..3).for_each(|_| {
            self.pixel.1 = BinaryColor::from(self.pixel.1).invert().into();
            app.ledc.write_pixel(self.pixel);
            app.delay.delay_ms(100_u32);
            // TODO 闪烁音效
            app.buzzer.timer_pixel_blinky();
        });
    }

    /// 执行像素的下落过程
    fn r#move<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.pixel.1 = BinaryColor::On.into();
        self.pixel.0.y += 4;
        app.delay.delay_ms(500_u32);
        app.ledc.write_pixel(self.pixel);
    }
}
