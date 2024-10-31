#![no_std]
#![no_main]

use cube::buzzer::Buzzer;
use cube::ledc::LedControl;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Ticker, Timer};
use esp_backtrace as _;
use esp_hal::gpio::Io;
use esp_hal::i2c::I2c;
use esp_hal::ledc::{LSGlobalClkSource, Ledc};
use esp_hal::prelude::*;
use esp_hal::rng::Rng;
use esp_hal::spi::master::Spi;
use esp_hal::spi::SpiMode;
use esp_hal::timer::systimer::{SystemTimer, Target};
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::esp_now::{PeerInfo, BROADCAST_ADDRESS};
use esp_wifi::wifi::{
    AccessPointConfiguration, ClientConfiguration, Configuration, WifiApDevice, WifiController,
    WifiDevice, WifiEvent, WifiStaDevice, WifiState,
};
use esp_wifi::EspWifiInitFor;
use log::{error, info};
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;

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

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let systimer = SystemTimer::new(peripherals.SYSTIMER).split::<Target>();
    let rng = Rng::new(peripherals.RNG);
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_alloc::heap_allocator!(72 * 1024);

    unsafe { cube::RNG.write(rng) };

    let init = esp_wifi::init(
        EspWifiInitFor::Wifi,
        timg0.timer0,
        rng,
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();
    info!("esp-now version {}", esp_now.get_version().unwrap());

    esp_hal_embassy::init(systimer.alarm0);

    // let mut ticker = Ticker::every(Duration::from_secs(5));
    // loop {
    //     let status = esp_now.send_async(&BROADCAST_ADDRESS, b"0123456789").await;
    //     info!("Send broadcast status: {:?}", status);
    //
    //     let r = esp_now.receive_async().await;
    //     info!("Received {:?}", r);
    //     if r.info.dst_address == BROADCAST_ADDRESS {
    //         if !esp_now.peer_exists(&r.info.src_address) {
    //             esp_now
    //                 .add_peer(PeerInfo {
    //                     peer_address: r.info.src_address,
    //                     lmk: None,
    //                     channel: None,
    //                     encrypt: false,
    //                 })
    //                 .unwrap();
    //         }
    //         let status = esp_now.send_async(&r.info.src_address, b"Hello Peer").await;
    //         info!("Send hello to peer status: {:?}", status);
    //     }
    // }

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let buzzer = Buzzer::new(io.pins.gpio11, ledc, spawner);
    unsafe { cube::BUZZER.write(buzzer) };

    let i2c = I2c::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        1_000u32.kHz(),
    );
    let mut mpu = Mpu6050::new(i2c, Address::default()).unwrap();
    mpu.initialize_dmp(&mut embassy_time::Delay).unwrap();

    let spi = Spi::new(peripherals.SPI2, 3_u32.MHz(), SpiMode::Mode0).with_mosi(io.pins.gpio3);
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
