use alloc::vec::Vec;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::{RgbColor, WebColors},
    Pixel,
};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use crate::{App, Gd, Position};

/// 迷宫
/// 左上角为坐标原点
/// 如果地图大小大于8*8,led是显示不完整的,就要添加一个视野的效果,地图的内容根据视野来加载
///
/// 那么视野怎么移动呢,是根据玩家的位置来移动视野的,但是玩家的位置是不能超过视野范围的
/// 因为当玩家向前移动时,需要把地图的内容加载到视野中,就是说当玩家移动到视野范围-1或-2时,
/// 视野范围就需要跟着玩家一起向前运动(也就是视野向后移动)
/// 当视野范围的边界的地图的边界重叠时,再向前移动,玩家可以靠近边界移动
///
#[derive(Debug)]
pub struct Maze<const W: usize, const H: usize> {
    map: MazeMap<W, H>,
    player: Player,
    vision: Vision,
    /// ms
    waiting_time: u32,
    game_over: bool,
}

impl<const W: usize, const H: usize> Maze<W, H> {
    pub fn new() -> Self {
        let map = MazeMap::<W, H>::new();
        // 随机玩家坐标
        let pp = loop {
            let pp = Position::random_range(1..W as i8, 1..H as i8);
            let md = map.data[pp.x as usize][pp.y as usize];
            if md.is_none() {
                break pp;
            }
        };
        let player = Player::new(pp);

        // 视野数据根据玩家坐标决定
        let mut vp = Position::default();
        vp.x = player.pos.x - 3;
        vp.y = player.pos.y - 3;

        if player.pos.x - 3 <= 0 {
            vp.x = 0;
        }
        if player.pos.y - 3 <= 0 {
            vp.y = 0;
        }
        if player.pos.x + 5 >= W as i8 {
            vp.x = W as i8 - 7;
        }
        if player.pos.y + 5 >= H as i8 {
            vp.y = H as i8 - 7;
        }

        let mut maze = Maze {
            map,
            player,
            vision: Vision::new(vp),
            waiting_time: 300,
            game_over: false,
        };
        maze.copy_map_to_vision();

        maze
    }

    pub fn run<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        app.gd = Gd::default();

        loop {
            // if self.game_over {
            //     // TODO 结束动画和音乐
            //     app.delay.delay_ms(3000_u32);
            //     log::info!("maze game over");
            //     break;
            // }
            app.gravity_direction();

            log::info!("player pos {:?}", self.player.pos);
            log::info!("vision pos {:?}", self.vision.pos);
            if !self.hit_wall(app) {
                self.player.r#move(app);
                // 玩家移动之后视野数据改变
                self.update_vision(app);
            } else {
                log::info!("hit wall");
            }
            self.draw(app);

            app.delay.delay_ms(self.waiting_time);
        }
    }

    pub fn draw<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        // 将视野数据和玩家数据合并后写入
        // 视野数据来源于玩家移动后的地图数据
        // self.map.data.push(self.player.pos);
        app.ledc.clear_with_color(BinaryColor::Off.into());
        // app.ledc.write_pixels(
        //     self.vision
        //         .data
        //         .iter()
        //         .flatten()
        //         .filter(|d| d.is_some())
        //         .map(|&d| d.unwrap().into())
        //         .into_iter(),
        // );
        // self.player.draw(app);

        // 需要进行坐标转换
        let mut pixels = Vec::<Pixel<Rgb888>>::new();
        for y in 0..8 {
            for x in 0..8 {
                if self.vision.data[y][x].is_some() {
                    pixels.push(Pixel((x as i32, y as i32).into(), self.map.color));
                }
            }
        }
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

    pub fn hit_wall<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) -> bool {
        let Position { x, y } = self.player.next_pos(app);
        let overlapping =
            x <= 0 || y <= 0 || x >= self.map.width as i8 - 1 || y >= self.map.height as i8 - 1;
        if overlapping {
            return true;
        }
        // 检测玩家下一个位置是否有墙
        let md = self.map.data[y as usize][x as usize];
        md.is_some()
    }

    /// 将对应的地图数据复制给视野
    fn copy_map_to_vision(&mut self) {
        let Position { x, y } = self.vision.pos;
        for (iy, y) in (y..(y + 8)).enumerate() {
            for (ix, x) in (x..(x + 8)).enumerate() {
                self.vision.data[iy as usize][ix as usize] = self.map.data[y as usize][x as usize];
            }
        }
    }

    /// 改变视野位置
    fn update_vision<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        let Position { x, y } = self.vision.next_pos(app);
        let overlapping =
            x < 0 || y < 0 || x >= self.map.width as i8 - 7 || y >= self.map.height as i8 - 7;
        log::info!("vision overlapping : {overlapping}");
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
struct MazeMap<const W: usize, const H: usize> {
    /// 宽度
    width: usize,
    /// 长度
    height: usize,
    /// 地图数据
    data: [[Option<Position>; H]; W],
    /// 地图颜色
    color: Rgb888,
    /// 起点
    spos: Position,
    /// 终点
    epos: Position,
}

