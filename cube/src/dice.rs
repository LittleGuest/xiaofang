use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use crate::{App, CubeRng, RNG};

/// 骰子
#[derive(Debug)]
pub struct Dice;

impl Dice {
    #[rustfmt::skip]
    fn dice(num: u8) -> [u8; 8] {
        match num {
            1 => [
                0b00000000,
                0b00011000,
                0b00111100,
                0b01111110,
                0b01111110,
                0b00111100,
                0b00011000,
                0b00000000,
            ],
            2 => [
                0b00000110,
                0b00001111,
                0b00001111,
                0b00000110,
                0b01100000,
                0b11110000,
                0b11110000,
                0b01100000,
            ],
            3 => [
                0b00011000,
                0b00111100,
                0b00111100,
                0b00011000,
                0b11000011,
                0b11100111,
                0b11100111,
                0b11100111,
            ],
            4 =>[
                0b11100111,
                0b11100111,
                0b11100111,
                0b00000000,
                0b00000000,
                0b11100111,
                0b11100111,
                0b11100111,
            ],
            5 => [
                0b11100111,
                0b11100111,
                0b11011011,
                0b00111100,
                0b00111100,
                0b11011011,
                0b11100111,
                0b11100111,
            ],
            6 => [
                0b11100111,
                0b11100111,
                0b00000000,
                0b11100111,
                0b11100111,
                0b00000000,
                0b11100111,
                0b11100111,
            ],
            _ => [0;8],
        }
    }

    fn random() -> [u8; 8] {
        let num = unsafe { CubeRng(RNG.assume_init_mut().random() as u64).random(1, 7_u32) } as u8;
        Self::dice(num)
    }

    pub fn run<T: hal::i2c::Instance>(&self, app: &mut App<T>) {
        app.ledc.clear();
        loop {
            let accel = app.accel();
            // if accel.x().abs() > 0.3 && accel.y().abs() > 0.3 {
            if (accel.x() > 0.3 || accel.x() < -0.3)
                && (accel.y() > 0.3 || accel.y() < -0.3)
                && (0..30)
                    // .map(|_| (app.accel().x().abs(), app.accel().y().abs()))
                    // .any(|(x, y)| x > 0.3 && y > 0.3)
                    .map(|_| (app.accel().x(), app.accel().y()))
                    .any(|(x, y)| !(-0.3..=0.3).contains(&x) && !(-0.3..=0.3).contains(&y))
            {
                app.ledc.bitmap(Self::random());
                app.ledc.upload();
            }
            app.delay.delay_ms(800_u32);

            // TODO 推出骰子模式
        }
    }
}
