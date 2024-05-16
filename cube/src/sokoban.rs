#![doc = include_str!("../../rfcs/007_sokoban.md")]

use alloc::vec::Vec;
use embassy_time::Timer;
use embedded_graphics::geometry::Point;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::WebColors,
    Pixel,
};

use crate::{
    map::{Map, MapCell, Vision},
    player::Player,
    App, Gd,
};

/// 推箱子
/// 左上角为坐标原点,所有的坐标都为全局坐标
/// 如果地图大小大于8*8,led是显示不完整的,就要添加一个视野的效果
/// 地图的内容根据视野来加载
#[derive(Debug)]
pub struct Sokoban {
    map: SokobanMap,
    player: Player,
    vision: Vision<8, 8, TargetType>,
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
########
#--#---#
#-$----#
#--*.*-#
#-$@*--#
###$*--#
-#--*--#
-#-#.--#
-#--.--#
-#######
";
        let map = SokobanMap::from_xsb(xsb);
        let width = map.map.width;
        let height = map.map.height;
        let player = Player::new(map.player.0 .0);
        let mut vision = Vision::new(width, height, player.pos);
        vision.update_data(&map.map);
        Sokoban {
            map,
            player,
            vision,
            waiting_time: 300,
            game_over: false,
        }
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
                    self.player.r#move(app.gd);
                }
                // 玩家移动之后视野数据改变
                self.vision.update(app.gd, &self.map.map);
                self.game_over();
            }
            self.draw(app);
        }
    }

    /// 推动箱子
    fn push_box<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) -> bool {
        let Point { x, y } = self.player.next_pos(app.gd);
        let boxs = self.map.boxs.clone();
        for (cp, ct) in self.map.boxs.iter_mut() {
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
                    matches!(m.1, TargetType::Box) && m.0 .0.x == boxp.x && m.0 .0.y == boxp.y
                });
                let is_wall = self.map.map.data.iter().any(|m| {
                    matches!(m.1, TargetType::Wall) && m.0 .0.x == boxp.x && m.0 .0.y == boxp.y
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
        let goals = self.map.goals.iter().map(|b| b.0 .0).collect::<Vec<_>>();
        let all = self.map.boxs.iter().all(|b| goals.contains(&b.0 .0));
        self.game_over = all;
    }

    fn draw<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear_with_color(BinaryColor::Off.into());
        let vp = self.vision.pos;
        let mut pixels = self
            .map
            .map
            .data
            .iter()
            .map(|m| m.0)
            .clone()
            .collect::<Vec<_>>();
        // 将全局坐标转换为led坐标
        for d in pixels.iter_mut() {
            d.0.x -= vp.x;
            d.0.y -= vp.y;
        }
        // 箱子
        let goals = self.map.goals.iter().map(|m| m.0 .0).collect::<Vec<_>>();
        for b in self.map.boxs.iter().map(|b| b.0) {
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
        let Point { x, y } = self.player.next_pos(app.gd);
        let overlapping = x <= 0
            || y <= 0
            || x >= self.map.map.width as i32 - 1
            || y >= self.map.map.height as i32 - 1;
        if overlapping {
            return true;
        }
        // 检测玩家下一个位置是否有墙
        self.map
            .map
            .data
            .iter()
            .any(|c| c.1 == TargetType::Wall && c.0 .0.x == x && c.0 .0.y == y)
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

/// 迷宫地图
#[derive(Debug, Default)]
struct SokobanMap<T = TargetType> {
    map: Map<T>,
    /// 初始玩家位置
    player: MapCell<T>,
    /// 箱子的位置，动态变化
    boxs: Vec<MapCell<T>>,
    /// 目标点
    goals: Vec<MapCell<T>>,
}

impl SokobanMap {
    /// TODO: 使用算法生成
    fn new() -> Self {
        unimplemented!()
    }

    /// 根据XSB生成地图
    fn from_xsb(xsb: &str) -> Self {
        let mut map = Self::default();
        for (y, line) in xsb.trim().lines().enumerate() {
            let y = y as i32;
            for (x, char) in line.chars().enumerate() {
                let x = x as i32;
                match char {
                    '@' => {
                        map.player = (Pixel((x, y).into(), Rgb888::CSS_RED), TargetType::Man);
                    }
                    '+' => {
                        map.player = (Pixel((x, y).into(), Rgb888::CSS_RED), TargetType::Man);
                        let goal = (Pixel((x, y).into(), Rgb888::CSS_GREEN), TargetType::Goal);
                        map.goals.push(goal);
                    }
                    '$' => {
                        map.boxs
                            .push((Pixel((x, y).into(), Rgb888::CSS_BLUE), TargetType::Box));
                    }
                    '*' => {
                        map.boxs
                            .push((Pixel((x, y).into(), Rgb888::CSS_BLUE), TargetType::Box));
                        let goal = (Pixel((x, y).into(), Rgb888::CSS_GREEN), TargetType::Goal);
                        map.goals.push(goal);
                    }
                    '#' => {
                        let wall = (Pixel((x, y).into(), Rgb888::CSS_WHITE), TargetType::Wall);
                        map.map.data.push(wall);
                    }
                    '.' => {
                        let goal = (Pixel((x, y).into(), Rgb888::CSS_GREEN), TargetType::Goal);
                        map.goals.push(goal);
                        map.map.data.push(goal);
                    }
                    _ => {}
                }
            }
        }
        map.map.height = map
            .map
            .data
            .iter()
            .max_by(|c1, c2| c1.0 .0.y.cmp(&c2.0 .0.y))
            .map_or(0, |c| (c.0 .0.y + 1) as usize);
        map.map.width = map
            .map
            .data
            .iter()
            .max_by(|c1, c2| c1.0 .0.x.cmp(&c2.0 .0.x))
            .map_or(0, |c| (c.0 .0.x + 1) as usize);
        map
    }

    /// TODO: 根据LURD生成地图
    fn from_lurd(_lurd: &str) -> Self {
        unimplemented!()
    }
}
