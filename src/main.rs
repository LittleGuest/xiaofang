use esp_idf_svc::hal::{delay::FreeRtos, gpio::PinDriver, i2c::I2cDriver, prelude::Peripherals};
use mpu6050_dmp::{address::Address, sensor::Mpu6050};
use xiaofang::ledc::LedControl;

struct Delay;

impl embedded_hal::blocking::delay::DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        FreeRtos::delay_ms(ms);
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let data = PinDriver::output(peripherals.pins.gpio8).unwrap();
    let cs = PinDriver::output(peripherals.pins.gpio7).unwrap();
    let clk = PinDriver::output(peripherals.pins.gpio2).unwrap();

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio4,
        peripherals.pins.gpio5,
        &Default::default(),
    )?;

    let mut delay = Delay;
    let mut mpu = Mpu6050::new(i2c, Address::default()).unwrap();
    mpu.initialize_dmp(&mut delay).unwrap();
    // let parameters = CalibrationParameters::new(
    //     AccelFullScale::G2,
    //     GyroFullScale::Deg2000,
    //     ReferenceGravity::Zero,
    // );
    // let (accel_offset, gyro_offset) = calibrate(&mut mpu, &mut delay, &parameters).unwrap();

    let mut ledc = LedControl::new(data, cs, clk);
    ledc.set_intensity(0x01);

    xiaofang::App::new(mpu, ledc).run()
}
