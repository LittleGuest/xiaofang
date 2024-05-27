use cube_rand::CubeRng;
use embassy_time::Timer;
use esp_hal::{
    gpio::{GpioPin, Output, PushPull},
    ledc::{
        channel::{self, config::PinConfig, Channel, ChannelIFace},
        timer::{self, TimerIFace},
        LowSpeed, LEDC,
    },
    prelude::_fugit_RateExtU32,
};
use log::info;

use crate::RNG;

/// 蜂鸣器
pub struct Buzzer<'d> {
    pub open: bool,
    pin: GpioPin<Output<PushPull>, 11>,
    ledc: LEDC<'d>,
}

impl<'d> Buzzer<'d> {
    pub fn new(ledc: LEDC<'d>, pin: GpioPin<Output<PushPull>, 11>) -> Self {
        Self {
            open: true,
            ledc,
            pin,
        }
    }

    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
    }

    pub fn change(&mut self) {
        self.open = !self.open
    }

    /// FIXME: esp_hal::ledc 暂时仅支持固定频率输出，不同频率需要重新配置定时器和通道
    fn drive(&mut self, frequency: u32, duty_pct: u8) {
        // 定时器配置:指定 PWM 信号的频率和占空比分辨率
        let mut lstimer0 = self.ledc.get_timer::<LowSpeed>(timer::Number::Timer0);
        lstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty13Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: frequency.Hz(),
            })
            .unwrap();
        // 通道配置:绑定定时器和输出 PWM 信号的 GPIO
        let mut channel0 = self
            .ledc
            .get_channel(channel::Number::Channel0, &mut self.pin);
        channel0
            .configure(channel::config::Config {
                timer: &lstimer0,
                duty_pct,
                pin_config: PinConfig::PushPull,
            })
            .unwrap();
    }

    /// 发声
    /// frequency: 发声频率,单位HZ
    /// duration: 发声时长,单位毫秒
    pub async fn tone(&mut self, frequency: u32, duration: u64) {
        self.drive(frequency, 50);
        Timer::after_millis(duration).await;
        self.no_tone();
    }

    /// 停止发声
    pub fn no_tone(&mut self) {
        self.drive(1, 0);
    }

    /// 菜单选择音效
    pub async fn menu_select(&mut self) {
        if !self.open {
            return;
        }
        self.tone(1500, 500).await;
        self.no_tone();
    }

    /// 菜单确认音效
    pub async fn menu_confirm(&mut self) {
        if !self.open {
            return;
        }
        for i in (400..2000).step_by(100) {
            self.tone(i, 50).await;
            Timer::after_millis(10).await;
        }
    }

    /// 菜单进入音效
    pub async fn menu_access(&mut self) {
        if !self.open {
            return;
        }
        for i in (200..=3000).rev().step_by(200) {
            self.tone(i, 50).await;
            Timer::after_millis(10).await;
        }
    }

    /// 八卦音效
    pub async fn bagua(&mut self) {
        if !self.open {
            return;
        }
        for i in (200..=3000).rev().step_by(400) {
            self.tone(i, 50).await;
            Timer::after_millis(10).await;
        }
    }

    /// 骰子音效
    pub async fn dice(&mut self) {
        if !self.open {
            return;
        }
        for i in (200..=3000).rev().step_by(400) {
            self.tone(i, 50).await;
            Timer::after_millis(10).await;
        }
    }

    /// 迷宫移动音效
    pub async fn maze_move(&mut self) {
        if self.open {
            self.tone(5000, 50).await;
            Timer::after_millis(50).await;
            self.no_tone();
        } else {
            Timer::after_millis(50).await;
        }
    }

    /// 休眠开启音效
    pub async fn hibernation(&mut self) {
        self.tone(8000, 100).await;
        Timer::after_millis(100).await;
        self.tone(2500, 100).await;
        Timer::after_millis(100).await;
        self.tone(800, 100).await;
        Timer::after_millis(100).await;
        self.no_tone();
    }

    /// 开机音效
    pub async fn power_on(&mut self) {
        self.tone(800, 200).await;
        Timer::after_millis(200).await;
        self.tone(2500, 100).await;
        Timer::after_millis(100).await;
        self.tone(8000, 200).await;
        Timer::after_millis(200).await;
        self.no_tone();
    }

    /// 唤醒音效
    pub async fn wakeup(&mut self) {
        self.tone(1500, 200).await;
        Timer::after_millis(200).await;
        self.tone(8000, 200).await;
        Timer::after_millis(200).await;
        self.no_tone();
    }

    /// 沙漏像素闪烁音效
    pub async fn timer_pixel_blinky(&mut self) {
        if !self.open {
            return;
        }
        self.tone(8000, 50).await;
        self.no_tone();
    }

    /// 沙漏像素反弹音效
    pub async fn timer_pixel_rebound(&mut self) {
        if self.open {
            self.tone(4000, 50).await;
            Timer::after_millis(50).await;
            self.no_tone();
        } else {
            Timer::after_millis(50).await;
        }
    }

    /// 沙漏结束音效
    pub async fn timers_over(&mut self) {
        if !self.open {
            return;
        }
        for _ in 0..3 {
            self.tone(6000, 100).await;
            // lc.bitmap(all_led_on);
            // lc.UpLoad();
            Timer::after_millis(1).await;
            // lc.clearDisplay();
            Timer::after_millis(100).await;
            self.no_tone();
            Timer::after_millis(20).await;
        }

        // lc.setIntensity(2);
        // lc.bitmap(all_led_on);
        // lc.UpLoad();
        self.tone(6000, 150).await;
        Timer::after_millis(50).await;
        // lc.clearDisplay();
        Timer::after_millis(100).await;
        self.no_tone();
        // lc.setIntensity(8);
    }

    /// 贪吃蛇得分音效
    pub async fn snake_score(&mut self) {
        if !self.open {
            return;
        }
        // lc.setIntensity(3);
        Timer::after_millis(100).await;
        self.tone(2000, 1000).await;
        Timer::after_millis(15).await;
        self.no_tone();
        // lc.setIntensity(0);
        Timer::after_millis(50).await;
        self.tone(3000, 1000).await;
        Timer::after_millis(15).await;
        self.no_tone();
        // lc.setIntensity(3);
        Timer::after_millis(25).await;
        self.tone(2000, 1000).await;
        Timer::after_millis(15).await;
        self.no_tone();
        // lc.setIntensity(0);
    }

    /// 贪吃蛇死亡音效
    pub async fn snake_die(&mut self) {
        if !self.open {
            return;
        }
        self.tone(500, 1000).await;
        Timer::after_millis(160).await;
        self.tone(300, 1000).await;
        Timer::after_millis(160).await;
        self.tone(100, 1000).await;
        Timer::after_millis(200).await;
        self.no_tone();
    }

    /// 休眠音效
    pub async fn sleep(&mut self) {
        if !self.open {
            return;
        }
        self.tone(6000, 50).await;
    }

    /// 休眠音效2
    pub async fn sleep2(&mut self) {
        if !self.open {
            return;
        }
        self.tone(
            unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64).random_range(3000..=9000) as u32
            },
            50,
        )
        .await;
    }

    /// 眨眼音效
    pub async fn blinky(&mut self) {
        if !self.open {
            return;
        }
        self.tone(8000, 50).await;
    }

    /// 眨眼音效2
    pub async fn blinky2(&mut self) {
        if !self.open {
            return;
        }
        self.tone(5000, 50).await;
    }
}
