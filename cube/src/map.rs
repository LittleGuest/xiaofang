use core::fmt::Display;

use alloc::vec::Vec;
use embedded_graphics::{geometry::Point, pixelcolor::Rgb888, Pixel};

use crate::Gd;

pub type MapCell<T = ()> = (Pixel<Rgb888>, T);

/// 地图
#[derive(Debug, Default)]
pub struct Map<T> {
    /// 宽度
    pub width: usize,
    /// 长度
    pub height: usize,
    /// 地图数据
    pub data: Vec<MapCell<T>>,
}

impl<T> Map<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: Vec::new(),
        }
    }
}

/// 视野
#[derive(Debug)]
pub struct Vision<T> {
    // /// 宽度
    // width: usize,
    // /// 长度
    // height: usize,
    /// 视野左上角坐标
    pub pos: Point,
    /// 视野数据
    pub data: Vec<MapCell<T>>,
}

impl<T> Display for Vision<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:?}", self.pos)?;
        let mut data = self
            .data
            .iter()
            .map(|c| (c.0 .0.x, c.0 .0.y))
            .collect::<Vec<_>>();
        data.sort_by_key(|c| c.0);
        writeln!(f, "{data:?}")
    }
}

impl<T: Clone> Vision<T> {
    /// 初始化视野
    pub fn new(width: usize, height: usize, player: Point) -> Self {
        Self {
            pos: {
                let x = {
                    if player.x - 3 <= 0 || width < 8 {
                        0
                    } else if player.x + 5 >= width as i32 {
                        player.x - 8 + width as i32 - player.x
                    } else {
                        player.x - 3
                    }
                };
                let y = {
                    if player.y - 3 <= 0 || height < 8 {
                        0
                    } else if player.y + 5 >= height as i32 {
                        player.y - 8 + height as i32 - player.y
                    } else {
                        player.y - 3
                    }
                };
                Point::new(x, y)
            },
            data: Vec::new(),
        }
    }

    /// 视野下一个位置
    pub fn next_pos(&self, gd: Gd) -> Point {
        let mut pos = self.pos;
        match gd {
            Gd::None => {}
            Gd::Up => pos.y -= 1,
            Gd::Right => pos.x += 1,
            Gd::Down => pos.y += 1,
            Gd::Left => pos.x -= 1,
        };
        pos
    }

    /// 移动视野
    pub fn r#move(&mut self, gd: Gd) {
        self.pos = self.next_pos(gd);
    }

    /// 根据视野位置更新视野数据
    pub fn update_data(&mut self, map: &Map<T>) {
        let Point { x, y } = self.pos;
        let data = map.data.iter();
        if map.width < 8 && map.height < 8 {
            self.data.clone_from(&map.data);
        } else if map.width < 8 {
            self.data = data
                .filter(|d| d.0 .0.y >= y && d.0 .0.y < y + 8)
                .cloned()
                .collect::<Vec<_>>();
        } else if map.height < 8 {
            self.data = data
                .filter(|d| d.0 .0.x >= x && d.0 .0.x < x + 8)
                .cloned()
                .collect::<Vec<_>>();
        } else {
            self.data = data
                .filter(|d| d.0 .0.x >= x && d.0 .0.x < x + 8 && d.0 .0.y >= y && d.0 .0.y < y + 8)
                .cloned()
                .collect::<Vec<_>>();
        }
    }

    /// 改变视野位置
    pub fn update(&mut self, gd: Gd, map: &Map<T>) {
        let Point { x, y } = self.next_pos(gd);
        let overlapping = {
            if map.width < 8 && map.height < 8 {
                true
            } else if map.width < 8 {
                x < 0 || y < 0 || x > map.width as i32 - 7 || y >= map.height as i32 - 7
            } else if map.height < 8 {
                x < 0 || y < 0 || x >= map.width as i32 - 7 || y > map.height as i32 - 7
            } else {
                x < 0 || y < 0 || x >= map.width as i32 - 7 || y >= map.height as i32 - 7
            }
        };
        if overlapping {
            return;
        }
        self.r#move(gd);
        self.update_data(map);
    }
}