impl<const W: usize, const H: usize> MazeMap<W, H> {
    fn new() -> Self {
        // 使用地图生成算法生成地图
        // TODO 迷宫大小,使用的算法都随机
        let mut data = [[None; H]; W];
        let maze = irrgarten::Maze::new(W, H).unwrap().generate();
        for y in 0..H {
            log::info!("{:?}", maze[y]);
            for x in 0..W {
                if maze[y][x] == 1 {
                    data[y][x] = Some(Position::new(x as i8, y as i8));
                }
            }
        }

        Self {
            width: W,
            height: H,
            data,
            color: Rgb888::WHITE,
            spos: Position::default(),
            epos: Position::default(),
        }
    }
}

/// 视野
#[derive(Debug)]
struct Vision {
    /// 视野左上角坐标
    pos: Position,
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

    fn next_pos<T: hal::i2c::Instance>(&self, app: &mut App<T>) -> Position {
        let mut pos = (*self).pos;
        match app.gd {
            Gd::None => {}
            Gd::Up => pos.y -= 1,
            Gd::Right => pos.x += 1,
            Gd::Down => pos.y += 1,
            Gd::Left => pos.x -= 1,
        };
        pos
    }

    fn r#move<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.pos = self.next_pos(app);
    }
}

/// 玩家
#[derive(Debug)]
struct Player {
    pos: Position,
    old_pos: Position,
    color: Rgb888,
    // poss: Vec<Position>,
}

impl Player {
    fn new(pos: Position) -> Self {
        // let width = 2;
        // let height = 2;
        // let mut poss = Vec::<Position>::with_capacity(4);
        // for y in 0..height {
        //     for x in 0..width {
        //         poss.push(Position::new(pos.x + x, pos.y + y));
        //     }
        // }
        Self {
            pos,
            old_pos: pos,
            color: Rgb888::CSS_ORANGE_RED,
        }
    }

    fn next_pos<T: hal::i2c::Instance>(&self, app: &mut App<T>) -> Position {
        let mut pos = (*self).pos;
        match app.gd {
            Gd::None => {}
            Gd::Up => pos.y -= 1,
            Gd::Right => pos.x += 1,
            Gd::Down => pos.y += 1,
            Gd::Left => pos.x -= 1,
        };
        pos
    }

    fn r#move<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.old_pos = self.pos;
        self.pos = self.next_pos(app);
    }

    fn draw<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.draw_off(app);
        self.draw_on(app);
    }

    fn draw_off<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.write_pixel(Pixel(
            (self.old_pos.x as i32, self.old_pos.y as i32).into(),
            BinaryColor::Off.into(),
        ));
    }

    fn draw_on<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.write_pixel(Pixel(
            (self.pos.x as i32, self.pos.y as i32).into(),
            self.color,
        ));
    }

    // fn as_pixels(&mut self) -> Vec<Pixel<Rgb888>> {
    //     self.poss
    //         .iter()
    //         .map(|p| Pixel((p.x as i32, p.y as i32).into(), BinaryColor::On.into()))
    //         .collect::<Vec<_>>()
    // }
}
