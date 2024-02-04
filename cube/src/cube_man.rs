//! 是方块人就下一百层
//!

use alloc::vec::Vec;
use cube_rand::CubeRng;
use embedded_graphics_core::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::WebColors,
    Pixel,
};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use crate::{mapping, App, Direction, Gd, Position, RNG};

#[derive(Debug)]
pub struct CubeManGame {
    man: CubeMan,
    floors: Vec<Floor>,
    score: u8,
    game_over: bool,
    /// ms
    waiting_time: u32,
}

impl CubeManGame {
    pub fn new() -> Self {
        Self {
            man: CubeMan::new((0, 0).into()),
            floors: FloorGen::random(),
            score: 0,
            game_over: false,
            waiting_time: 180,
        }
    }

    pub fn run<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear();
        app.gd = Gd::default();

        loop {
            if self.game_over {
                // TODO 历史最高分动画,音乐
                self.draw_score(app);
                app.delay.delay_ms(3000_u32);
                break;
            }
            app.gravity_direction();
            self.r#move(app.gd, app);
            // TODO 移动音效,得分音效和画面效果,死亡音效
            self.draw(app);

            app.delay.delay_ms(self.waiting_time);
        }
    }

    fn r#move<T: hal::i2c::Instance>(&mut self, gd: Gd, app: &mut App<T>) {
        let np = self.man.next_pos(app);
        let on_floor = self.on_floor(np);
        if self.outside(np) {
            self.game_over = true;
        } else if self.hit_wall(np) {
            return;
        } else {
            self.man.r#move(app);
            // 如果下面是楼层,在停在楼层上
            if !on_floor {
                self.man.fall();
            }
            app.delay.delay_ms(70_u32);
        }
    }

    fn calc_score(&mut self) {
        self.score += 1;
    }

    fn outside(&self, pos: Position) -> bool {
        pos.y < 0 || pos.y >= 8
    }

    fn hit_wall(&self, pos: Position) -> bool {
        pos.x < 0 || pos.x >= 8
    }

    fn on_floor(&self, pos: Position) -> bool {
        self.floors
            .iter()
            .any(|f| f.data.iter().any(|p| p.x == pos.x && p.y == pos.y + 1))
    }

    pub fn draw<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        app.ledc.clear_with_color(BinaryColor::Off.into());
        let mut pixels = Vec::<Pixel<Rgb888>>::new();
        // 地板
        for floor in self.floors.iter() {
            for d in floor.data.iter() {
                pixels.push(Pixel((d.x as i32, d.y as i32).into(), floor.color));
            }
        }
        app.ledc.write_pixels(pixels);

        // 人物
        let mp = self.man.pos;
        app.ledc
            .write_pixel(Pixel((mp.x as i32, mp.y as i32).into(), self.man.color));
    }

    fn draw_score<T: hal::i2c::Instance>(&self, app: &mut App<T>) {
        app.ledc.clear();

        let dn = self.score / 10;
        let sn = self.score % 10;
        let dn = mapping::num_map(dn);
        let mut sn = mapping::num_map(sn);

        let mut buf_work = [0; 8];
        (0..8).for_each(|i| buf_work[i] = dn[i]);

        (0..8).for_each(|i| sn[i] >>= 4);
        (0..8).for_each(|i| buf_work[i] |= sn[i]);
        (0..8).for_each(|i| buf_work[i] >>= 1);

        app.ledc.write_bytes(buf_work);
    }
}

/// 传送带旋转方向
#[derive(Debug)]
enum ConveyorDir {
    /// 顺时针
    Clockwise,
    /// 逆时针
    Counterclockwise,
}

/// 地板类型
#[derive(Debug)]
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
#[derive(Debug)]
struct Floor {
    /// 类型
    r#type: FloorType,
    /// 颜色
    color: Rgb888,
    data: Vec<Position>,
}

impl Floor {
    fn new(ft: FloorType, data: &[Position]) -> Self {
        Self {
            r#type: ft,
            color: BinaryColor::On.into(),
            data: data.into(),
        }
    }
}

struct FloorGen;

impl FloorGen {
    /// - 楼层的长度最小为 3,最长长度 5
    /// - 楼层的类型
    ///   - 正常的楼层
    ///   - 陷阱楼层
    ///     - 易碎楼层: 用闪烁来表示易碎,当人物站在该楼层上时,该楼层急速闪烁 2s 后碎裂,人物继续往下掉;
    ///     - 传送带楼层:
    ///       - 顺时针旋转: 两端不闪烁,中间从左到右闪烁,即该楼层的长度最少是 4
    ///       - 逆时针旋转: 两端不闪烁,中间从右到左闪烁,即该楼层的长度最少是 4
    ///     - 弹簧楼层: 用黄色表示,当人物站在该楼层上时,将会被弹起 3 格高度
    ///
    /// 楼层的不同类型根据设置的频率随机生成:
    ///
    /// - 正常的楼层 : 70%
    /// - 陷阱楼层 : 30%
    ///   - 易碎楼层: 10%
    ///   - 传送带楼层: 10%
    ///   - 弹簧楼层: 10%
    fn random() -> Vec<Floor> {
        // let len =
        //     unsafe { CubeRng(RNG.assume_init_mut().random() as u64).random_range(3..=5) as usize };
        // let data = Vec::<Position>::with_capacity(len);

        let mut floors = Vec::<Floor>::with_capacity(100);
        floors.push(Floor::new(
            FloorType::Normal,
            &[(2, 1).into(), (3, 1).into(), (4, 1).into()],
        ));
        floors.push(Floor::new(
            FloorType::Normal,
            &[(3, 3).into(), (4, 3).into(), (5, 3).into()],
        ));
        floors.push(Floor::new(
            FloorType::Normal,
            &[(1, 5).into(), (2, 5).into(), (3, 5).into(), (4, 5).into()],
        ));
        floors.push(Floor::new(
            FloorType::Normal,
            &[(5, 7).into(), (6, 7).into(), (7, 7).into(), (8, 7).into()],
        ));

        floors
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

    fn next_pos<T: hal::i2c::Instance>(&self, app: &mut App<T>) -> Position {
        let mut pos = self.pos;
        match app.gd {
            Gd::Right => pos.x += 1,
            Gd::Left => pos.x -= 1,
            _ => {}
        };
        pos
    }

    fn r#move<T: hal::i2c::Instance>(&mut self, app: &mut App<T>) {
        self.pos = self.next_pos(app);
    }

    fn fall(&mut self) {
        self.pos.y += 1;
    }
}
