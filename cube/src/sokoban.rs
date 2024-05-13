#![doc = include_str!("../../rfcs/007_sokoban.md")]

use alloc::vec::Vec;
use embassy_time::Timer;
use embedded_graphics::{geometry::Point, pixelcolor::PixelColor};
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::{RgbColor, WebColors},
    Pixel,
};
use log::{debug, info};

use crate::{App, CubeRng, Gd, RNG};

/// 推箱子
/// 左上角为坐标原点,所有的坐标都为全局坐标
/// 如果地图大小大于8*8,led是显示不完整的,就要添加一个视野的效果
/// 地图的内容根据视野来加载
#[derive(Debug)]
pub struct Sokoban {
    map: Map,
    player: Player,
    vision: Vision,
    /// ms
    waiting_time: u64,
    game_over: bool,
}

impl Default for Sokoban {
    fn default() -> Self {
        Self::new()
    }
}

impl Sokoban {
    pub fn new() -> Self {
        let xsb = "
----#####----------
----#---#----------
----#$--#----------
--###--$##---------
--#--$-$-#---------
###-#-##-#---######
#---#-##-#####--..#
#-$--$----------..#
#####-###-#@##--..#
----#-----#########
----#######--------
";
        let map = Map::from_xsb(xsb);
        debug!("sokoban: {map}");
        let pp = map
            .data
            .iter()
            .flatten()
            .filter(|m| m.is_some())
            .cloned()
            .find(|m| m.unwrap().0 == TargetType::Man);
        let player = Player::new(pp.unwrap().unwrap().1 .0);

        // 设置一个初始视野坐标
        let vpx = {
            if player.pos.x - 3 <= 0 {
                0
            } else if player.pos.x + 5 >= map.width as i32 {
                player.pos.x - 8 + map.width as i32 - player.pos.x
            } else {
                player.pos.x - 3
            }
        };
        let vpy = {
            if player.pos.y - 3 <= 0 {
                0
            } else if player.pos.y + 5 >= map.height as i32 {
                player.pos.y - 8 + map.height as i32 - player.pos.y
            } else {
                player.pos.y - 3
            }
        };

        let mut maze = Sokoban {
            map,
            player,
            vision: Vision::new(Point::new(vpx, vpy)),
            waiting_time: 300,
            game_over: false,
        };

        maze.copy_map_to_vision();

        maze
    }

    pub async fn run<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<'_, T>) {
        app.ledc.clear();
        app.gd = Gd::default();

        loop {
            if self.game_over {
                // TODO: 结束动画和音乐
                Timer::after_millis(3000).await;
                break;
            }
            app.gravity_direction();

            if !self.hit_wall(app) {
                self.player.r#move(app);
                // 玩家移动之后视野数据改变
                self.update_vision(app);

                // TODO: 游戏结束条件是所有箱子都在目标点上
                // if self.player.pos.x == self.map.epos.x && self.player.pos.y == self.map.epos.y {
                self.game_over = true;
                // }
            }
            self.draw(app);

            Timer::after_millis(self.waiting_time).await;
        }
    }

    fn draw<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear_with_color(BinaryColor::Off.into());
        let mut pixels = Vec::<Pixel<Rgb888>>::new();
        // 将地图坐标转换为led坐标
        for y in 0..8 {
            for x in 0..8 {
                if let Some(d) = self.vision.data[y][x] {
                    pixels.push(Pixel((x as i32, y as i32).into(), d.1 .1));
                }
            }
        }
        // 人物
        let pp = {
            let pp = self.player.pos;
            let vp = self.vision.pos;
            Pixel(((pp.x - vp.x), (pp.y - vp.y)).into(), self.player.color)
        };
        pixels.push(pp);
        app.ledc.write_pixels(pixels);
    }

    /// 检测是否撞墙
    fn hit_wall<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) -> bool {
        let Point { x, y } = self.player.next_pos(app);
        let overlapping =
            x <= 0 || y <= 0 || x >= self.map.width as i32 - 1 || y >= self.map.height as i32 - 1;
        if overlapping {
            return true;
        }
        // 检测玩家下一个位置是否有墙
        self.map.data[y as usize][x as usize].is_some()
    }

    /// 将对应的地图数据复制给视野
    fn copy_map_to_vision(&mut self) {
        let Point { x, y } = self.vision.pos;
        for (iy, y) in (y..(y + 8)).enumerate() {
            for (ix, x) in (x..(x + 8)).enumerate() {
                self.vision.data[iy][ix] = self.map.data[y as usize][x as usize];
            }
        }
    }

    /// 改变视野位置
    fn update_vision<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        let Point { x, y } = self.vision.next_pos(app);
        let overlapping =
            x < 0 || y < 0 || x >= self.map.width as i32 - 7 || y >= self.map.height as i32 - 7;
        if overlapping {
            return;
        }
        self.vision.r#move(app);
        // 根据视野位置更新视野数据
        self.copy_map_to_vision();
    }
}

