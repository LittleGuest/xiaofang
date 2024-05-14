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
#######
#--#--#
#-$---#
#--*.*#
#-$@*-#
###$*-#
-#--*-#
-#-#.-#
-#--.-#
-######
";
        let map = Map::from_xsb(xsb);
        let player = Player::new(map.player.1 .0);

        // 设置一个初始视野坐标
        let vpx = {
            if player.pos.x - 3 <= 0 || map.width < 8 {
                0
            } else if player.pos.x + 5 >= map.width as i32 {
                player.pos.x - 8 + map.width as i32 - player.pos.x
            } else {
                player.pos.x - 3
            }
        };
        let vpy = {
            if player.pos.y - 3 <= 0 || map.height < 8 {
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
            Timer::after_millis(self.waiting_time).await;

            if self.game_over {
                // TODO: 结束进入下一关
                Timer::after_millis(1500).await;
                app.face
                    .break_record_animate(&mut app.ledc, &mut app.buzzer)
                    .await;
                Timer::after_millis(500).await;
                break;
            }
            app.gravity_direction();

            if !self.hit_wall(app) {
                // 不撞墙，是否在推动箱子，能否推动箱子，能一起移动
                let can_push = self.push_box(app);
                if can_push {
                    self.player.r#move(app);
                }
                // 玩家移动之后视野数据改变
                self.update_vision(app);
                self.game_over();
            }
            self.draw(app);
        }
    }

    /// 推动箱子
    fn push_box<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) -> bool {
        let Point { x, y } = self.player.next_pos(app);
        let boxs = self.map.boxs.clone();
        for (ct, cp) in self.map.boxs.iter_mut() {
            // 下一个位置是箱子且能推动则推箱子
            if TargetType::Box.eq(ct) && cp.0.x == x && cp.0.y == y {
                // 再下一个位置
                let mut boxp = cp.0;
                match app.gd {
                    Gd::None => {}
                    Gd::Up => boxp.y -= 1,
                    Gd::Right => boxp.x += 1,
                    Gd::Down => boxp.y += 1,
                    Gd::Left => boxp.x -= 1,
                };
                let is_box = boxs.iter().any(|m| {
                    matches!(m.0, TargetType::Box) && m.1 .0.x == boxp.x && m.1 .0.y == boxp.y
                });
                let is_wall = self.map.data.iter().flatten().flatten().any(|m| {
                    matches!(m.0, TargetType::Wall) && m.1 .0.x == boxp.x && m.1 .0.y == boxp.y
                });
                if is_box || is_wall {
                    return false;
                }

                // 推动箱子
                match app.gd {
                    Gd::None => {}
                    Gd::Up => cp.0.y -= 1,
                    Gd::Right => cp.0.x += 1,
                    Gd::Down => cp.0.y += 1,
                    Gd::Left => cp.0.x -= 1,
                };
            }
        }
        true
    }

    /// 游戏结束，条件是所有箱子都在目标点上
    fn game_over(&mut self) {
        let goals = self.map.goals.iter().map(|b| b.1 .0).collect::<Vec<_>>();
        let all = self.map.boxs.iter().all(|b| goals.contains(&b.1 .0));
        self.game_over = all;
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

        let vp = self.vision.pos;
        let goals = self.map.goals.iter().map(|m| m.1 .0).collect::<Vec<_>>();
        // 箱子
        for b in self.map.boxs.iter().map(|b| b.1) {
            // 青色表示箱子在目标点上
            let color = if goals.contains(&b.0) {
                Rgb888::CSS_CYAN
            } else {
                b.1
            };
            let pp = Pixel(((b.0.x - vp.x), (b.0.y - vp.y)).into(), color);
            pixels.push(pp);
        }
        // 人物
        let pp = {
            let pp = self.player.pos;
            // 黄色表示玩家在目标点上
            let color = if goals.contains(&pp) {
                Rgb888::CSS_YELLOW
            } else {
                self.player.color
            };
            Pixel(((pp.x - vp.x), (pp.y - vp.y)).into(), color)
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
        if let Some(cell) = self.map.data[y as usize][x as usize] {
            matches!(cell.0, TargetType::Wall)
        } else {
            false
        }
    }

    /// 将对应的地图数据复制给视野
    fn copy_map_to_vision(&mut self) {
        let Point { x, y } = self.vision.pos;
        if self.map.width < 8 && self.map.height < 8 {
            for (iy, y) in (y..self.map.height as i32).enumerate() {
                for (ix, x) in (x..self.map.width as i32).enumerate() {
                    self.vision.data[iy][ix] = self.map.data[y as usize][x as usize];
                }
            }
        } else if self.map.width < 8 {
            for (iy, y) in (y..(y + 8)).enumerate() {
                for (ix, x) in (x..self.map.width as i32).enumerate() {
                    self.vision.data[iy][ix] = self.map.data[y as usize][x as usize];
                }
            }
        } else if self.map.height < 8 {
            for (iy, y) in (y..self.map.height as i32).enumerate() {
                for (ix, x) in (x..(x + 8)).enumerate() {
                    self.vision.data[iy][ix] = self.map.data[y as usize][x as usize];
                }
            }
        } else {
            for (iy, y) in (y..(y + 8)).enumerate() {
                for (ix, x) in (x..(x + 8)).enumerate() {
                    self.vision.data[iy][ix] = self.map.data[y as usize][x as usize];
                }
            }
        }
    }

    /// 改变视野位置
    fn update_vision<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        let Point { x, y } = self.vision.next_pos(app);
        let overlapping = {
            if self.map.width < 8 && self.map.height < 8 {
                true
            } else if self.map.width < 8 {
                x < 0 || y < 0 || x > self.map.width as i32 - 7 || y >= self.map.height as i32 - 7
            } else if self.map.height < 8 {
                x < 0 || y < 0 || x >= self.map.width as i32 - 7 || y > self.map.height as i32 - 7
            } else {
                x < 0 || y < 0 || x >= self.map.width as i32 - 7 || y >= self.map.height as i32 - 7
            }
        };
        if overlapping {
            return;
        }
        self.vision.r#move(app);
        // 根据视野位置更新视野数据
        self.copy_map_to_vision();
    }
}

