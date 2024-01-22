use alloc::vec::Vec;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    Pixel,
};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use crate::{App, CubeRng, Position, RNG};

#[derive(Debug, Clone, Copy)]
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
    fn blink<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        (0..3).for_each(|_| {
            self.pixel.1 = BinaryColor::from(self.pixel.1).invert().into();
            app.ledc.write_pixel(self.pixel);
            app.delay.delay_ms(100_u32);
            // TODO 闪烁音效
        });
    }

    /// 执行像素的下落过程
    fn r#move<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.pixel.1 = BinaryColor::On.into();
        self.pixel.0.y += 4;
        app.delay.delay_ms(500_u32);
        app.ledc.write_pixel(self.pixel);
        // TODO 优化下落过程
    }
}

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
    fn init<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        app.ledc.clear_work();
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

    pub fn pixels(&mut self) -> Vec<Pixel<Rgb888>> {
        self.pixels.iter().map(|p| p.pixel).collect::<Vec<_>>()
    }

    pub fn run<T: hal::i2c::Instance>(app: &mut App<T>) {
        let mut timer = Self::default();
        timer.init(app);

        loop {
            if timer.pixels.is_empty() {
                break;
            }
            let index = unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64)
                    .random(0, (timer.pixels.len()) as u32)
            } as usize;
            app.delay.delay_ms(1000_u32);
            let mut pixel = timer.pixels.remove(index);
            pixel.blink(app);
            pixel.r#move(app);
        }
    }
}
