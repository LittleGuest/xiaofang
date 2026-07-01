use crate::{Ad, Color, Point};
use embedded_graphics::{
    pixelcolor::{Rgb888, WebColors},
    Pixel,
};

/// 玩家
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub pos: Point,
    pub color: Color,
}

impl From<Player> for Pixel<Rgb888> {
    fn from(p: Player) -> Self {
        Pixel(p.pos.into(), p.color)
    }
}

impl Player {
    /// 初始化人物
    pub fn new(pos: Point) -> Self {
        Self {
            pos,
            color: Rgb888::CSS_RED,
        }
    }

    /// 获取玩家的下一个位置
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

    /// 玩家移动
    pub fn r#move(&mut self, gd: Ad) -> bool {
        self.pos = self.next_pos(gd);
        gd != Ad::None
    }
}