/// 标记地图中的类型,表示墙,人还是目标点
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TargetType {
    /// 人
    Man,
    /// 人在目标点上
    ManOnGoal,
    /// 箱子
    Box,
    /// 箱子在目标点上
    BoxOnGoal,
    /// 墙
    Wall,
    /// 目标点
    Goal,
    /// 地板
    Floor,
}

type MapCell = (TargetType, Pixel<Rgb888>);

/// 迷宫地图
#[derive(Debug)]
struct Map {
    /// 宽度
    width: usize,
    /// 长度
    height: usize,
    /// 地图数据
    data: Vec<Vec<Option<MapCell>>>,
}

impl core::fmt::Display for Map {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "whidth: {}, height: {}", self.width, self.height);
        for y in self.data.iter() {
            for x in y.iter() {
                if let Some((tt, x)) = x {
                    match tt {
                        TargetType::Man => write!(f, "@"),
                        TargetType::ManOnGoal => write!(f, "+"),
                        TargetType::Box => write!(f, "$"),
                        TargetType::BoxOnGoal => write!(f, "*"),
                        TargetType::Wall => write!(f, "#"),
                        TargetType::Goal => write!(f, "."),
                        TargetType::Floor => write!(f, "-"),
                    };
                } else {
                    write!(f, "-");
                }
            }
            writeln!(f);
        }
        Ok(())
    }
}

impl Map {
    /// TODO: 使用算法生成
    fn new() -> Self {
        unimplemented!()
    }

    /// 根据XSB生成地图
    fn from_xsb(xsb: &str) -> Self {
        let mut width = 1;
        let mut height = 1;
        let mut data = Vec::<Vec<Option<(TargetType, Pixel<Rgb888>)>>>::new();
        for (y, line) in xsb.lines().enumerate() {
            height = y;
            let y = y as i32;
            let mut tmp = Vec::<Option<(TargetType, Pixel<Rgb888>)>>::new();
            for (x, char) in line.chars().enumerate() {
                width = x;
                let x = x as i32;
                let pos = match char {
                    '@' => Some((TargetType::Man, Pixel((x, y).into(), Rgb888::CSS_RED))),
                    '+' => Some((
                        TargetType::ManOnGoal,
                        Pixel((x, y).into(), Rgb888::CSS_YELLOW),
                    )),
                    '$' => Some((TargetType::Box, Pixel((x, y).into(), Rgb888::CSS_BROWN))),
                    '*' => Some((
                        TargetType::BoxOnGoal,
                        Pixel((x, y).into(), Rgb888::CSS_DARK_SLATE_GRAY),
                    )),
                    '#' => Some((TargetType::Wall, Pixel((x, y).into(), Rgb888::CSS_WHITE))),
                    '.' => Some((TargetType::Goal, Pixel((x, y).into(), Rgb888::CSS_GREEN))),
                    _ => None,
                };
                tmp.push(pos);
            }
            data.push(tmp);
        }

        Self {
            width,
            height,
            data,
        }
    }

    /// TODO: 根据LURD生成地图
    fn from_lurd(lurd: &str) -> Self {
        unimplemented!()
    }
}

/// 视野
#[derive(Debug)]
struct Vision {
    /// 视野左上角坐标
    pos: Point,
    /// 视野数据
    data: [[Option<MapCell>; 8]; 8],
}

impl Vision {
    fn new(pos: Point) -> Self {
        Self {
            pos,
            data: [[None; 8]; 8],
        }
    }

    fn next_pos<T: esp_hal::i2c::Instance>(&self, app: &mut App<T>) -> Point {
        let mut pos = self.pos;
        match app.gd {
            Gd::None => {}
            Gd::Up => pos.y -= 1,
            Gd::Right => pos.x += 1,
            Gd::Down => pos.y += 1,
            Gd::Left => pos.x -= 1,
        };
        pos
    }

    fn r#move<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.pos = self.next_pos(app);
    }
}

/// 玩家
#[derive(Debug, Clone, Copy)]
struct Player {
    pos: Point,
    color: Rgb888,
}

impl Player {
    fn new(pos: Point) -> Self {
        Self {
            pos,
            color: Rgb888::CSS_RED,
        }
    }

    fn next_pos<T: esp_hal::i2c::Instance>(&self, app: &mut App<T>) -> Point {
        let mut pos = self.pos;
        match app.gd {
            Gd::None => {}
            Gd::Up => pos.y -= 1,
            Gd::Right => pos.x += 1,
            Gd::Down => pos.y += 1,
            Gd::Left => pos.x -= 1,
        };
        pos
    }

    fn r#move<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.pos = self.next_pos(app);
    }
}
