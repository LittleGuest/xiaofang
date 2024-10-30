use crate::{BUZZER, RNG};
use cube_rand::CubeRng;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{
    gpio::GpioPin,
    ledc::{
        channel::{self, config::PinConfig},
        timer, Ledc, LowSpeed,
    },
    prelude::*,
};

/// 蜂鸣器
pub struct Buzzer<'d> {
    pub open: bool,
    pin: GpioPin<11>,
    ledc: Ledc<'d>,
    spawner: Spawner,
}

impl<'d> Buzzer<'d> {
    pub fn new(pin: GpioPin<11>, ledc: Ledc<'d>, spawner: Spawner) -> Self {
        Self {
            open: true,
            ledc,
            pin,
            spawner,
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
    async fn drive(&mut self, frequency: u32, duty_pct: u8) {
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
        self.drive(frequency, 50).await;
        Timer::after_millis(duration).await;
        if duration != 0 {
            self.no_tone().await;
        }
    }

    /// 停止发声
    pub async fn no_tone(&mut self) {
        self.drive(1, 0).await;
    }

    /// 菜单选择音效
    pub async fn menu_select(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(1500, 300)).ok();
    }

    /// 菜单确认音效
    pub async fn menu_confirm(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_range_task(400..2000, 50, 100)).ok();
    }

    /// 菜单进入音效
    pub async fn menu_access(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_range_task((200..=3000).rev(), 50, 200))
            .ok();
    }

    /// 八卦音效
    pub async fn bagua(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_range_task((200..=3000).rev(), 50, 400))
            .ok();
    }

    /// 骰子音效
    pub async fn dice(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_range_task((200..=3000).rev(), 50, 400))
            .ok();
    }

    /// 迷宫移动音效
    pub async fn maze_move(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(5000, 100)).ok();
    }

    /// 迷宫结束音效
    pub async fn maze_over(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_ranges_task(
                [(6000, 100), (6000, 100), (6000, 100), (6000, 150)].into_iter(),
            ))
            .ok();
    }

    /// 休眠开启音效
    pub async fn hibernation(&mut self) {
        self.spawner
            .spawn(tone_ranges_task(
                [(8000, 100), (2500, 100), (800, 100)].into_iter(),
            ))
            .ok();
    }

    /// 开机音效
    pub async fn power_on(&mut self) {
        self.spawner
            .spawn(tone_ranges_task(
                [(800, 200), (2500, 100), (8000, 200)].into_iter(),
            ))
            .ok();
    }

    /// 唤醒音效
    pub async fn wakeup(&mut self) {
        self.spawner
            .spawn(tone_ranges_task([(1500, 200), (8000, 200)].into_iter()))
            .ok();
    }

    /// 沙漏像素闪烁音效
    pub async fn timer_pixel_blinky(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(8000, 100)).ok();
    }

    /// 沙漏像素反弹音效
    pub async fn timer_pixel_rebound(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(4000, 100)).ok();
    }

    /// 沙漏结束音效
    pub async fn timers_over(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_ranges_task(
                [(6000, 100), (6000, 100), (6000, 100), (6000, 150)].into_iter(),
            ))
            .ok();
    }

    /// 贪吃蛇移动音效
    pub async fn snake_move(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(5000, 100)).ok();
    }

    /// 贪吃蛇得分音效
    pub async fn snake_score(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_ranges_task(
                [(2000, 1000), (3000, 1000), (2000, 1000)].into_iter(),
            ))
            .ok();
    }

    /// 贪吃蛇死亡音效
    pub async fn snake_die(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_ranges_task(
                [(500, 1000), (300, 1000), (100, 1000)].into_iter(),
            ))
            .ok();
    }

    /// 推箱子移动音效
    pub async fn sokoban_move(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(5000, 100)).ok();
    }

    /// 休眠音效
    pub async fn sleep(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(6000, 100)).ok();
    }

    /// 休眠音效2
    pub async fn sleep2(&mut self) {
        if !self.open {
            return;
        }
        self.spawner
            .spawn(tone_task(
                unsafe {
                    CubeRng(RNG.assume_init_mut().random() as u64).random_range(3000..=9000) as u32
                },
                100,
            ))
            .ok();
    }

    /// 眨眼音效
    pub async fn blinky(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(8000, 100)).ok();
    }

    /// 眨眼音效2
    pub async fn blinky2(&mut self) {
        if !self.open {
            return;
        }
        self.spawner.spawn(tone_task(5000, 100)).ok();
    }
}

#[embassy_executor::task]
async fn tone_task(frequency: u32, duration: u64) {
    let buzzer = unsafe { BUZZER.assume_init_mut() };
    buzzer.tone(frequency, duration).await;
}

#[embassy_executor::task]
async fn tone_range_task(
    freq_range: impl Iterator<Item = u32> + 'static,
    duration: u64,
    step: usize,
) {
    let buzzer = unsafe { BUZZER.assume_init_mut() };
    for i in freq_range.step_by(step) {
        buzzer.tone(i, duration).await;
    }
}

#[embassy_executor::task]
async fn tone_ranges_task(range: impl Iterator<Item = (u32, u64)> + 'static) {
    let buzzer = unsafe { BUZZER.assume_init_mut() };
    for (freq, dur) in range {
        buzzer.tone(freq, dur).await;
    }
}
