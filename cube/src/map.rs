use core::fmt::Display;

use crate::{Ad, Point};
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::Rgb888, Pixel};

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
pub struct Vision<const W: usize, const H: usize, T> {
    /// 视野左上角坐标
    pub pos: Point,
    /// 视野数据
    pub data: Vec<MapCell<T>>,
}

impl<const W: usize, const H: usize, T> Display for Vision<W, H, T> {
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

impl<const W: usize, const H: usize, T: Clone> Vision<W, H, T> {
    /// 初始化视野
    pub fn new(width: usize, height: usize, player: Point) -> Self {
        Self {
            pos: {
                let x = {
                    if player.x - 3 <= 0 || width < W {
                        0
                    } else if player.x + 5 >= width as i32 {
                        player.x - W as i32 + width as i32 - player.x
                    } else {
                        player.x - 3
                    }
                };
                let y = {
                    if player.y - 3 <= 0 || height < H {
                        0
                    } else if player.y + 5 >= height as i32 {
                        player.y - H as i32 + height as i32 - player.y
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
    pub fn next_pos(&self, gd: Ad) -> Point {
        let mut pos = self.pos;
        match gd {
            Ad::Front => pos.y -= 1,
            Ad::Right => pos.x += 1,
            Ad::Back => pos.y += 1,
            Ad::Left => pos.x -= 1,
            _ => {}
        };
        pos
    }

    /// 移动视野
    pub fn r#move(&mut self, gd: Ad) {
        self.pos = self.next_pos(gd);
    }

    /// 根据视野位置更新视野数据
    pub fn update_data(&mut self, map: &Map<T>) {
        let Point { x, y } = self.pos;
        let data = map.data.iter();
        if map.width < W && map.height < H {
            self.data.clone_from(&map.data);
        } else if map.width < W {
            self.data = data
                .filter(|d| d.0 .0.y >= y && d.0 .0.y < y + H as i32)
                .cloned()
                .collect::<Vec<_>>();
        } else if map.height < H {
            self.data = data
                .filter(|d| d.0 .0.x >= x && d.0 .0.x < x + W as i32)
                .cloned()
                .collect::<Vec<_>>();
        } else {
            self.data = data
                .filter(|d| {
                    d.0 .0.x >= x
                        && d.0 .0.x < x + W as i32
                        && d.0 .0.y >= y
                        && d.0 .0.y < y + H as i32
                })
                .cloned()
                .collect::<Vec<_>>();
        }
    }

    /// 改变视野位置
    pub fn update(&mut self, gd: Ad, map: &Map<T>) {
        let Point { x, y } = self.next_pos(gd);
        let overlapping = {
            if map.width < W && map.height < H {
                true
            } else if map.width < W {
                x < 0 || y < 0 || x > map.width as i32 - 7 || y >= map.height as i32 - 7
            } else if map.height < H {
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
