#![no_std]
#![no_main]

use esp_backtrace as _;
use hal::spi::master::Spi;
use hal::spi::SpiMode;
use hal::{clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, Delay, IO};
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;
use ws2812_spi::Ws2812;
use xiaofang::ledc::LedControl;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    esp_println::logger::init_logger_from_env();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

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

    let ws = Ws2812::new(spi);

    let mut ledc = LedControl::new(ws);
    ledc.set_intensity(0x01);

    xiaofang::init();
    xiaofang::App::new(mpu, ledc).run()
}
