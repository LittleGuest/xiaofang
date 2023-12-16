use max7219::connectors::Connector;
use rand::Rng;

use crate::{delay_ms, App};

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
        let gua = match num {
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
        };
        gua
    }

    fn random() -> [u8; 8] {
        let num = rand::thread_rng().gen_range(1..=8);
        Self::bagua(num)
    }

    pub fn run<C: Connector>(app: &mut App<C>) {
        app.ledc.clear();
        loop {
            let accel = app.accel();
            if accel.x().abs() > 0.3 && accel.y().abs() > 0.3 {
                if (0..30)
                    .map(|_| (app.accel().x().abs(), app.accel().y().abs()))
                    .any(|(x, y)| x > 0.3 && y > 0.3)
                {
                    app.ledc.bitmap(Self::random());
                    app.ledc.upload();
                }
            }
            delay_ms(800);
        }
    }
}
