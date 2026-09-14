#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use cube::{buzzer::Buzzer, ledc::LedControl};
use defmt::{error, info};
use embassy_executor::Spawner;
use esp_hal::{
    Blocking,
    analog::adc::{Adc, AdcCalLine, AdcConfig, Attenuation},
    clock::CpuClock,
    i2c,
    i2c::master::I2c,
    ledc::{LSGlobalClkSource, Ledc},
    rmt::Rmt,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_hal_smartled::{RmtSmartLeds, WS2812_TIMING, buffer_size, color_order::Rgb};
use esp_println as _;
use esp_radio::ble::controller::BleConnector;
use esp_storage::FlashStorage;
use mpu6050_dmp::{address::Address, sensor::Mpu6050};
use smart_leds::RGB8;

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

    let rng = esp_hal::rng::Rng::new();
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let (mut _wifi_controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default()).expect("Failed to initialize Wi-Fi controller");

    // 取出 ESP-NOW 接口与本机 MAC，其余接口丢弃
    let esp_now = interfaces.esp_now;
    let my_mac = interfaces.station.mac_address();
    info!(
        "My MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        my_mac[0], my_mac[1], my_mac[2], my_mac[3], my_mac[4], my_mac[5]
    );
    // 双方固定同一信道（ESP-NOW 联机对打）
    esp_now.set_channel(6).ok();

    let _connector = BleConnector::new(peripherals.BT, Default::default());

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let buzzer = cube::buzzer::BUZZER_CELL.init(Buzzer::new(peripherals.GPIO11, ledc));
    cube::buzzer::start_player(spawner, buzzer);

    let Ok(i2c) = I2c::new(peripherals.I2C0, i2c::master::Config::default()) else {
        error!("初始化 I2C 失败");
        return;
    };
    let i2c = i2c.with_sda(peripherals.GPIO4).with_scl(peripherals.GPIO5);

    let Ok(mut mpu) = Mpu6050::new(i2c, Address::default()) else {
        error!("初始化Mpu6050失败");
        return;
    };
    if mpu.initialize_dmp(&mut embassy_time::Delay).is_err() {
        error!("初始化 DMP 失败");
        return;
    }

    let Ok(rmt) = Rmt::new(peripherals.RMT, Rate::from_mhz(80)) else {
        error!("初始化 RMT 失败");
        return;
    };
    let Ok(led) = RmtSmartLeds::<{ buffer_size::<RGB8>(64) }, Blocking, RGB8, Rgb>::new(
        WS2812_TIMING,
        rmt.channel0,
        peripherals.GPIO3,
    ) else {
        error!("初始化 LED 灯带失败");
        return;
    };
    let ledc = LedControl::new(led);
    let flash = FlashStorage::new(peripherals.FLASH);

    // 麦克风 ADC 采样(GPIO0 = ADC1_CH0)
    let mut adc_config = AdcConfig::new();
    let mic_pin = adc_config.enable_pin_with_cal::<_, AdcCalLine<esp_hal::peripherals::ADC1<'static>>>(
        peripherals.GPIO0,
        Attenuation::_11dB,
    );
    let adc = Adc::new(peripherals.ADC1, adc_config);

    cube::App::new(mpu, ledc, spawner, flash, rng, adc, mic_pin, esp_now, my_mac)
        .run()
        .await;
}
