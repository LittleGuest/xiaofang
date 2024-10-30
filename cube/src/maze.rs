#![doc = include_str!("../../rfcs/005_maze.md")]

use crate::{
    map::{Map, Vision},
    player::Player,
    Ad, App, CubeRng, Point, BUZZER, RNG,
};
use alloc::vec::Vec;
use embassy_time::Timer;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::{RgbColor, WebColors},
    Pixel,
};

/// 迷宫
/// 左上角为坐标原点,所有的坐标都为全局坐标
/// 如果地图大小大于8*8,led是显示不完整的,就要添加一个视野的效果,地图的内容根据视野来加载
#[derive(Debug)]
pub struct Maze {
    map: MazeMap,
    player: Player,
    vision: Vision<8, 8, ()>,
    /// ms
    waiting_time: u64,
    game_over: bool,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        let map = MazeMap::new(width, height);
        // FIXME: 随机玩家坐标,这里可能导致起始位置较近，后续使用路径算法生成
        let pp = loop {
            let pp = unsafe {
                Point {
                    x: CubeRng(RNG.assume_init_mut().random() as u64).random_range(1..width) as i32,
                    y: CubeRng(RNG.assume_init_mut().random() as u64).random_range(1..height)
                        as i32,
                }
            };
            let md = map
                .map
                .data
                .iter()
                .any(|c| c.0 .0.x == pp.x && c.0 .0.y == pp.y);
            if !md {
                break pp;
            }
        };
        let player = Player::new(pp);
        let mut vision = Vision::new(width, height, player.pos);
        vision.update_data(&map.map);
        let mut maze = Maze {
            map,
            player,
            vision,
            waiting_time: 300,
            game_over: false,
        };
        maze.map.spos = player.pos;
        maze.map.cal_epos();
        maze
    }

    pub async fn run<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<'_, T>) {
        app.ledc.clear();
        app.ad = Ad::default();

        loop {
            Timer::after_millis(self.waiting_time).await;

            if self.game_over {
                // TODO: 结束动画
                unsafe { BUZZER.assume_init_mut().maze_over().await };
                Timer::after_millis(3000).await;
                break;
            }
            app.acc_direction();

            if !self.hit_wall(app) {
                let moved = self.player.r#move(app.ad);
                if moved {
                    unsafe { BUZZER.assume_init_mut().maze_move().await };
                    // 玩家移动之后视野数据改变
                    self.vision.update(app.ad, &self.map.map);
                    // 游戏结束
                    if self.player.pos.x == self.map.epos.x && self.player.pos.y == self.map.epos.y
                    {
                        self.game_over = true;
                    }
                }
            }
            self.draw(app);
        }
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
        // 终点
        let pp = {
            let pp = self.map.epos;
            let vp = self.vision.pos;
            Pixel(((pp.x - vp.x), (pp.y - vp.y)).into(), self.map.color_epos)
        };
        pixels.push(pp);
        // 玩家
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
        let Point { x, y } = self.player.next_pos(app.ad);
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
            .any(|c| c.0 .0.x == x && c.0 .0.y == y)
    }
}

/// 迷宫地图
#[derive(Debug)]
struct MazeMap {
    map: Map<()>,
    /// 地图颜色
    color: Rgb888,
    /// 起点
    spos: Point,
    /// 终点
    epos: Point,
    /// 终点颜色
    color_epos: Rgb888,
}

impl MazeMap {
    fn new(width: usize, height: usize) -> Self {
        // 使用地图生成算法生成地图 TODO: 迷宫大小,使用的算法都随机
        let maze = maze::Maze::new(width, height)
            .unwrap()
            .generate(&mut unsafe { CubeRng(RNG.assume_init_mut().random() as u64) });
        log::info!("\n{maze}\n");
        let mut map = Map::new(width, height);
        for y in 0..height {
            for x in 0..width {
                if maze[y][x] == 1 {
                    map.data
                        .push((Pixel((x as i32, y as i32).into(), Rgb888::CSS_WHITE), ()));
                }
            }
        }
        Self {
            map,
            color: Rgb888::WHITE,
            spos: Point::default(),
            epos: Point::default(),
            color_epos: Rgb888::CSS_GREEN,
        }
    }

    /// 计算结束位置
    fn cal_epos(&mut self) {
        let pos = loop {
            let x = unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64).random_range(0..self.map.width - 1)
            } as i32;

            let y = unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64).random_range(0..self.map.height - 1)
            } as i32;

            if self.map.data.iter().any(|c| c.0 .0.x == x && c.0 .0.y == y)
                || (self.spos.x == x && self.spos.y == y)
            {
                continue;
            }

            break (x, y);
        };
        self.epos = pos.into();
    }
}