/// 标记地图中的类型,表示墙,人还是目标点
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
enum TargetType {
    /// 人
    Man,
    /// 箱子
    Box,
    /// 墙
    Wall,
    /// 目标点
    Goal,
    /// 地板
    #[default]
    Floor,
}

type MapCell = (TargetType, Pixel<Rgb888>);

/// 迷宫地图
#[derive(Debug, Default)]
struct Map {
    /// 宽度
    width: usize,
    /// 长度
    height: usize,
    /// 原始地图数据
    data: Vec<Vec<Option<MapCell>>>,
    /// 初始玩家位置
    player: MapCell,
    /// 箱子的位置，动态变化
    boxs: Vec<MapCell>,
    /// 目标点
    goals: Vec<MapCell>,
}

impl Map {
    /// TODO: 使用算法生成
    fn new() -> Self {
        unimplemented!()
    }

    /// 根据XSB生成地图
    fn from_xsb(xsb: &str) -> Self {
        let mut map = Self::default();
        for (y, line) in xsb.trim().lines().enumerate() {
            let y = y as i32;
            let mut tmp = Vec::<Option<(TargetType, Pixel<Rgb888>)>>::new();
            for (x, char) in line.chars().enumerate() {
                let x = x as i32;
                let pos = match char {
                    '@' => {
                        map.player = (TargetType::Man, Pixel((x, y).into(), Rgb888::CSS_RED));
                        None
                    }
                    '+' => {
                        map.player = (TargetType::Man, Pixel((x, y).into(), Rgb888::CSS_RED));
                        let goal = (TargetType::Goal, Pixel((x, y).into(), Rgb888::CSS_GREEN));
                        map.goals.push(goal);
                        None
                    }
                    '$' => {
                        map.boxs
                            .push((TargetType::Box, Pixel((x, y).into(), Rgb888::CSS_BLUE)));
                        None
                    }
                    '*' => {
                        map.boxs
                            .push((TargetType::Box, Pixel((x, y).into(), Rgb888::CSS_BLUE)));
                        let goal = (TargetType::Goal, Pixel((x, y).into(), Rgb888::CSS_GREEN));
                        map.goals.push(goal);
                        None
                    }
                    '#' => Some((TargetType::Wall, Pixel((x, y).into(), Rgb888::CSS_WHITE))),
                    '.' => {
                        let goal = (TargetType::Goal, Pixel((x, y).into(), Rgb888::CSS_GREEN));
                        map.goals.push(goal);
                        Some(goal)
                    }
                    _ => None,
                };
                tmp.push(pos);
            }
            map.data.push(tmp);
        }

        map.data.extract_if(|d| d.iter().all(|c| c.is_none()));

        map.height = map.data.len();
        map.width = if map.height == 0 {
            0
        } else {
            map.data[0].len()
        };
        map
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
