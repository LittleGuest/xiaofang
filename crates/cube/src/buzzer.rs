use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};

use cube_rand::CubeRng;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use esp_hal::{
    gpio::{DriveMode, Level, Output, OutputConfig},
    ledc::{Ledc, LowSpeed, channel, channel::ChannelIFace as _, timer, timer::TimerIFace},
    peripherals::GPIO11,
    time::Rate,
};
use static_cell::StaticCell;

pub static BUZZER_CELL: StaticCell<Buzzer<'static>> = StaticCell::new();

/// 蜂鸣器
pub struct Buzzer<'d> {
    pin: GPIO11<'d>,
    ledc: Ledc<'d>,
}

impl<'d> Buzzer<'d> {
    pub fn new(pin: GPIO11<'d>, ledc: Ledc<'d>) -> Self {
        Self { ledc, pin }
    }

    /// FIXME: esp_hal::ledc 暂时仅支持固定频率输出，不同频率需要重新配置定时器和通道
    async fn drive(&mut self, frequency: u32, duty_pct: u8) {
        // 定时器配置:指定 PWM 信号的频率和占空比分辨率
        let mut lstimer0 = self.ledc.timer::<LowSpeed>(timer::Number::Timer0);
        // 配置失败(如频率超范围): 发声是尽力而为, 放弃本次失败不影响游戏
        if lstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty13Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: Rate::from_hz(frequency),
            })
            .is_err()
        {
            return;
        }
        // 通道配置:绑定定时器和输出 PWM 信号的 GPIO
        let config = OutputConfig::default();
        let led = Output::new(self.pin.reborrow(), Level::High, config);
        let mut channel0 = self.ledc.channel(channel::Number::Channel0, led);
        let _ = channel0.configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct,
            drive_mode: DriveMode::PushPull,
        });
    }

    /// 发声
    /// frequency: 发声频率,单位HZ
    /// duration: 发声时长,单位毫秒
    async fn tone(&mut self, frequency: u32, duration: u64) {
        self.drive(frequency, 50).await;
        Timer::after_millis(duration).await;
        if duration != 0 {
            self.no_tone().await;
        }
    }

    /// 停止发声
    async fn no_tone(&mut self) {
        self.drive(1, 0).await;
    }
}

/// 音效开关（与主任务、音效任务共享）
static OPEN: AtomicBool = AtomicBool::new(true);

/// 翻转音效开关
pub fn change() {
    OPEN.store(!OPEN.load(Ordering::Relaxed), Ordering::Relaxed);
}

