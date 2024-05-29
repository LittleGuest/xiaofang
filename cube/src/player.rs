use crate::Gd;
use embedded_graphics::{
    geometry::Point,
    pixelcolor::{Rgb888, WebColors},
    Pixel,
};

/// 玩家
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub pos: Point,
    pub color: Rgb888,
}

impl From<Player> for Pixel<Rgb888> {
    fn from(p: Player) -> Self {
        Pixel(p.pos, p.color)
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

    /// 玩家移动
    pub fn r#move(&mut self, gd: Gd) -> bool {
        self.pos = self.next_pos(gd);
        gd != Gd::None
    }
}
