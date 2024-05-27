#![no_std]
#![no_main]
#![allow(unused)]

use core::mem::MaybeUninit;
use cube::buzzer::Buzzer;
use cube::ledc::LedControl;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::primitives::{
    Circle, Line, Primitive, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle,
};
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;
use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::IO;
use esp_hal::ledc::channel::config::PinConfig;
use esp_hal::ledc::{channel, timer, LSGlobalClkSource, LowSpeed, LEDC};
use esp_hal::spi::master::Spi;
use esp_hal::spi::SpiMode;
use esp_hal::timer::TimerGroup;
use esp_hal::{clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*};
use log::info;
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;
use smart_leds::RGB8;
use smart_leds_matrix::layout::Rectangular;
use smart_leds_matrix::SmartLedMatrix;
use ws2812_spi::Ws2812;

extern crate alloc;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    init_heap();
    esp_println::logger::init_logger_from_env();
    let _timer = esp_hal::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let tg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    esp_hal::embassy::init(&clocks, tg0);

    let mut ledc = LEDC::new(peripherals.LEDC, &clocks);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let buzzer = Buzzer::new(ledc, io.pins.gpio11.into_push_pull_output());

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        1_000u32.kHz(),
        &clocks,
        None,
    );
    let mut mpu = Mpu6050::new(i2c, Address::default()).unwrap();
    mpu.initialize_dmp(&mut delay).unwrap();

    let spi =
        Spi::new(peripherals.SPI2, 3_u32.MHz(), SpiMode::Mode0, &clocks).with_mosi(io.pins.gpio3);

    let ledc = LedControl::new(spi);

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    unsafe { cube::RNG.write(rng) };

    cube::App::new(mpu, ledc, buzzer, spawner).run().await;
}