/// 单音效任务的指令
#[derive(Clone)]
enum SoundCmd {
    Tone(u32, u64),
    Range(Vec<u32>, u64),
    Ranges(&'static [(u32, u64)]),
}

/// 音量关闭时丢弃音效
fn play(cmd: SoundCmd) {
    if !OPEN.load(Ordering::Relaxed) {
        return;
    }
    let _ = SOUND.try_send(cmd);
}

/// 有界队列：所有音效统一异步入队，由唯一音效任务串行播放
static SOUND: Channel<CriticalSectionRawMutex, SoundCmd, 16> = Channel::new();

/// 唯一音效任务：独占持有蜂鸣器，串行播放已入队的音效
#[embassy_executor::task]
async fn sound_player(buzzer: &'static mut Buzzer<'static>) {
    loop {
        match SOUND.receive().await {
            SoundCmd::Tone(f, d) => buzzer.tone(f, d).await,
            SoundCmd::Range(freqs, d) => {
                for f in freqs {
                    buzzer.tone(f, d).await;
                }
            }
            SoundCmd::Ranges(items) => {
                for (f, d) in items {
                    buzzer.tone(*f, *d).await;
                }
            }
        }
    }
}

/// 启动音效任务。`buzzer` 为持有蜂鸣器的 `&'static mut` 引用。
pub fn start_player(spawner: Spawner, buzzer: &'static mut Buzzer<'static>) {
    if let Ok(token) = sound_player(buzzer) {
        spawner.spawn(token);
    }
}

/// 菜单选择音效
pub async fn menu_select() {
    play(SoundCmd::Tone(1500, 300));
}

/// 菜单确认音效
pub async fn menu_confirm() {
    play(SoundCmd::Range((400..2000).step_by(100).collect(), 50));
}

/// 菜单进入音效
pub async fn menu_access() {
    play(SoundCmd::Range((200..=3000).rev().step_by(200).collect(), 50));
}

/// 八卦音效
pub async fn bagua() {
    play(SoundCmd::Range((200..=3000).rev().step_by(200).collect(), 50));
}

/// 骰子音效
pub async fn dice() {
    play(SoundCmd::Range((200..=3000).rev().step_by(400).collect(), 50));
}

/// 迷宫移动音效
pub async fn maze_move() {
    play(SoundCmd::Tone(5000, 100));
}

/// 迷宫结束音效
pub async fn maze_over() {
    play(SoundCmd::Ranges(&[(6000, 100), (6000, 100), (6000, 100), (6000, 150)]));
}

/// 休眠开启音效
pub async fn hibernation() {
    play(SoundCmd::Ranges(&[(8000, 100), (2500, 100), (800, 100)]));
}

/// 开机音效
pub async fn power_on() {
    play(SoundCmd::Ranges(&[(800, 200), (2500, 100), (8000, 200)]));
}

/// 唤醒音效
pub async fn wakeup() {
    play(SoundCmd::Ranges(&[(1500, 200), (8000, 200)]));
}

/// 沙漏像素闪烁音效
pub async fn timer_pixel_blinky() {
    play(SoundCmd::Tone(8000, 100));
}

/// 沙漏像素反弹音效
pub async fn timer_pixel_rebound() {
    play(SoundCmd::Tone(4000, 100));
}

/// 沙漏结束音效
pub async fn timers_over() {
    play(SoundCmd::Ranges(&[(6000, 100), (6000, 100), (6000, 100), (6000, 150)]));
}

/// 贪吃蛇移动音效
pub async fn snake_move() {
    play(SoundCmd::Tone(5000, 100));
}

/// 贪吃蛇得分音效
pub async fn snake_score() {
    play(SoundCmd::Ranges(&[(2000, 1000), (3000, 1000), (2000, 1000)]));
}

/// 贪吃蛇死亡音效
pub async fn snake_die() {
    play(SoundCmd::Ranges(&[(500, 1000), (300, 1000), (100, 1000)]));
}

/// 推箱子移动音效
pub async fn sokoban_move() {
    play(SoundCmd::Tone(5000, 100));
}

/// 推箱子过关音效
pub async fn sokoban_complete() {
    play(SoundCmd::Ranges(&[(2000, 200), (3000, 200), (4000, 200), (5000, 400)]));
}

/// 躲避方块移动音效
pub async fn dodge_cube_move() {
    play(SoundCmd::Tone(5000, 100));
}

/// 躲避方块得分音效
pub async fn dodge_cube_score() {
    play(SoundCmd::Ranges(&[(2000, 500), (3000, 500), (4000, 500)]));
}

/// 躲避方块死亡音效
pub async fn dodge_cube_die() {
    play(SoundCmd::Ranges(&[(500, 500), (300, 500), (100, 500)]));
}

/// 方块人移动音效
pub async fn cube_man_move() {
    play(SoundCmd::Tone(5000, 100));
}

/// 方块人得分音效
pub async fn cube_man_score() {
    play(SoundCmd::Ranges(&[(2000, 500), (3000, 500), (2000, 500)]));
}

/// 方块人死亡音效
pub async fn cube_man_die() {
    play(SoundCmd::Ranges(&[(500, 1000), (300, 1000), (100, 1000)]));
}

/// 方块人破纪录音效(上行号角)
pub async fn cube_man_record() {
    play(SoundCmd::Ranges(&[
        (2000, 100),
        (2500, 100),
        (3000, 100),
        (3500, 100),
        (4000, 300),
    ]));
}

/// 对打球配对成功音效
pub async fn pong_connect() {
    play(SoundCmd::Ranges(&[(2000, 100), (3000, 100), (4000, 100)]));
}

/// 对打球击球音效
pub async fn pong_hit() {
    play(SoundCmd::Tone(6000, 80));
}

/// 对打球得分音效
pub async fn pong_score() {
    play(SoundCmd::Ranges(&[(2000, 200), (3000, 200), (4000, 200)]));
}

/// 对打球比赛结束音效
pub async fn pong_over() {
    play(SoundCmd::Ranges(&[(4000, 200), (3000, 200), (2000, 200), (1000, 400)]));
}

/// 休眠音效
pub async fn sleep() {
    play(SoundCmd::Tone(6000, 100));
}

/// 休眠音效2
pub async fn sleep2(random: u32) {
    let freq = CubeRng(random as u64).random_range(3000..=9000) as u32;
    play(SoundCmd::Tone(freq, 100));
}

/// 眨眼音效
pub async fn blinky() {
    play(SoundCmd::Tone(8000, 100));
}

/// 眨眼音效2
pub async fn blinky2() {
    play(SoundCmd::Tone(5000, 100));
}

/// 破记录音效
pub async fn break_record_beep() {
    play(SoundCmd::Tone(8000, 50));
}
