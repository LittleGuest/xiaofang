#![no_std]
#![no_main]

use cube::buzzer::Buzzer;
use cube::ledc::LedControl;
use esp_backtrace as _;
use hal::ledc::{channel, timer, LSGlobalClkSource, LowSpeed, LEDC};
use hal::spi::master::Spi;
use hal::spi::SpiMode;
use hal::{clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, Delay, IO};
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    esp_println::logger::init_logger_from_env();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

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

    let buzzer = Buzzer::new(ledc, delay);

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
    );
    let mut mpu = Mpu6050::new(i2c, Address::default()).unwrap();
    mpu.initialize_dmp(&mut delay).unwrap();
    // let parameters = CalibrationParameters::new(
    //     AccelFullScale::G2,
    //     GyroFullScale::Deg2000,
    //     ReferenceGravity::Zero,
    // );
    // let (accel_offset, gyro_offset) = calibrate(&mut mpu, &mut delay, &parameters).unwrap();

    let spi = Spi::new_mosi_only(
        peripherals.SPI2,
        io.pins.gpio3,
        3_u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    );
    let ledc = LedControl::new(delay, spi);

    let rng = hal::Rng::new(peripherals.RNG);
    unsafe { cube::RNG.write(rng) };

    cube::init();
    cube::App::new(delay, mpu, ledc, buzzer).run()
}
