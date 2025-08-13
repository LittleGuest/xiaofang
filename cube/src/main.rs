#![no_std]
#![no_main]

use cube::buzzer::Buzzer;
use cube::ledc::LedControl;
use embassy_executor::Spawner;
use esp_hal::i2c::master::I2c;
use esp_hal::ledc::{LSGlobalClkSource, Ledc};
use esp_hal::spi::master::Spi;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{i2c, spi};
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;

use defmt::info;
use esp_hal::clock::CpuClock;
use esp_println as _;

extern crate alloc;

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    info!("初始化 hal");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    info!("初始化 hal 完成");

    esp_alloc::heap_allocator!(size: 64 * 1024);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    info!("初始化 embassy");
    esp_hal_embassy::init(timer0.alarm0);
    info!("初始化 embassy 完成");

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init =
        esp_wifi::init(timer1.timer0, rng).expect("Failed to initialize WIFI/BLE controller");
    let (mut _wifi_controller, _interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");
    // let _connector = BleConnector::new(&wifi_init, peripherals.BT);

    unsafe { cube::RNG.write(rng) };

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

    cube::App::new(mpu, ledc, spawner).run().await;
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
