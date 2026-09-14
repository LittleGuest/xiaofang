use defmt::error;
use embedded_graphics::{pixelcolor::*, prelude::*};
use esp_hal::Blocking;
use esp_hal_smartled::{
    RmtSmartLeds, buffer_size,
    color_order::{self, Grb, Rgb},
};
use heapless::Vec;
use smart_leds::RGB8;
use smart_leds_matrix::{
    SmartLedMatrix,
    layout::{Rectangular, invert_axis::NoInvert},
};

use crate::mapping;

/// led 数量
const NUM_LEDS: usize = 64;

pub struct LedControl<'d> {
    matrix: SmartLedMatrix<
        RmtSmartLeds<'d, { buffer_size::<RGB8>(NUM_LEDS) }, Blocking, RGB8, Rgb>,
        Rectangular<NoInvert>,
        NUM_LEDS,
    >,
}

impl<'d> LedControl<'d> {
    pub fn new(led: RmtSmartLeds<'d, { buffer_size::<RGB8>(NUM_LEDS) }, Blocking, RGB8, Rgb>) -> Self {
        let mut matrix = SmartLedMatrix::<_, _, { 8 * 8 }>::new(led, Rectangular::new(8, 8));
        matrix.set_brightness(1);
        matrix.clear(Rgb888::new(0, 0, 0)).unwrap();

        Self { matrix }
    }

    pub fn off(&mut self) {
        self.matrix.set_brightness(0);
    }

    // 设置亮度
    pub fn set_brightness(&mut self, b: u8) {
        self.matrix.set_brightness(b);
    }

    /// 清屏
    pub fn clear(&mut self) {
        self.write_bytes([0; 8]);
    }

    /// 清屏
    pub fn clear_with_color(&mut self, color: Rgb888) {
        if let Err(e) = self.matrix.clear(color) {
            error!("clear_with_color error {:?}", e);
        }
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

                pixels.push(Pixel((x, y as i32).into(), on_off.into())).ok();
            }
        }
        self.write_pixels(pixels);
    }

    pub fn write_pixels<I>(&mut self, pixels: I)
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        if let Err(e) = self.matrix.draw_iter(pixels) {
            error!("write pixels error: {:?}", e);
        }
        if let Err(e) = self.matrix.flush() {
            error!("write pixels error: {:?}", e);
        }
    }

    pub fn write_pixel(&mut self, pixel: Pixel<Rgb888>) {
        self.write_pixels([pixel]);
    }

    /// 绘制分数
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
