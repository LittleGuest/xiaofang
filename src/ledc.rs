use embedded_hal::digital::v2::OutputPin;
use max7219::{
    connectors::{Connector, PinConnector},
    MAX7219,
};

use crate::Gd;

pub struct LedControl<Connector> {
    /// 初级显存
    pub buf_work: [u8; 8],
    /// 最终上传的数据
    buf: [u8; 8],
    pub gd: Gd,
    led: MAX7219<Connector>,
}

impl<DATA, CS, CLK> LedControl<PinConnector<DATA, CS, CLK>>
where
    DATA: OutputPin,
    CS: OutputPin,
    CLK: OutputPin,
{
    pub fn new(data: DATA, cs: CS, clk: CLK) -> Self {
        let mut max7219 = MAX7219::from_pins(4, data, cs, clk).unwrap();
        max7219.power_on().unwrap();
        max7219.clear_display(0).unwrap();

        Self {
            buf_work: [0; 8],
            buf: [0; 8],
            gd: Gd::default(),
            led: max7219,
        }
    }
}

impl<C> LedControl<C>
where
    C: Connector,
{
    pub fn shutdown(&mut self) {
        self.led.power_off();
    }

    // 设置亮度，亮度范围0x00-0x0F(0-15)
    pub fn set_intensity(&mut self, intensity: u8) {
        self.led.set_intensity(0, intensity);
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

    pub fn upload(&mut self) {
        if let Err(e) = self.led.write_raw(0, &self.buf) {
            log::error!("{e:?}");
        }
    }

    pub fn upload_raw(&mut self, raw: [u8; 8]) {
        if let Err(e) = self.led.write_raw(0, &raw) {
            log::error!("{e:?}");
        }
    }

    /// 设置指定坐标单个led的亮灭
    pub fn set_led_work(&mut self, x: u8, y: u8, on: bool) {
        if x > 7 || y > 7 {
            return;
        }
        let y = y as usize;
        if on {
            self.buf_work[y] = self.buf_work[y] | (1 << (7 - x));
        } else {
            self.buf_work[y] = self.buf_work[y] & (!(1 << (7 - x)));
        }
    }

    /// 设置指定坐标单个led的亮灭
    pub fn set_led(&mut self, x: u8, y: u8, on: bool) {
        if x > 7 || y > 7 {
            return;
        }
        let y = y as usize;
        if on {
            self.buf[y] = self.buf[y] | (1 << (7 - x));
        } else {
            self.buf[y] = self.buf[y] & (!(1 << (7 - x)));
        }
    }

    // /// 获取缓存里指定坐标的状态，返回1或者0
    // /// 判断在后级缓存里指定坐标像素的状态
    // /// 这个用于判断用于上传的显存里指定坐标像素的状态
    // pub fn get_led_state(&mut self, x: u8, y: u8) -> u8 {
    //     let y = y as usize;
    //     let mut x_b = 0;
    //     if x >= 0 && x <= 7 && y >= 0 && y <= 7 {
    //         x_b = self.buf[y] & (1 << (7 - x));
    //     } else {
    //         return 3;
    //     } // 如果参数不合法就返回3

    //     if x_b == 1 {
    //         return 1;
    //     } else {
    //         return 0;
    //     }
    // }

    /// 判断在外部缓存里指定坐标像素的状态
    /// 获取外部显存数组指定坐标的状态,这个用于像素互动判断
    pub fn get_led_state_work(&mut self, x: u8, y: u8, view: [u8; 8]) -> u8 {
        let state = if x <= 7 && y <= 7 {
            view[y as usize] & (1 << (7 - x))
        } else {
            3
        };

        match state {
            1 => 1,
            _ => 0,
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
        match self.gd {
            Gd::None => {}
            Gd::Up => {
                // 上方朝下时
                for j in 0..8 {
                    self.buf[j] = 0;
                    for i in 0..8 {
                        self.buf[j] |= ((buf[7 - j] >> i) & 1) << (7 - i);
                    }
                }
            }
            Gd::Right => {
                // 右方朝下时ok
                for i in 0..8 {
                    self.buf[7 - i as usize] = 0;
                    for j in 0..8 {
                        self.buf[7 - i] |= ((buf[j] & (0b10000000 >> i)) << i) >> j;
                    }
                }
            }
            Gd::Down => {
                // 下方朝下时【默认方向，就直接写入，不做变换】ok
                for i in 0..8 {
                    self.buf[i] = buf[i];
                }
            }
            Gd::Left => {
                // 左方朝下时ok
                for i in 0..8 {
                    self.buf[7 - i] = 0;
                    for j in 0..8 {
                        self.buf[7 - i] |= ((buf[7 - j] & (0b00000001 << i)) >> i) << (7 - j);
                    }
                }
            }
        }
    }

    /// 按某个方向滚动
    /// 把后级缓存里的内容按某个方向滚动一格
    pub fn roll(&mut self, gd: &Gd) {
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
                (0..8).for_each(|i| self.buf[i] = self.buf[i] >> 1);
            }
            Gd::Down => {
                (1..7).rev().for_each(|i| self.buf[i] = self.buf[i - 1]);
                self.buf[0] = 0;
            }
            Gd::Left => {
                (0..8).for_each(|i| self.buf[i] = self.buf[i] << 1);
            }
        }
    }
}
