use alloc::vec::Vec;
use cube_rand::CubeRng;
use embassy_time::Timer;

use crate::{buzzer::Buzzer, ledc::LedControl, RNG};

/// 表情
/// 左上角为坐标原点
#[derive(Debug, Default)]
pub struct Face {
    pub data: [u8; 8],
    frames: Vec<[u8; 8]>,
}

impl Face {
    pub fn set_work(&mut self, x: u8, y: u8) {
        if x > 7 || y > 7 {
            return;
        }
        self.data[y as usize] |= 1 << (7 - x);
    }

    pub fn clear_work(&mut self, x: u8, y: u8) {
        if x > 7 || y > 7 {
            return;
        }
        self.data[y as usize] ^= 1 << (7 - x);
    }

    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|r| *r = 0);
    }

    /// 呆滞眼
    pub fn slack_eyes(&mut self, x: u8, y: u8) {
        // 左眼
        self.set_work(x, 7 - y);
        self.set_work(x, 7 - y - 1);
        self.set_work(x + 1, 7 - y);
        self.set_work(x + 1, 7 - y - 1);

        // 右眼
        self.set_work(x + 4, 7 - y);
        self.set_work(x + 4, 7 - y - 1);
        self.set_work(x + 5, 7 - y);
        self.set_work(x + 5, 7 - y - 1);
    }

    /// 闭眼
    pub fn close_eyes(&mut self) {
        // 左眼
        self.set_work(0, 3);
        self.set_work(1, 3);
        self.set_work(2, 3);

        // 右眼
        self.set_work(5, 3);
        self.set_work(6, 3);
        self.set_work(7, 3);
    }

    /// 大笑眼
    pub fn laugh_eyes(&mut self) {
        // 左眼
        self.set_work(0, 3);
        self.set_work(1, 2);
        self.set_work(2, 3);

        // 右眼
        self.set_work(5, 3);
        self.set_work(6, 2);
        self.set_work(7, 3);
    }

    /// 生气眼
    pub fn angry_eyes(&mut self) {
        // 左眼
        self.set_work(1, 1);
        self.set_work(1, 3);
        self.set_work(2, 2);
        self.set_work(3, 3);

        // 右眼
        self.set_work(4, 3);
        self.set_work(5, 2);
        self.set_work(6, 1);
        self.set_work(6, 3);
    }

    /// 虚眼
    pub fn slightly_closed_eyes(&mut self) {
        // 左眼
        self.set_work(1, 4);
        self.set_work(1, 3);
        self.set_work(2, 3);
        self.set_work(0, 3);

        // 右眼
        self.set_work(6, 4);
        self.set_work(5, 3);
        self.set_work(6, 3);
        self.set_work(7, 3);
    }

    /// 呆滞嘴
    pub fn slack_mouth(&mut self) {
        self.set_work(3, 5);
        self.set_work(4, 5);
    }

    /// 无奈嘴
    pub fn powerless_mouth(&mut self) {
        self.set_work(2, 6);
        self.set_work(3, 6);
        self.set_work(4, 6);
        self.set_work(5, 6);
    }

    /// 嘟嘴
    pub fn pout_mouth(&mut self) {
        self.set_work(3, 6);
        self.set_work(3, 5);
        self.set_work(4, 6);
        self.set_work(4, 5);
    }

    /// 惊恐嘴
    pub fn terrify_mouth(&mut self) {
        self.set_work(2, 6);
        self.set_work(2, 5);
        self.set_work(3, 7);
        self.set_work(3, 4);
        self.set_work(4, 7);
        self.set_work(4, 4);
        self.set_work(5, 6);
        self.set_work(5, 5);
    }

    /// 大笑嘴
    pub fn laugh_mouth(&mut self) {
        self.set_work(3, 6);
        self.set_work(4, 6);
        self.set_work(2, 5);
        self.set_work(5, 5);
    }

    /// 生气嘴
    pub fn angry_mouth(&mut self) {
        self.set_work(3, 5);
        self.set_work(4, 5);
        self.set_work(2, 6);
        self.set_work(5, 6);
    }

    /// 呆滞表情
    pub fn slack_face(&mut self, x: u8, y: u8) {
        self.clear();
        self.slack_eyes(x, y);
        self.slack_mouth();
    }

    /// 嘟嘴表情
    pub fn pout_face(&mut self, x: u8, y: u8) {
        self.clear();
        self.slack_eyes(x, y);
        self.pout_mouth();
    }

    /// 眨眼动画
    pub async fn blink_animate<'d>(
        &mut self,
        x: u8,
        y: u8,
        ledc: &mut LedControl<'d>,
        buzzer: &mut Buzzer<'d>,
    ) {
        self.clear();
        self.close_eyes();
        self.laugh_mouth();
        ledc.write_bytes(self.data);
        Timer::after_millis(80).await;

        buzzer.tone(6000, 50);

        self.clear();
        self.slack_eyes(x, y);
        self.laugh_mouth();
        ledc.write_bytes(self.data);
        Timer::after_millis(500).await;
    }

    /// 休眠动画
    pub async fn dormancy_animate<'d>(
        &mut self,
        ledc: &mut LedControl<'d>,
        buzzer: &mut Buzzer<'d>,
    ) {
        let ex: u8 = 1;
        let ey: u8 = 4;

        // 东张西望
        self.slack_face(ex, ey);
        ledc.write_bytes(self.data);
        Timer::after_millis(500).await;

        buzzer.tone(6000, 50);

        //呆滞眼左看
        self.slack_face(ex - 1, ey);
        ledc.write_bytes(self.data);
        Timer::after_millis(400).await;

        // 眼神复位
        self.slack_face(ex, ey);
        ledc.write_bytes(self.data);
        Timer::after_millis(10).await;

        buzzer.tone(6000, 50);

        //呆滞眼右看
        self.slack_face(ex + 1, ey);
        ledc.write_bytes(self.data);
        Timer::after_millis(500).await;

        // 眼神复位
        self.slack_face(ex, ey);
        ledc.write_bytes(self.data);
        Timer::after_millis(500).await;

        //微笑
        self.clear();
        self.slack_eyes(ex, ey);
        self.laugh_mouth();
        ledc.write_bytes(self.data);
        Timer::after_millis(1000).await;

        //眨眼
        for _ in 0..3 {
            self.blink_animate(ex, ey, ledc, buzzer).await;
            self.blink_animate(ex, ey, ledc, buzzer).await;
            self.blink_animate(ex, ey, ledc, buzzer).await;
        }

        for _ in 0..6 {
            let freq =
                unsafe { CubeRng(RNG.assume_init_mut().random() as u64).random_range(3000..=9000) };
            buzzer.tone(freq as u64, 50);

            // 呆滞嘴
            self.slack_face(ex, ey);
            ledc.write_bytes(self.data);
            Timer::after_millis(100).await;

            let freq =
                unsafe { CubeRng(RNG.assume_init_mut().random() as u64).random_range(3000..=9000) };
            buzzer.tone(freq as u64, 50);

            // 嘟嘴
            self.pout_face(ex, ey);
            ledc.write_bytes(self.data);
            Timer::after_millis(200).await;
        }

        //眨眼等待
        for _ in 0..2 {
            self.blink_animate(ex, ey, ledc, buzzer).await;
            self.blink_animate(ex, ey, ledc, buzzer).await;
        }
    }

    /// 唤醒动画
    pub async fn wakeup_animate<'d>(&mut self, ledc: &mut LedControl<'d>, buzzer: &mut Buzzer<'d>) {
        let ex: u8 = 1;
        let ey: u8 = 4;

        for _ in 0..2 {
            buzzer.tone(8000, 50);
            self.clear();
            self.close_eyes();
            self.slack_mouth();
            ledc.write_bytes(self.data);
            Timer::after_millis(100).await;

            self.clear();
            self.slack_eyes(ex, ey);
            self.slack_mouth();
            ledc.write_bytes(self.data);
            Timer::after_millis(700).await;
        }
    }

    /// 破记录动画
    pub async fn break_record_animate<'d>(
        &mut self,
        ledc: &mut LedControl<'d>,
        buzzer: &mut Buzzer<'d>,
    ) {
        let ex = 1;
        let ey = 4;

        for _ in 0..3 {
            self.clear();
            self.slack_eyes(ex, ey);
            self.terrify_mouth();
            ledc.write_bytes(self.data);
            Timer::after_millis(500).await;

            buzzer.tone(8000, 50);

            self.clear();
            self.close_eyes();
            self.terrify_mouth();
            ledc.write_bytes(self.data);
            Timer::after_millis(100).await;
        }

        self.clear();
        self.slack_eyes(ex, ey);
        self.terrify_mouth();
        ledc.write_bytes(self.data);
        Timer::after_millis(700).await;

        self.clear();
        self.slack_eyes(ex, ey);
        self.laugh_mouth();
        ledc.write_bytes(self.data);
        Timer::after_millis(1000).await;
    }
}
