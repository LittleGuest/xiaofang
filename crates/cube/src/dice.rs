#![doc = include_str!("../../../rfcs/002_dice.md")]

use crate::{App, CubeRng, buzzer};
use embassy_time::Timer;
use esp_hal::rng::Rng;

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

    fn random(rng: &mut Rng) -> [u8; 8] {
        let num = CubeRng(rng.random() as u64).random(1, 7_u32) as u8;
        Self::dice(num)
    }

    pub async fn run(&self, app: &mut App<'_>) {
        app.ledc.clear();
        loop {
            let accel = app.accel();
            if (accel.x() > 0.3 || accel.x() < -0.3)
                && (accel.y() > 0.3 || accel.y() < -0.3)
                && (0..30)
                    .map(|_| (app.accel().x(), app.accel().y()))
                    .any(|(x, y)| !(-0.3..=0.3).contains(&x) && !(-0.3..=0.3).contains(&y))
            {
                // 滚动动画：快速随机切换骰子面，逐渐减速
                let steps = 8;
                for i in (1..=steps).rev() {
                    let face = Self::random(&mut app.rng);
                    app.ledc.write_bytes(face);
                    // 逐渐增加停留时间（减速效果）
                    Timer::after_millis(30 + (steps - i) as u64 * 20).await;
                }
                // 最终结果
                let result = Self::random(&mut app.rng);
                app.ledc.write_bytes(result);
                buzzer::dice().await;
            }
            Timer::after_millis(800).await;

            app.check_pause().await;
        }
    }
}
