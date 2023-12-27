use alloc::vec::Vec;

use crate::{App, Position};

/// 沙漏
#[derive(Debug, Default)]
pub struct Timer {
    pixels: Vec<Pixel>,
}

#[derive(Debug)]
struct Pixel {
    /// 初始坐标
    x: u8,
    y: u8,
    /// 保留上一次的坐标
    x_old: u8,
    y_old: u8,
    /// 当前坐标
    x_now: u8,
    y_now: u8,

    /// 下面两个参数记录当前的速度和速度方向
    /// 两个轴向的速度
    speed_x_now: f32,
    speed_y_now: f32,
    /// 两个轴向的初始速度
    speed_x: f32,
    speed_y: f32,
    moved: bool,
}

impl Pixel {
    fn new(x: u8, y: u8, speed_x: f32, speed_y: f32) -> Self {
        Self {
            x: x,
            y: y,
            x_old: x,
            y_old: y,
            x_now: x,
            y_now: y,
            speed_x_now: speed_x,
            speed_y_now: speed_y,
            speed_x,
            speed_y,
            moved: false,
        }
    }

    fn blink<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        // if (beeper) {
        //     tone(10, 8000, 50);
        // }
        app.ledc.set_led_work(self.x, self.y, false);
        app.ledc.bitmap(app.ledc.buf_work);
        app.ledc.upload();
        // delay_ms(50);
        // if (beeper) {
        //     noTone(10);
        // }
        app.ledc.set_led_work(self.x, self.y, true);
        app.ledc.bitmap(app.ledc.buf_work);
        app.ledc.upload();
        // delay_ms(100);

        app.ledc.set_led_work(self.x, self.y, false);
        app.ledc.bitmap(app.ledc.buf_work);
        app.ledc.upload();
        // delay_ms(100);
    }

    fn r#move<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        if !self.moved {}
    }
}

impl Timer {
    fn init<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        app.ledc.clear_work();
        app.gravity_direction();

        // self.p = Position::new(5, 3);

        for y in 0..4 {
            for x in 0..8 {
                app.ledc.set_led_work(x, y, true);
            }
        }
        app.ledc.bitmap(app.ledc.buf_work);
        app.ledc.upload();

        // delay_ms(1000);

        // 闪烁三次配音效后开始
        (0..3).for_each(|_| {
            // TODO 音效
            app.ledc.set_intensity(0x01);
            // delay_ms(50);
            app.ledc.set_intensity(0x00);
            // delay_ms(50);
        });

        // delay_ms(1000);
    }

    pub fn run<T: hal::i2c::Instance>(app: &mut App<T>) {
        let mut timer = Self::default();
        // timer.init(app);

        // loop {
        let mut pixel = Pixel::new(5, 3, 0.0, 0.0);
        // 在点亮的像素里随机一个像素坐标,如果下方是空的就全屏闪烁然后开始下落
        for _ in 0..32 {
            let state = app
                .ledc
                .get_led_state_work(pixel.x, pixel.y, app.ledc.buf_work);
            let state_next = app
                .ledc
                .get_led_state_work(pixel.x, pixel.y + 1, app.ledc.buf_work);
            log::info!("led state {state}");
            log::info!("led state {state_next}");
            while !(state && !state_next) {
                let pos = Position::random(8, 4);
                pixel.x = pos.x as u8;
                pixel.y = pos.y as u8;
                log::info!("timer.p random => {pos:?}");
            }

            // 闪烁一下选中的像素,
            // pixel.blink(app);
            // 执行像素的下落过程
            // pixel.r#move(app);
        }
        // TODO 退出沙漏模式
        // }
    }
}
