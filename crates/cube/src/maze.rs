#![doc = include_str!("../../../rfcs/005_maze.md")]

use alloc::vec::Vec;

use embassy_time::Timer;
use embedded_graphics_core::{
    Pixel,
    pixelcolor::{BinaryColor, Rgb888},
    prelude::WebColors,
};
use esp_hal::rng::Rng;

use crate::{
    Ad, App, CubeRng, Point, buzzer,
    map::{Map, Vision},
    player::Player,
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
    pub fn new(width: usize, height: usize, rng: &mut Rng) -> Self {
        let map = MazeMap::new(width, height, rng);
        let pp = loop {
            let pp = Point {
                x: CubeRng(rng.random() as u64).random_range(1..width) as i32,
                y: CubeRng(rng.random() as u64).random_range(1..height) as i32,
            };
            let md = map.map.data.iter().any(|c| c.0.0.x == pp.x && c.0.0.y == pp.y);
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

    pub async fn run(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.ad = Ad::default();

        loop {
            Timer::after_millis(self.waiting_time).await;

            if self.game_over {
                // 庆祝动画：绿色闪烁
                for _ in 0..3 {
                    app.ledc.clear_with_color(Rgb888::CSS_GREEN);
                    Timer::after_millis(200).await;
                    app.ledc.clear();
                    Timer::after_millis(200).await;
                }
                buzzer::maze_over().await;
                Timer::after_millis(1500).await;
                break;
            }
            app.acc_direction();
            app.check_pause().await;

            if !self.hit_wall(app) {
                let moved = self.player.r#move(app.ad);
                if moved {
                    buzzer::maze_move().await;
                    // 玩家移动之后视野数据改变
                    self.vision.update(app.ad, &self.map.map);
                    // 游戏结束
                    if self.player.pos.x == self.map.epos.x && self.player.pos.y == self.map.epos.y {
                        self.game_over = true;
                    }
                }
            }
            self.draw(app);
        }
    }

    fn draw(&mut self, app: &mut App<'_>) {
        app.ledc.clear_with_color(BinaryColor::Off.into());
        let vp = self.vision.pos;
        let mut pixels = self.map.map.data.iter().map(|m| m.0).clone().collect::<Vec<_>>();
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
    fn hit_wall(&mut self, app: &mut App<'_>) -> bool {
        let Point { x, y } = self.player.next_pos(app.ad);
        let overlapping = x <= 0 || y <= 0 || x >= self.map.map.width as i32 - 1 || y >= self.map.map.height as i32 - 1;
        if overlapping {
            return true;
        }
        // 检测玩家下一个位置是否有墙
        self.map.map.data.iter().any(|c| c.0.0.x == x && c.0.0.y == y)
    }
}

/// 迷宫地图
#[derive(Debug)]
struct MazeMap {
    map: Map<()>,
    /// 起点
    spos: Point,
    /// 终点
    epos: Point,
    /// 终点颜色
    color_epos: Rgb888,
}

impl MazeMap {
    fn new(width: usize, height: usize, rng: &mut Rng) -> Self {
        // 使用地图生成算法生成地图 TODO: 迷宫大小,使用的算法都随机
        // 迷宫需为奇数且>=5 的尺寸; 调用方 Maze::new 已保证(cr 为 19..=33 的奇数), 不可达时为程序错误
        let maze = maze::Maze::new(width, height)
            .expect("迷宫尺寸非法: 需为奇数且>=5")
            .generate(&mut CubeRng(rng.random() as u64));
        let mut map = Map::new(width, height);
        for y in 0..height {
            for (x, item) in maze[y].iter().enumerate().take(width) {
                if *item == 1 {
                    map.data
                        .push((Pixel((x as i32, y as i32).into(), Rgb888::CSS_WHITE), ()));
                }
            }
        }
        Self {
            map,
            spos: Point::default(),
            epos: Point::default(),
            color_epos: Rgb888::CSS_GREEN,
        }
    }

    /// 使用BFS计算结束位置，选择距离起点最远的可达点
    fn cal_epos(&mut self) {
        use alloc::{collections::VecDeque, vec::Vec};

        let w = self.map.width;
        let h = self.map.height;
        let walls: Vec<(i32, i32)> = self.map.data.iter().map(|c| (c.0.0.x, c.0.0.y)).collect();

        // distance[y][x] = 距离，usize::MAX 表示未访问
        let mut distance = alloc::vec![alloc::vec![usize::MAX; w]; h];
        let mut queue = VecDeque::new();

        let sx = self.spos.x as usize;
        let sy = self.spos.y as usize;
        distance[sy][sx] = 0;
        queue.push_back((sx, sy));

        let dirs: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        let mut max_dist = 0;
        let mut best = (sx, sy);

        while let Some((cx, cy)) = queue.pop_front() {
            let cur_d = distance[cy][cx];
            for (dx, dy) in dirs {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if walls.contains(&(nx as i32, ny as i32)) || distance[ny][nx] != usize::MAX {
                    continue;
                }
                distance[ny][nx] = cur_d + 1;
                if cur_d + 1 > max_dist {
                    max_dist = cur_d + 1;
                    best = (nx, ny);
                }
                queue.push_back((nx, ny));
            }
        }

        self.epos = Point::new(best.0 as i32, best.1 as i32);
    }
}
