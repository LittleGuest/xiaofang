use embedded_graphics::{pixelcolor::*, prelude::*};
use esp_hal::{
    peripherals::SPI2,
    spi::{master::Spi, FullDuplexMode},
};
use heapless::Vec;
use smart_leds_matrix::{
    layout::{invert_axis::NoInvert, Rectangular},
    SmartLedMatrix,
};
use ws2812_spi::Ws2812;

use crate::mapping;

/// led 数量
const NUM_LEDS: usize = 64;

pub struct LedControl<'d> {
    /// 最终上传的数据
    buf: [u8; 8],
    // gd: Gd,
    /// 亮度
    // brightness: u8,
    ws: Ws2812<Spi<'d, SPI2, FullDuplexMode>>,
    matrix: SmartLedMatrix<Rectangular<NoInvert>, NUM_LEDS>,
}

impl<'d> LedControl<'d> {
    pub fn new(spi: Spi<'d, SPI2, FullDuplexMode>) -> Self {
        let brightness = 10;

        let ws = Ws2812::new(spi);
        let mut matrix = SmartLedMatrix::<_, { 8 * 8 }>::new(Rectangular::new(8, 8));
        matrix.set_brightness(brightness);
        matrix.clear(Rgb888::new(0, 0, 0)).unwrap();

        Self {
            buf: [0; 8],
            // gd: Gd::default(),
            // brightness,
            ws,
            matrix,
        }
    }

    pub fn shutdown(&mut self) {
        // self.led.power_off();
    }

    // 设置亮度
    pub fn set_brightness(&mut self, b: u8) {
        self.matrix.set_brightness(b)
    }

    pub fn clear(&mut self) {
        for i in 0..self.buf.len() {
            self.buf[i] = 0;
        }
        self.upload();
    }

    pub fn clear_with_color(&mut self, color: Rgb888) {
        if let Err(e) = self.matrix.clear(color) {
            log::error!("clear_with_color error {e:?}");
        }
    }

    pub fn upload(&mut self) {
        self.write_bytes(self.buf);
    }

    pub fn write_bytes(&mut self, data: [u8; 8]) {
        let mut pixels = Vec::<Pixel<Rgb888>, NUM_LEDS>::new();
        for (y, _) in data.iter().enumerate() {
            for x in 0..8 {
                let on_off = if data[y] & (1 << (7 - x)) > 0 {
                    BinaryColor::On
                } else {
                    BinaryColor::Off
                };

                pixels
                    .push(Pixel((x, y as i32).into(), on_off.into()))
                    .unwrap();
            }
        }
        self.write_pixels(pixels);
    }

    pub fn write_pixels<I>(&mut self, pixels: I)
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        self.matrix.draw_iter(pixels).unwrap();
        if let Err(e) = self.matrix.flush_with_gamma(&mut self.ws) {
            log::error!("write_rgb {e:?}");
        }
    }

    pub fn write_pixel(&mut self, pixel: Pixel<Rgb888>) {
        self.write_pixels([pixel]);
    }

    pub fn draw_score(&mut self, score: u8) {
        self.clear();

        let dn = score / 10;
        let sn = score % 10;
        let dn = mapping::num_map(dn);
        let mut sn = mapping::num_map(sn);

        let mut buf_work = [0; 8];
        (0..8).for_each(|i| buf_work[i] = dn[i]);

        (0..8).for_each(|i| sn[i] >>= 4);
        (0..8).for_each(|i| buf_work[i] |= sn[i]);
        (0..8).for_each(|i| buf_work[i] >>= 1);

        self.write_bytes(buf_work);
    }
}
