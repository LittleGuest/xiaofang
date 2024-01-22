use embedded_graphics::{pixelcolor::*, prelude::*};
use hal::{
    peripherals::SPI2,
    spi::{master::Spi, FullDuplexMode},
    Delay,
};
use heapless::Vec;
use smart_leds_matrix::{
    layout::{invert_axis::NoInvert, Rectangular},
    SmartLedMatrix,
};
use ws2812_spi::Ws2812;

use crate::Gd;

/// led 数量
const NUM_LEDS: usize = 64;

pub struct LedControl<'d> {
    /// 初级显存
    pub buf_work: [u8; 8],
    /// 最终上传的数据
    buf: [u8; 8],
    gd: Gd,

    /// 亮度
    _brightness: u8,
    ws: Ws2812<Spi<'d, SPI2, FullDuplexMode>>,
    matrix: SmartLedMatrix<Rectangular<NoInvert>, NUM_LEDS>,
    _delay: Delay,
}

impl<'d> LedControl<'d> {
    pub fn new(delay: Delay, spi: Spi<'d, SPI2, FullDuplexMode>) -> Self {
        let brightness = 10;

        let ws = Ws2812::new(spi);
        let mut matrix = SmartLedMatrix::<_, { 8 * 8 }>::new(Rectangular::new(8, 8));
        matrix.set_brightness(brightness);
        matrix.clear(Rgb888::new(0, 0, 0)).unwrap();

        Self {
            buf_work: [0; 8],
            buf: [0; 8],
            gd: Gd::default(),
            _brightness: brightness,
            ws,
            matrix,
            _delay: delay,
        }
    }

    pub fn shutdown(&mut self) {
        // self.led.power_off();
    }

    // 设置亮度
    pub fn set_brightness(&mut self, b: u8) {
        self.matrix.set_brightness(b)
    }

    pub fn clear_work(&mut self) {
        for i in 0..self.buf_work.len() {
            self.buf_work[i] = 0;
        }
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
        // self.clear_with_color(Rgb888::default());
        // self.delay.delay_ms(50_u32);
        self.matrix.draw_iter(pixels).unwrap();
        if let Err(e) = self.matrix.flush_with_gamma(&mut self.ws) {
            log::error!("write_rgb {e:?}");
        }
    }

    pub fn write_pixel(&mut self, pixel: Pixel<Rgb888>) {
        self.write_pixels([pixel]);
    }

    /// 设置指定坐标单个led的亮灭
    pub fn set_led_work(&mut self, x: u8, y: u8, on: bool) {
        if x > 7 || y > 7 {
            return;
        }
        let y = y as usize;
        if on {
            self.buf_work[y] |= 1 << (7 - x);
        } else {
            self.buf_work[y] &= !(1 << (7 - x));
        }
    }

    /// 设置指定坐标单个led的亮灭
    pub fn set_led(&mut self, x: u8, y: u8, on: bool) {
        if x > 7 || y > 7 {
            return;
        }
        let y = y as usize;
        if on {
            self.buf[y] |= 1 << (7 - x);
        } else {
            self.buf[y] &= !(1 << (7 - x));
        }
    }

    /// 判断在外部缓存里指定坐标像素的状态
    /// 获取外部显存数组指定坐标的状态,这个用于像素互动判断
    pub fn get_led_state_work(&self, x: u8, y: u8, view: [u8; 8]) -> bool {
        log::info!("led view {view:?}");
        if x <= 7 && y <= 7 {
            let state = view[y as usize] & (1 << (7 - x));
            log::info!("led stat {state}");
            state > 0
        } else {
            false
        }
    }

    /// 把指定画面转存到初级显存里
    /// 负责把游戏运行显存的内容拷贝到初级显存
    pub fn bitmap_work(&mut self, buf: [u8; 8]) {
        (0..8).for_each(|i| self.buf_work[i] = buf[i]);
    }

    /// 图形显示，把图形数组传递给显存数组
    /// 按指定方向变化画面防线写入后级显存，根据屏幕姿态控制显示方向，可以实现画面跟随重力自动旋转,fangxiang为上下左右对应的1342
    pub fn bitmap(&mut self, buf: [u8; 8]) {
        // 把指定画面转存到后级显存里
        // 默认重力方向为下3
        // 根据当前方向和按键输入，更新蛇头移动方向
        log::info!("ledc.gd => {:?}", self.gd);
        self.buf.copy_from_slice(&buf);
        // match self.gd {
        //     Gd::None => {}
        //     Gd::Up => {
        //         // 上方朝下时
        //         for j in 0..8 {
        //             self.buf[j] = 0;
        //             for i in 0..8 {
        //                 self.buf[j] |= ((buf[7 - j] >> i) & 1) << (7 - i);
        //             }
        //         }
        //     }
        //     Gd::Right => {
        //         // 右方朝下时ok
        //         for i in 0..8 {
        //             self.buf[7 - i as usize] = 0;
        //             for j in 0..8 {
        //                 self.buf[7 - i] |= ((buf[j] & (0b10000000 >> i)) << i) >> j;
        //             }
        //         }
        //     }
        //     Gd::Down => {
        //         // 下方朝下时【默认方向，就直接写入，不做变换】ok
        //         for i in 0..8 {
        //             self.buf[i] = buf[i];
        //         }
        //     }
        //     Gd::Left => {
        //         // 左方朝下时ok
        //         for i in 0..8 {
        //             self.buf[7 - i] = 0;
        //             for j in 0..8 {
        //                 self.buf[7 - i] |= ((buf[7 - j] & (0b00000001 << i)) >> i) << (7 - j);
        //             }
        //         }
        //     }
        // }
    }

    /// 按某个方向滚动
    /// 把后级缓存里的内容按某个方向滚动一格
    fn _roll(&mut self, gd: &Gd) {
        // 左右，上下，左右速度，上下速度
        // 把后级缓存里的内容按某个方向滚动一格
        // 左右，上下，某个方向不滚动的话就设置为0
        // 定义：左滚动1，右滚动2，上滚动3，下滚动4
        match gd {
            Gd::None => {}
            Gd::Up => {
                (0..7).for_each(|i| self.buf[i] = self.buf[i + 1]);
                self.buf[7] = 0;
            }
            Gd::Right => {
                (0..8).for_each(|i| self.buf[i] >>= 1);
            }
            Gd::Down => {
                (1..7).rev().for_each(|i| self.buf[i] = self.buf[i - 1]);
                self.buf[0] = 0;
            }
            Gd::Left => {
                (0..8).for_each(|i| self.buf[i] <<= 1);
            }
        }
    }
}
