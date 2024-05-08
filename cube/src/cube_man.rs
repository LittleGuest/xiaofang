//! 是方块人就下一百层
//!

use alloc::{collections::VecDeque, vec::Vec};
use cube_rand::CubeRng;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::WebColors,
    Pixel,
};
use embedded_hal::delay::DelayNs;
use esp_println::dbg;

use crate::{App, Gd, Position, RNG};

#[derive(Debug)]
pub struct CubeManGame {
    man: CubeMan,
    floors: VecDeque<Option<Floor>>,
    floor_gen: FloorGen,
    depth: usize,
    score: u8,
    game_over: bool,
    /// ms
    waiting_time: u32,
}

impl CubeManGame {
    pub fn new() -> Self {
        let floors = FloorGen::init();

        Self {
            man: CubeMan::new((0, 0).into()),
            floors,
            floor_gen: FloorGen::new(),
            depth: 0,
            score: 0,
            game_over: false,
            waiting_time: 180,
        }
    }

    pub fn run<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        app.gd = Gd::default();

        loop {
            if self.game_over {
                // TODO 历史最高分动画,音乐
                app.ledc.draw_score(self.score);
                app.delay.delay_ms(3000_u32);
                break;
            }
            app.gravity_direction();

            {
                self.floors.pop_front();

                let span = unsafe {
                    CubeRng(RNG.assume_init_mut().random() as u64).random_range(2..=3) as usize
                };

                let mut floor = self.floor_gen.floor(self.depth);
                if let Some(ref mut floor) = floor {
                    // floor
                    //     .data
                    //     .iter_mut()
                    //     .for_each(|f| f.0.y += pb.data[0].0.y + span as i32);
                }
                self.floors.push_back(floor);

                self.floors.iter_mut().for_each(|f| {
                    if let Some(f) = f {
                        f.data.iter_mut().for_each(|f| f.0.y -= 1);
                    }
                });
            }

            dbg!(&self.man.pos);
            dbg!(&self.floors);

            self.r#move(app);
            // TODO 移动音效,得分音效和画面效果,死亡音效
            self.draw(app);

            app.delay.delay_ms(self.waiting_time);
            self.depth += 1;
        }
    }

    fn r#move<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        let np = self.man.next_pos(app);
        if self.outside(&np) {
            self.game_over = true;
        } else if self.hit_wall(&np) {
            return;
        } else {
            self.man.r#move(app);
            // 如果下面是楼层,在停在楼层上
            if let Some(floor) = Self::on_floor(
                &self
                    .floors
                    .iter()
                    .filter(|f| f.is_some())
                    .map(|f| f.clone().unwrap())
                    .collect::<Vec<_>>(),
                &np,
            ) {
                self.moving_on_floor(&floor, app);
                // TODO 随地板一起向上运动
            } else {
                self.man.fall();
            }
            // 左右移动时间
            app.delay.delay_ms(70_u32);
        }
    }

    fn calc_score(&mut self) {
        self.score += 1;
    }

    fn outside(&self, pos: &Position) -> bool {
        pos.y < 0 || pos.y >= 8
    }

    fn hit_wall(&self, pos: &Position) -> bool {
        pos.x < 0 || pos.x >= 8
    }

    /// 是否在地板上
    fn on_floor(floors: &[Floor], pos: &Position) -> Option<Floor> {
        let floor = floors.iter().find(|f| {
            f.data
                .iter()
                .any(|p| p.0.x == pos.x as i32 && p.0.y == pos.y as i32 + 1)
        });

        floor.cloned()
    }

    /// 在地板上的移动
    fn moving_on_floor<T: esp_hal::i2c::Instance>(&mut self, floor: &Floor, app: &mut App<T>) {
        match &floor.r#type {
            FloorType::Normal => {}
            FloorType::Fragile(t) => {
                let mut fds = floor.data.clone();
                (0..3).for_each(|_| {
                    for fd in fds.iter_mut() {
                        fd.1 = BinaryColor::from(fd.1).invert().into();
                    }
                    app.ledc.write_pixels(fds.clone());
                    app.delay.delay_ms(50_u32);
                });

                for fd in fds.iter_mut() {
                    fd.1 = BinaryColor::Off.into();
                }
                app.ledc.write_pixels(fds);

                app.delay.delay_ms(*t);
            }
            FloorType::Conveyor(cd) => {
                if app.gd == Gd::Left || app.gd == Gd::Right {
                    return;
                }
                match cd {
                    ConveyorDir::Clockwise => {
                        if self.man.pos.x + 1 < 8 {
                            self.man.pos.x += 1;
                        }
                    }
                    ConveyorDir::Counterclockwise => {
                        if self.man.pos.x - 1 < 8 {
                            self.man.pos.x -= 1;
                        }
                    }
                }
            }
            FloorType::Spring(h) => {
                self.man.pos.y -= *h as i8;
            }
        }
    }

    pub fn draw<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        log::info!("level {:?}", self.depth);
        log::info!("falled floor {:?}", self.floors);
        app.ledc.clear_with_color(BinaryColor::Off.into());
        let mut pixels = Vec::<Pixel<Rgb888>>::new();
        // 地板
        // 将地图坐标转换为led坐标
        for y in 0..8 {
            if let Some(Some(floor)) = self.floors.get(y) {
                // for x in 0..8 {
                for d in floor.data.iter() {
                    pixels.push(Pixel((d.0.x, d.0.y).into(), d.1));
                }
                // }
            }
        }
        log::info!("draw pixels : {pixels:?}");
        // for floor in self.floors.iter() {
        //     for d in floor.data.iter() {
        //         // pixels.push(Pixel((d.x as i32, d.y as i32).into(), floor.color));
        //         pixels.push(d.clone());
        //     }
        // }
        app.ledc.write_pixels(pixels);

        // 人物
        let mp = self.man.pos;
        app.ledc
            .write_pixel(Pixel((mp.x as i32, mp.y as i32).into(), self.man.color));

        // 将地图坐标转换为led坐标
        // for y in 0..8 {
        //     for x in 0..8 {
        //         if let Some(pixel) = self.vision.data[y][x] {
        //             pixels.push(pixel);
        //         }
        //     }
        // }
        //
        // let pp = {
        //     let pp = self.man.pos;
        //     let vp = self.vision.pos;
        //     Pixel(
        //         ((pp.x - vp.x) as i32, (pp.y - vp.y) as i32).into(),
        //         self.man.color,
        //     )
        // };
        // pixels.push(pp);
        // app.ledc.write_pixels(pixels);
    }
}

