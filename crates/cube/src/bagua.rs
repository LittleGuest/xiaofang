#![doc = include_str!("../../../rfcs/001_bagua.md")]

use crate::{App, CubeRng};
use embassy_time::Timer;
use esp_hal::rng::Rng;

/// 八卦
#[derive(Debug)]
pub enum BaGua {
    Qian,
    Kun,
    Zhen,
    Gen,
    Li,
    Kan,
    Dui,
    Xun,
}

impl BaGua {
    #[rustfmt::skip]
    fn bagua(num: u8) -> [u8; 8] {
        match num {
            // 八卦：乾
            1 => [
                0b11111111,
                0b11111111,
                0b00000000,
                0b11111111,
                0b11111111,
                0b00000000,
                0b11111111,
                0b11111111,
            ],
            // 八卦：坤
            2 => [
                0b11100111,
                0b11100111,
                0b00000000,
                0b11100111,
                0b11100111,
                0b00000000,
                0b11100111,
                0b11100111,
            ],
            // 八卦：震
            3 => [
                0b11100111,
                0b11100111,
                0b00000000,
                0b11100111,
                0b11100111,
                0b00000000,
                0b11111111,
                0b11111111,
            ],
            // 八卦：艮
            4 => [
                0b11111111,
                0b11111111,
                0b00000000,
                0b11100111,
                0b11100111,
                0b00000000,
                0b11100111,
                0b11100111,
            ],
            // 八卦：离
            5 => [
                0b11111111,
                0b11111111,
                0b00000000,
                0b11100111,
                0b11100111,
                0b00000000,
                0b11111111,
                0b11111111,
            ],
            // 八卦：坎
            6 => [
                0b11100111,
                0b11100111,
                0b00000000,
                0b11111111,
                0b11111111,
                0b00000000,
                0b11100111,
                0b11100111,
            ],
            // 八卦：兑
            7 => [
                0b11100111,
                0b11100111,
                0b00000000,
                0b11111111,
                0b11111111,
                0b00000000,
                0b11111111,
                0b11111111,
            ],
            // 八卦：巽
            8 => [
                0b11111111,
                0b11111111,
                0b00000000,
                0b11111111,
                0b11111111,
                0b00000000,
                0b11100111,
                0b11100111,
            ],
            _ => [0; 8],
        }
    }

    fn random(rng: Rng) -> [u8; 8] {
        let num = CubeRng(rng.random() as u64).random(1, 9_u32) as u8;
        Self::bagua(num)
    }

    pub async fn run(app: &mut App<'_>) {
        app.ledc.clear();
        loop {
            let accel = app.accel();
            if (accel.x() > 0.3 || accel.x() < -0.3)
                && (accel.y() > 0.3 || accel.y() < -0.3)
                && (0..30)
                    .map(|_| (app.accel().x(), app.accel().y()))
                    .any(|(x, y)| !(-0.3..=0.3).contains(&x) && !(-0.3..=0.3).contains(&y))
            {
                app.ledc.write_bytes(Self::random(app.rng));
                app.buzzer.bagua().await;
            }
            Timer::after_millis(800).await;

            if app.quit() {
                break;
            }
        }
    }
}
