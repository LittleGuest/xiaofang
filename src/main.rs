use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::prelude::*;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use max7219::MAX7219;

use crate::face::Face;

pub mod face;
pub mod mapping;
pub mod maze;
pub mod snake;

pub fn delay(dur: u32) {
    FreeRtos::delay_ms(dur);
}

/// 小方
pub struct App {
    /// 蜂鸣器开关
    buzzer: bool,
    /// 重力方向
    gravity: u8,
}

impl App {
    pub fn gravity_dire() {}
}

impl App {
    pub fn new() -> Self {
        App {
            buzzer: true,
            gravity: 3,
        }
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let data = PinDriver::output(peripherals.pins.gpio4).unwrap();
    let cs = PinDriver::output(peripherals.pins.gpio7).unwrap();
    let clk = PinDriver::output(peripherals.pins.gpio2).unwrap();
    let mut max7219 = MAX7219::from_pins(4, data, cs, clk).unwrap();
    max7219.power_on().unwrap();

    // (0..4).for_each(|v| {
    //     // 设置亮度
    //     max7219.set_intensity(v, 0b00000001).unwrap();
    // });

    // let _ = max7219.write_raw(0, &mapping::ALL_ON);
    // let _ = max7219.write_raw(1, &mapping::UPPER_L);
    // let _ = max7219.write_raw(2, &mapping::UPPER_K);
    // let _ = max7219.write_raw(3, &mapping::EXCLAMATION_MARK);

    // (0..4).for_each(|v| {
    //     let _ = max7219.clear_display(v);
    // });

    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default().frequency(100.Hz().into()),
    )
    .unwrap();
    let mut driver = LedcDriver::new(
        peripherals.ledc.channel0,
        timer_driver,
        peripherals.pins.gpio8,
    )
    .unwrap();

    let mut app = App::new();
    let mut face = Face::new();

    loop {
        // let _ = max7219.write_raw(0, &mapping::DICE_1);
        // FreeRtos::delay_ms(2000);
        // let _ = max7219.write_raw(0, &mapping::DICE_2);
        // FreeRtos::delay_ms(2000);
        // let _ = max7219.write_raw(0, &mapping::DICE_3);
        // FreeRtos::delay_ms(2000);
        // let _ = max7219.write_raw(0, &mapping::DICE_4);
        // FreeRtos::delay_ms(2000);
        // let _ = max7219.write_raw(0, &mapping::DICE_5);
        // FreeRtos::delay_ms(2000);
        // let _ = max7219.write_raw(0, &mapping::DICE_6);
        // FreeRtos::delay_ms(2000);

        // face.clear();
        // face.close_eyes();
        // let _ = max7219.write_raw(0, &face.ram);
        // FreeRtos::delay_ms(2000);
        //
        // face.clear();
        // face.laugh_eyes();
        // face.laugh_mouth();
        // let _ = max7219.write_raw(0, &face.ram);
        // FreeRtos::delay_ms(2000);
        //
        // face.clear();
        // face.angry_eyes();
        // face.angry_mouth();
        // let _ = max7219.write_raw(0, &face.ram);
        // FreeRtos::delay_ms(2000);
        //
        // face.clear();
        // face.slack_eyes(1, 4);
        // face.slack_mouth();
        // let _ = max7219.write_raw(0, &face.ram);
        // FreeRtos::delay_ms(2000);

        // face.clear();
        // face.slightly_closed_eyes();
        // let _ = max7219.write_raw(0, &face.ram);
        // FreeRtos::delay_ms(2000);

        // face.clear();
        // face.laugh_eyes();
        // face.terrify_mouth();
        // let _ = max7219.write_raw(0, &face.ram);
        // FreeRtos::delay_ms(2000);

        face.clear();
        face.break_record_face(&app);
        let _ = max7219.write_raw(0, &face.ram);
        FreeRtos::delay_ms(200);

        // for mut duty in 0..=255 {
        //     // driver.set_duty(driver.get_max_duty() * 3 / 4).unwrap();
        //     driver.set_duty(duty).unwrap();
        //     FreeRtos::delay_ms(500);
        // }
    }
}