/// 传送带旋转方向
#[derive(Debug, Clone, Copy)]
enum ConveyorDir {
    /// 顺时针
    Clockwise,
    /// 逆时针
    Counterclockwise,
}

/// 地板类型
#[derive(Debug, Clone, Copy)]
enum FloorType {
    /// 正常
    Normal,
    /// 易碎(碎裂时间)
    Fragile(u32),
    /// 传送带(传送带旋转方向)
    Conveyor(ConveyorDir),
    /// 弹簧(反弹的高度)
    Spring(u8),
}

// impl FloorType {
//     fn random() -> Self {
//         todo!()
//     }
// }

/// 地板
#[derive(Debug, Clone)]
struct Floor {
    /// 类型
    r#type: FloorType,
    data: Vec<Pixel<Rgb888>>,
}

impl Floor {
    fn new(ft: FloorType, data: &[Position]) -> Self {
        Self {
            r#type: ft,
            data: data.into_iter().map(|p| p.into()).collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug)]
struct FloorGen {
    pos: Position,
    data: VecDeque<Option<Floor>>,
}

impl FloorGen {
    fn init() -> VecDeque<Option<Floor>> {
        let mut floors = VecDeque::<Option<Floor>>::new();
        (0..8).for_each(|_| floors.push_back(None));
        floors
    }

    fn new() -> Self {
        Self {
            pos: (0, 0).into(),
            data: Self::init(),
        }
    }

    /// 随机生成地板
    fn random(level: usize) -> Option<Floor> {
        // 概率生成地板
        let per =
            unsafe { CubeRng(RNG.assume_init_mut().random() as u64).random_range(1..=10) as usize };
        if per < 7 {
            return None;
        }

        // 地板长度
        let len =
            unsafe { CubeRng(RNG.assume_init_mut().random() as u64).random_range(3..=5) as usize };
        let mut data = Vec::<Position>::with_capacity(len);
        for i in 0..len {
            data.push((i, 0).into());
        }

        // TODO 根据关卡等级生成地板
        let floor = if level <= 50 {
            Floor::new(FloorType::Normal, &data)
        } else if level > 50 && level <= 150 {
            Floor::new(FloorType::Fragile(500), &data)
        } else {
            Floor::new(FloorType::Conveyor(ConveyorDir::Clockwise), &data)
            // Floor::new(FloorType::Conveyor(ConveyorDir::Counterclockwise), &data)
            // Floor::new(FloorType::Spring(2), &data)
        };
        Some(floor)
    }

    fn floor(&mut self, level: usize) -> Option<Floor> {
        let mut floor = Self::random(level);
        if let Some(ref mut floor) = floor {
            floor.data.iter_mut().for_each(|f| f.0.y = 8);
        }
        floor
    }

    fn floors(&mut self, level: usize) -> VecDeque<Option<Floor>> {
        let mut floors = VecDeque::<Option<Floor>>::new();
        floors.push_back(None);
        floors.push_back(None);
        let mut floor = Self::random(level);
        if let Some(ref mut floor) = floor {
            floor.data.iter_mut().for_each(|f| f.0.y = 8);
        }
        floors.push_back(floor);

        let span =
            unsafe { CubeRng(RNG.assume_init_mut().random() as u64).random_range(2..=6) as usize };
        for _ in 0..span {
            floors.push_back(None);
        }
        let mut floor = Self::random(level);
        if let Some(ref mut floor) = floor {
            floor.data.iter_mut().for_each(|f| f.0.y += span as i32);
        }
        floors.push_back(floor);

        floors

        // (0..2)
        //     .map(|_| {
        //         let span = unsafe {
        //             CubeRng(RNG.assume_init_mut().random() as u64).random_range(2..=6) as usize
        //         };
        //         let mut floor = Self::random(level);
        //         floor.data.iter_mut().for_each(|f| f.0.y += span as i32);
        //         Some(floor)
        //     })
        //     .collect::<VecDeque<_>>()
    }
}

#[derive(Debug)]
struct CubeMan {
    /// 位置
    pos: Position,
    /// 移动速度
    move_speed: f32,
    /// 下落速度
    fall_speed: f32,
    /// 颜色
    color: Rgb888,
}

impl CubeMan {
    fn new(pos: Position) -> Self {
        Self {
            pos,
            fall_speed: 1.0,
            move_speed: 1.0,
            color: Rgb888::CSS_ORANGE_RED,
        }
    }

    fn next_pos<T: esp_hal::i2c::Instance>(&self, app: &mut App<T>) -> Position {
        let mut pos = self.pos;
        match app.gd {
            Gd::Right => pos.x += 1,
            Gd::Left => pos.x -= 1,
            _ => {}
        };
        pos
    }

    fn r#move<T: esp_hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.pos = self.next_pos(app);
    }

    fn fall(&mut self) {
        self.pos.y += 1;
    }
}
