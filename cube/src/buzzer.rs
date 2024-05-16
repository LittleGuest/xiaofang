use cube_rand::CubeRng;
use embassy_time::Timer;
use esp_hal::{
    gpio::{GpioPin, Output, PushPull},
    ledc::{
        channel::{Channel, ChannelIFace},
        LowSpeed,
    },
};

use crate::RNG;

/// 蜂鸣器
pub struct Buzzer<'d> {
    pub open: bool,
    // ledc: LEDC<'d>,
    channel: Channel<'d, LowSpeed, GpioPin<Output<PushPull>, 8>>,
}

impl<'d> Buzzer<'d> {
    pub fn new(
        // ledc: LEDC<'d>,
        channel: Channel<'d, LowSpeed, GpioPin<Output<PushPull>, 8>>,
    ) -> Self {
        Self {
            open: true,
            // ledc,
            channel,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// 发声
    /// frequency: 发声频率,单位HZ
    /// duration: 发声时长,负数表示一直发声,单位微妙
    pub fn tone(&mut self, frequency: u64, duration: u64) {
        if !self.open {
            return;
        }

        self.channel
            .start_duty_fade(0, 100, duration as u16)
            .unwrap();
        while self.channel.is_duty_fade_running() {}
    }

    ///停止发声
    pub fn no_tone(&mut self) {
        self.close();
    }

    /// 菜单选择音效
    pub async fn menu_select(&mut self) {
        if self.open {
            // self.tone(1500, 100);
            self.tone(1500, 500);
        }
        Timer::after_millis(200).await;
    }

    /// 菜单选择音效2
    pub async fn menu_select_2(&mut self) {
        if !self.open {
            return;
        }
        self.tone(100, 50);
        self.no_tone();
        Timer::after_millis(50).await;
    }

    /// 菜单确认音效
    pub async fn menu_confirm(&mut self) {
        if !self.open {
            return;
        }
        for i in (400..2000).step_by(100) {
            self.tone(i, 50);
            Timer::after_millis(10).await;
        }
    }

    /// 菜单进入音效
    pub async fn menu_access(&mut self) {
        if !self.open {
            return;
        }
        for i in (200..=3000).rev().step_by(200) {
            self.tone(i, 50);
            Timer::after_millis(10).await;
        }
    }

    /// 八卦音效
    pub async fn bagua(&mut self) {
        if !self.open {
            return;
        }
        for i in (200..=3000).rev().step_by(400) {
            self.tone(i, 50);
            Timer::after_millis(10).await;
        }
    }

    /// 迷宫移动音效
    pub async fn maze_move(&mut self) {
        if self.open {
            self.tone(5000, 50);
            Timer::after_millis(50).await;
            self.no_tone();
        } else {
            Timer::after_millis(50).await;
        }
    }

    /// 休眠开启音效
    pub async fn hibernation(&mut self) {
        self.tone(8000, 100);
        Timer::after_millis(100).await;
        self.tone(2500, 100);
        Timer::after_millis(100).await;
        self.tone(800, 100);
        Timer::after_millis(100).await;
        self.no_tone();
    }

    /// 开机音效
    pub async fn power_on(&mut self) {
        self.tone(800, 200);
        Timer::after_millis(200).await;
        self.tone(2500, 100);
        Timer::after_millis(100).await;
        self.tone(8000, 200);
        Timer::after_millis(200).await;
        self.no_tone();
    }

    /// 唤醒音效
    pub async fn wakeup(&mut self) {
        self.tone(1500, 200);
        Timer::after_millis(200).await;
        self.tone(8000, 200);
        Timer::after_millis(200).await;
        self.no_tone();
    }

    /// 沙漏像素闪烁音效
    pub async fn timer_pixel_blinky(&mut self) {
        if !self.open {
            return;
        }
        self.tone(8000, 50);
        Timer::after_millis(50).await;
        self.no_tone();
    }

    /// 沙漏像素反弹音效
    pub async fn timer_pixel_rebound(&mut self) {
        if self.open {
            self.tone(4000, 50);
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
            self.tone(6000, 100);
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
        self.tone(6000, 150);
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
        self.tone(2000, 1000);
        Timer::after_millis(15).await;
        self.no_tone();
        // lc.setIntensity(0);
        Timer::after_millis(50).await;
        self.tone(3000, 1000);
        Timer::after_millis(15).await;
        self.no_tone();
        // lc.setIntensity(3);
        Timer::after_millis(25).await;
        self.tone(2000, 1000);
        Timer::after_millis(15).await;
        self.no_tone();
        // lc.setIntensity(0);
    }

    /// 贪吃蛇死亡音效
    pub async fn snake_die(&mut self) {
        if !self.open {
            return;
        }
        self.tone(500, 1000);
        Timer::after_millis(160).await;
        self.tone(300, 1000);
        Timer::after_millis(160).await;
        self.tone(100, 1000);
        Timer::after_millis(200).await;
        self.no_tone();
    }

    /// 休眠音效
    pub fn sleep(&mut self) {
        if !self.open {
            return;
        }
        self.tone(6000, 50);
    }

    /// 休眠音效2
    pub fn sleep2(&mut self) {
        if !self.open {
            return;
        }
        self.tone(
            unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64).random_range(3000..=9000) as u64
            },
            50,
        );
    }

    /// 眨眼音效
    pub fn blinky(&mut self) {
        if !self.open {
            return;
        }
        self.tone(8000, 50);
    }

    /// 眨眼音效2
    pub fn blinky2(&mut self) {
        if !self.open {
            return;
        }
        self.tone(5000, 50);
    }
}
