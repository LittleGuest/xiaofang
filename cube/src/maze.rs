use alloc::vec::Vec;
use embassy_time::Timer;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::{RgbColor, WebColors},
    Pixel,
};

use crate::{App, CubeRng, Gd, Position, RNG};

/// 迷宫
/// 左上角为坐标原点,所有的坐标都为全局坐标
/// 如果地图大小大于8*8,led是显示不完整的,就要添加一个视野的效果,地图的内容根据视野来加载
#[derive(Debug)]
pub struct Maze {
    map: MazeMap,
    player: Player,
    vision: Vision,
    /// ms
    waiting_time: u64,
    game_over: bool,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        let map = MazeMap::new(width, height);
        // 随机玩家坐标
        let pp = loop {
            let pp = Position::random_range_usize(1..width, 1..height);
            let md = map.data[pp.x as usize][pp.y as usize];
            if md.is_none() {
                break pp;
            }
        };
        let player = Player::new(pp);

        // 设置一个初始视野坐标
        let vpx = {
            if player.pos.x - 3 <= 0 {
                0
            } else if player.pos.x + 5 >= width as i8 {
                // W as i8 - 8 + (W as i8 - player.pos.x) - 1
                player.pos.x - 8 + width as i8 - player.pos.x
            } else {
                player.pos.x - 3
            }
        };
        let vpy = {
            if player.pos.y - 3 <= 0 {
                0
            } else if player.pos.y + 5 >= height as i8 {
                // H as i8 - 8 + (H as i8 - player.pos.y) - 1
                player.pos.y - 8 + height as i8 - player.pos.y
            } else {
                player.pos.y - 3
            }
        };

        let mut maze = Maze {
            map,
            player,
            vision: Vision::new(Position::new(vpx, vpy)),
            waiting_time: 300,
            game_over: false,
        };

        maze.copy_map_to_vision();
        maze.map.spos = player.pos;
        maze.map.cal_epos();

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

                // 游戏结束
                if self.player.pos.x == self.map.epos.x && self.player.pos.y == self.map.epos.y {
                    self.game_over = true;
                }
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
                if self.vision.data[y][x].is_some() {
                    pixels.push(Pixel((x as i32, y as i32).into(), self.map.color));
                }
            }
        }

        let pp = {
            let pp = self.map.epos;
            let vp = self.vision.pos;
            Pixel(
                ((pp.x - vp.x) as i32, (pp.y - vp.y) as i32).into(),
                self.map.color_epos,
            )
        };
        pixels.push(pp);

        let pp = {
            let pp = self.player.pos;
            let vp = self.vision.pos;
            Pixel(
                ((pp.x - vp.x) as i32, (pp.y - vp.y) as i32).into(),
                self.player.color,
            )
        };
        pixels.push(pp);
        app.ledc.write_pixels(pixels);
    }

    /// 检测是否撞墙
    fn hit_wall<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) -> bool {
        let Position { x, y } = self.player.next_pos(app);
        let overlapping =
            x <= 0 || y <= 0 || x >= self.map.width as i8 - 1 || y >= self.map.height as i8 - 1;
        if overlapping {
            return true;
        }
        // 检测玩家下一个位置是否有墙
        self.map.data[y as usize][x as usize].is_some()
    }

    /// 将对应的地图数据复制给视野
    fn copy_map_to_vision(&mut self) {
        let Position { x, y } = self.vision.pos;
        for (iy, y) in (y..(y + 8)).enumerate() {
            for (ix, x) in (x..(x + 8)).enumerate() {
                self.vision.data[iy][ix] = self.map.data[y as usize][x as usize];
            }
        }
    }

    /// 改变视野位置
    fn update_vision<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        let Position { x, y } = self.vision.next_pos(app);
        let overlapping =
            x < 0 || y < 0 || x >= self.map.width as i8 - 7 || y >= self.map.height as i8 - 7;
        if overlapping {
            return;
        }
        self.vision.r#move(app);
        // 根据视野位置更新视野数据
        self.copy_map_to_vision();
    }
}

/// 迷宫地图
#[derive(Debug)]
struct MazeMap {
    /// 宽度
    width: usize,
    /// 长度
    height: usize,
    /// 地图数据
    data: Vec<Vec<Option<Position>>>,
    /// 地图颜色
    color: Rgb888,
    /// 起点
    spos: Position,
    /// 终点
    epos: Position,
    /// 终点颜色
    color_epos: Rgb888,
}

impl MazeMap {
    fn new(width: usize, height: usize) -> Self {
        // 使用地图生成算法生成地图 TODO: 迷宫大小,使用的算法都随机
        let mut data = Vec::<Vec<Option<Position>>>::with_capacity(height);
        for _ in 0..height {
            let mut tmp = Vec::<Option<Position>>::with_capacity(width);
            for _ in 0..width {
                tmp.push(None);
            }
            data.push(tmp);
        }

        let maze = maze::Maze::new(width, height)
            .unwrap()
            .generate(&mut unsafe { CubeRng(RNG.assume_init_mut().random() as u64) });
        log::info!("\n{maze}\n");

        for y in 0..height {
            for x in 0..width {
                if maze[y][x] == 1 {
                    data[y][x] = Some(Position::new(x as i8, y as i8));
                }
            }
        }

        Self {
            width,
            height,
            data,
            color: Rgb888::WHITE,
            spos: Position::default(),
            epos: Position::default(),
            color_epos: Rgb888::CSS_LIGHT_GREEN,
        }
    }

    /// 计算结束位置
    fn cal_epos(&mut self) {
        let pos = loop {
            let x = unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64).random_range(0..self.width - 1)
            };

            let y = unsafe {
                CubeRng(RNG.assume_init_mut().random() as u64).random_range(0..self.height - 1)
            };

            if self.data[y][x].is_some() || (self.spos.x as usize == x && self.spos.y as usize == y)
            {
                continue;
            }

            break (x, y);
        };
        self.epos = pos.into();
    }
}

/// 视野
#[derive(Debug)]
struct Vision {
    /// 视野左上角坐标
    pos: Position,
    /// 视野数据
    data: [[Option<Position>; 8]; 8],
}

impl core::fmt::Display for Vision {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:?}", self.pos)?;
        for y in 0..8 {
            writeln!(f, "{:?}", self.data[y])?;
        }
        Ok(())
    }
}

impl Vision {
    fn new(pos: Position) -> Self {
        Self {
            pos,
            data: [[None; 8]; 8],
        }
    }

    fn next_pos<T: esp_hal::i2c::Instance>(&self, app: &mut App<T>) -> Position {
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
    pos: Position,
    old_pos: Position,
    color: Rgb888,
}

impl Player {
    fn new(pos: Position) -> Self {
        Self {
            pos,
            old_pos: pos,
            color: Rgb888::CSS_ORANGE_RED,
        }
    }

    fn next_pos<T: esp_hal::i2c::Instance>(&self, app: &mut App<T>) -> Position {
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
        self.old_pos = self.pos;
        self.pos = self.next_pos(app);
    }
}
