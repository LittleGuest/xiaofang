#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![allow(unused)]

use alloc::vec::Vec;
use core::f32::consts::PI;
use core::intrinsics::roundf32;
use core::mem::MaybeUninit;
use cube::buzzer::Buzzer;
use cube::ledc::LedControl;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::clock::Clocks;
use esp_hal::delay::Delay;
use esp_hal::gpio::Io;
use esp_hal::ledc::{LSGlobalClkSource, Ledc};
use esp_hal::peripherals::ADC1;
use esp_hal::spi::master::Spi;
use esp_hal::spi::SpiMode;
use esp_hal::system::SystemControl;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*};
use log::info;
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;
use spectrum_analyzer::scaling::{self, divide_by_N_sqrt};
use spectrum_analyzer::windows::{hamming_window, hann_window};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

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

pub static mut CLOCKS: MaybeUninit<Clocks> = MaybeUninit::uninit();

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    unsafe { CLOCKS.write(clocks) };
    let clocks = unsafe { CLOCKS.assume_init_ref() };

    let mut delay = Delay::new(clocks);
    init_heap();
    esp_println::logger::init_logger_from_env();
    let _timer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let tg0 = TimerGroup::new_async(peripherals.TIMG0, clocks);

    // let _init = esp_wifi::initialize(
    //     esp_wifi::EspWifiInitFor::Wifi,
    //     timer,
    //     esp_hal::rng::Rng::new(peripherals.RNG),
    //     peripherals.RADIO_CLK,
    //     &clocks,
    // )
    // .unwrap();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    esp_hal_embassy::init(clocks, tg0);

    let mut ledc = Ledc::new(peripherals.LEDC, clocks);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let buzzer = Buzzer::new(io.pins.gpio11, ledc, spawner);
    unsafe { cube::BUZZER.write(buzzer) };

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        1_000u32.kHz(),
        clocks,
        None,
    );
    let mut mpu = Mpu6050::new(i2c, Address::default()).unwrap();
    mpu.initialize_dmp(&mut delay).unwrap();

    let spi =
        Spi::new(peripherals.SPI2, 3_u32.MHz(), SpiMode::Mode0, clocks).with_mosi(io.pins.gpio3);
    let ledc = LedControl::new(spi);

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    unsafe { cube::RNG.write(rng) };

    let mut adc1_config = AdcConfig::new();
    let mut adc1_pin = adc1_config.enable_pin(io.pins.gpio1, Attenuation::Attenuation11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    let mut samples = alloc::vec![0.0;64];
    loop {
        for sample in samples.iter_mut().take(64) {
            let data = nb::block!(adc1.read_oneshot(&mut adc1_pin)).unwrap();
            // *sample = data as f32 * 1.1 / 4095.;
            *sample = data as f32;
            Timer::after_micros(250).await;
        }
        // let hann_window = hamming_window(&samples);
        let hann_window = hann_window(&samples);
        let spectrum_hann_window = samples_fft_to_spectrum(
            &hann_window,
            8000,
            // FrequencyLimit::Min(80.),
            FrequencyLimit::All,
            Some(&scaling::divide_by_N_sqrt),
        )
        .unwrap();

        info!("ADC reading: {samples:?}");

        // 频率（Hz），频谱中的频率值（幅度）
        for (fr, fr_val) in spectrum_hann_window.data().iter() {
            let maped = map_range(fr.val(), 80., 4000., 0., 8.);
            info!("{fr}Hz => {fr_val}, maped: {maped},round: {:?}", unsafe {
                roundf32(maped)
            });
        }
        Timer::after_millis(1000).await;
    }

    // cube::App::new(mpu, ledc, spawner).run().await;
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
