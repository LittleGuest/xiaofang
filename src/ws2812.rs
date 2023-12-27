

use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    peripherals::{Peripherals, SPI2},
    prelude::*,
    spi::{master::Spi, FullDuplexMode, SpiMode},
    Delay, IO,
};

use smart_leds::{SmartLedsWrite, RGB, RGB8};
use ws2812_spi::Ws2812;

const NUM_LEDS: usize = 20;
const STEPS: u8 = 10;
const TOP_ROW: usize = 4;
const MID_ROW: usize = 10;
const DT: u32 = 5;
const BREATHING_MULTIPLIER: u32 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LightData {
    leds: [RGB8; NUM_LEDS],
}
impl LightData {
    fn empty() -> Self {
        Self {
            leds: [RGB8::new(0, 0, 0); NUM_LEDS],
        }
    }
    fn from_gradient(from: RGB8, to: RGB8) -> Self {
        let mut result = [RGB8::default(); NUM_LEDS];
        let r_delta = to.r as i16 - from.r as i16;
        let g_delta = to.g as i16 - from.g as i16;
        let b_delta = to.b as i16 - from.b as i16;
        for i in 0..NUM_LEDS {
            let r = (from.r + (r_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            let g = (from.g + (g_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            let b = (from.b + (b_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            result[i] = RGB8 { r, g, b };
        }
        Self::from(result)
    }
    fn get_brightness(&self) -> u8 {
        self.leds
            .iter()
            .map(|led| led.r + led.g + led.b)
            .max()
            .unwrap()
    }
    fn write_to_strip<'d>(&self, strip: &mut Ws2812<Spi<'d, SPI2, FullDuplexMode>>) {
        strip.write(self.leds.iter().cloned()).unwrap();
    }
    fn get_led(&self, index: usize) -> RGB8 {
        self.leds[index]
    }
    fn set_color_all(&mut self, color: RGB8) {
        for i in 0..NUM_LEDS {
            self.set_color(i, color);
        }
    }
    fn set_red(&mut self, index: usize, red: u8) {
        self.leds[index].r = red;
    }
    fn set_green(&mut self, index: usize, green: u8) {
        self.leds[index].g = green;
    }
    fn set_blue(&mut self, index: usize, blue: u8) {
        self.leds[index].b = blue;
    }
    fn set_color(&mut self, led: usize, color: RGB8) {
        self.leds[led] = color;
    }
    fn set_lightness_percent_all(&mut self, lightness: f32) {
        for led in 0..self.leds.len() {
            self.set_lightness_percent(lightness, led);
        }
    }
    fn set_lightness_percent(&mut self, lightness: f32, led: usize) {
        self.leds[led].r = (self.leds[led].r as f32 * lightness) as u8;
        self.leds[led].g = (self.leds[led].g as f32 * lightness) as u8;
        self.leds[led].b = (self.leds[led].b as f32 * lightness) as u8;
    }
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            leds: [RGB8::new(STEPS, STEPS, STEPS); NUM_LEDS],
        }
    }
}

impl From<[RGB8; NUM_LEDS]> for LightData {
    fn from(data: [RGB8; NUM_LEDS]) -> Self {
        Self { leds: data }
    }
}

struct Strip<'d> {
    ws: Ws2812<Spi<'d, SPI2, FullDuplexMode>>,
    data: LightData,
    brightness: u8,

    delay: Delay,
}

impl<'d> Strip<'d> {
    fn fade_into(&mut self, data: LightData) {
        while self.data != data {
            for i in 0..NUM_LEDS {
                let r_delta = self.data.get_led(i).r as i32 - data.get_led(i).r as i32;
                let g_delta = self.data.get_led(i).g as i32 - data.get_led(i).g as i32;
                let b_delta = self.data.get_led(i).b as i32 - data.get_led(i).b as i32;
                let mut r_step = (r_delta as f32 * 0.05) as u8;
                let mut g_step = (g_delta as f32 * 0.05) as u8;
                let mut b_step = (b_delta as f32 * 0.05) as u8;
                if r_step == 0 {
                    r_step = 1;
                }
                if g_step == 0 {
                    g_step = 1;
                }
                if b_step == 0 {
                    b_step = 1;
                }
                if r_delta < 0 {
                    self.data.set_red(i, self.data.get_led(i).r + r_step);
                } else if r_delta > 0 {
                    self.data.set_red(i, self.data.get_led(i).r - r_step);
                }
                if g_delta < 0 {
                    self.data.set_green(i, self.data.get_led(i).g + g_step);
                } else if g_delta > 0 {
                    self.data.set_green(i, self.data.get_led(i).g - g_step);
                }
                if b_delta < 0 {
                    self.data.set_blue(i, self.data.get_led(i).b + b_step);
                } else if b_delta > 0 {
                    self.data.set_blue(i, self.data.get_led(i).b - b_step);
                }
            }
            self.write();
            self.delay.delay(BREATHING_MULTIPLIER * 1_000_000);
        }
        self.get_brightness();
    }

    fn startup_animation(&mut self) {
        self.data = LightData::empty();
        self.write();
        for i in 0..TOP_ROW {
            self.set_color(RGB8::new(self.brightness, 0, 0), i);
            self.write();
            // self.delay.delay(5_000_000);
            self.delay.delay_ms(100_u32);
        }
        for i in TOP_ROW..MID_ROW + TOP_ROW {
            self.set_color(RGB8::new(0, self.brightness, 0), i);
            self.write();
            // self.delay.delay(5_000_000);
            self.delay.delay_ms(100_u32);
        }
        for i in MID_ROW + TOP_ROW..NUM_LEDS {
            self.set_color(RGB8::new(0, 0, self.brightness), i);
            self.write();
            // self.delay.delay(5_000_000);
            self.delay.delay_ms(100_u32);
        }
        self.delay.delay(40_000_000 * 10);
    }

    fn shutdown_animation(&mut self) {
        let mut i = NUM_LEDS;
        while i > 0 {
            i -= 1;
            self.set_color(RGB8::new(0, 0, 0), i);
            self.write();
            self.delay.delay(5_000_000);
        }
    }

    fn write(&mut self) {
        self.data.write_to_strip(&mut self.ws);
    }
    fn set_color(&mut self, color: RGB8, index: usize) {
        self.data.set_color(index, color);
        self.write();
    }
    fn set_solid(&mut self, color: RGB8) {
        self.data.set_color_all(color);
        self.write();
    }
    fn get_brightness(&mut self) {
        self.data.get_brightness();
    }
}

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let spi = Spi::new_mosi_only(
        peripherals.SPI2,
        io.pins.gpio3,
        3_u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    );

    let ws = Ws2812::new(spi);

    let mut strip = Strip {
        ws,
        data: LightData::from_gradient(RGB8::new(40, 0, 0), RGB::new(0, 0, 40)),
        brightness: 10,
        delay,
    };

    loop {
        strip.startup_animation();
        // delay.delay(1_000_000);
        delay.delay_ms(500_u32);
        strip.fade_into(LightData::from_gradient(
            RGB8::new(40, 0, 0),
            RGB::new(0, 0, 40),
        ));
        // delay.delay(DT * 40_000_000);
        delay.delay_ms(500_u32);
        strip.shutdown_animation();
    }
}
