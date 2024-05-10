#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use cube::buzzer::Buzzer;
use cube::ledc::LedControl;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::IO;
use esp_hal::ledc::{channel, timer, LSGlobalClkSource, LowSpeed, LEDC};
use esp_hal::rng::Rng;
use esp_hal::spi::master::Spi;
use esp_hal::spi::SpiMode;
use esp_hal::timer::TimerGroup;
use esp_hal::Blocking;
use esp_hal::{clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*};
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;

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
    let timer = esp_hal::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let tg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    esp_hal::embassy::init(&clocks, tg0);

    // let _init = esp_wifi::initialize(
    //     esp_wifi::EspWifiInitFor::Wifi,
    //     timer,
    //     esp_hal::rng::Rng::new(peripherals.RNG),
    //     system.radio_clock_control,
    //     &clocks,
    // )
    // .unwrap();

    let mut ledc = LEDC::new(peripherals.LEDC, &clocks);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    // 定时器配置:指定 PWM 信号的频率和占空比分辨率
    let mut lstimer0 = ledc.get_timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: 24_u32.kHz(),
        })
        .unwrap();
    // 通道配置:绑定定时器和输出 PWM 信号的 GPIO
    let mut channel0 = ledc.get_channel(
        channel::Number::Channel0,
        io.pins.gpio8.into_push_pull_output(),
    );
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();
    // 改变 PWM 信号:输出 PWM 信号来驱动
    channel0.set_duty(0).unwrap();

    let buzzer = Buzzer::new(ledc);

    // loop {
    // channel0.set_duty(0).unwrap();
    // delay.delay_ms(2000_u32);
    // channel0.set_duty(0).unwrap();
    // delay.delay_ms(2000_u32);
    // channel0.start_duty_fade(0, 100, 1000).unwrap();
    // while channel0.is_duty_fade_running() {}
    // channel0.start_duty_fade(100, 0, 1000).unwrap();
    // while channel0.is_duty_fade_running() {}
    // }

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
    // let parameters = CalibrationParameters::new(
    //     AccelFullScale::G2,
    //     GyroFullScale::Deg2000,
    //     ReferenceGravity::Zero,
    // );
    // let (accel_offset, gyro_offset) = calibrate(&mut mpu, &mut delay, &parameters).unwrap();

    let spi =
        Spi::new(peripherals.SPI2, 3_u32.MHz(), SpiMode::Mode0, &clocks).with_mosi(io.pins.gpio3);

    let ledc = LedControl::new(spi);

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    unsafe { cube::RNG.write(rng) };

    cube::App::new(mpu, ledc, buzzer).run().await
}
