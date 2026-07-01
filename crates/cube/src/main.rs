#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::ble::controller::BleConnector;

use defmt::error;
use esp_println as _;

use cube::buzzer::Buzzer;
use cube::ledc::LedControl;
use embassy_executor::Spawner;
use esp_hal::i2c::master::I2c;
use esp_hal::ledc::{LSGlobalClkSource, Ledc};
use esp_hal::spi::master::Spi;
use esp_hal::{i2c, spi};
use esp_storage::FlashStorage;
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(size: 64 * 1024);

    // let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    // info!("初始化 embassy");
    // // FIXME : esp_hal_embassy::init(timer0.alarm0);
    // info!("初始化 embassy 完成");

    let rng = esp_hal::rng::Rng::new();
    unsafe { cube::RNG.write(rng) };
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    let _connector = BleConnector::new(peripherals.BT, Default::default());

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let buzzer = Buzzer::new(peripherals.GPIO11, ledc, spawner);
    unsafe { cube::BUZZER.write(buzzer) };

    let i2c = I2c::new(peripherals.I2C0, i2c::master::Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO4)
        .with_scl(peripherals.GPIO5);

    let mut mpu = Mpu6050::new(i2c, Address::default()).unwrap();
    mpu.initialize_dmp(&mut embassy_time::Delay).unwrap();

    let spi = Spi::new(peripherals.SPI2, spi::master::Config::default())
        .unwrap()
        .with_mosi(peripherals.GPIO3);
    let ledc = LedControl::new(spi);
    let flash = FlashStorage::new(peripherals.FLASH);
    cube::App::new(mpu, ledc, spawner, flash).run().await;
}

fn map_range(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

// void displayUpdate(){
//   color = 0;
//   for(int i = 0; i < xres; i++){
//     for(int j = 0; j < yres; j++){
//       if(j <= Intensity[i]){                                // Light everything within the intensity range
// //        if(j%2 == 0){
// //          leds[(xres*(j+1))-i-1] = CHSV(color, 255, BRIGHTNESS);
// //        }
// //        else{
// //          leds[(xres*j)+i] = CHSV(color, 255, BRIGHTNESS);
// //        }
//         if(j>freq_block[i]){
//           freq_block[i] = min(j+1,8);
//         }
// leds[(xres*j)+i] = CHSV(color, 255, BRIGHTNESS);
//       }
//       else{                                                  // Everything outside the range goes dark
// //        if(j%2 == 0){
// //          leds[(xres*(j+1))-i-1] = CHSV(color, 255, 0);
// //        }
// //        else{
// //          leds[(xres*j)+i] = CHSV(color, 255, 0);
// //        }
//         if(j == freq_block[i]){
//           leds[(xres*j)+i] = CHSV(color, 0, BRIGHTNESS);//白色坠落点
//         }else{
//           leds[(xres*j)+i] = CHSV(color, 255, 0);
//         }
//
//       }
//     }
//     color += 255/xres;             // Increment the Hue to get the Rainbow
//
//   }
// }
