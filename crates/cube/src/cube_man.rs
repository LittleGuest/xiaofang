#![doc = include_str!("../../../rfcs/006_cube_man.md")]

use crate::{Ad, App, buzzer};
use alloc::{collections::VecDeque, vec::Vec};
use cube_rand::CubeRng;
use embassy_time::Timer;
use embedded_graphics::{geometry::Point, pixelcolor::RgbColor};
use embedded_graphics_core::{
    Pixel,
    pixelcolor::{BinaryColor, Rgb888},
    prelude::WebColors,
};
use esp_hal::rng::Rng;

/// 是方块人就下一百层
#[derive(Debug)]
pub struct CubeManGame {
    man: CubeMan,
    floors: VecDeque<Option<Floor>>,
    floor_gen: FloorGen,
    depth: usize,
    score: u8,
    pub highest: u8,
    game_over: bool,
    /// ms
    waiting_time: u64,
    /// 得分闪烁状态
    score_flash: bool,
}

impl Default for CubeManGame {
    fn default() -> Self {
        Self::new()
    }
}

impl CubeManGame {
    pub fn new() -> Self {
        let mut floors = FloorGen::init();
        // 人物最开始站在正常的楼梯上
        let data: Vec<Point> = (2..=4).map(|x| Point::new(x, 7)).collect();
        floors[2] = Some(Floor::new(FloorType::Normal, &data));

        Self {
            man: CubeMan::new((3, 5).into()),
            floors,
            floor_gen: FloorGen::new(),
            depth: 0,
            score: 0,
            highest: 0,
            game_over: false,
            waiting_time: 230,
            score_flash: false,
        }
    }

    pub async fn run(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.ad = Ad::default();

        loop {
            if self.game_over {
                buzzer::cube_man_die().await;
                app.ledc.draw_score(self.score);
                Timer::after_millis(1500).await;
                if self.score > self.highest {
                    self.highest = self.score;
                    app.face.break_record_animate(&mut app.ledc).await;
                }
                Timer::after_millis(500).await;
                break;
            }
            app.acc_direction();
            app.check_pause().await;
            {
                self.floors.pop_front();
                self.floors
                    .push_back(self.floor_gen.floor(self.depth, &mut app.rng));
                self.floors.iter_mut().for_each(|f| {
                    if let Some(f) = f {
                        f.data.iter_mut().for_each(|f| f.0.y -= 1);
                    }
                });
            }
            self.r#move(app).await;
            self.draw(app);
            // 随游戏进度加快下落速度,越往后越快(下限 80ms)
            self.waiting_time = 230u64.saturating_sub(self.depth as u64 * 2).max(80);

            Timer::after_millis(self.waiting_time).await;
            self.depth += 1;
        }
    }

    async fn r#move(&mut self, app: &mut App<'_>) {
        let np = self.man.next_pos(app);
        if self.outside(&np) {
            self.game_over = true;
        } else if self.hit_wall(&np) {
        } else {
            self.man.r#move(app);
            // 左右移动音效
            if app.ad == Ad::Left || app.ad == Ad::Right {
                buzzer::cube_man_move().await;
            }
            // 如果下面是楼梯,在停在楼梯上
            if let Some(floor) = Self::on_floor(
                &self
                    .floors
                    .iter()
                    .filter_map(|f| f.clone())
                    .collect::<Vec<_>>(),
                &self.man.pos,
            ) {
                // 随楼梯一起向上运动
                self.man.up();
                self.calc_score();
                self.score_flash = true;
                buzzer::cube_man_score().await;
                self.moving_on_floor(&floor, app).await;
            } else {
                self.man.fall();
            }
        }
    }

    fn calc_score(&mut self) {
        self.score += 1;
    }

    fn outside(&self, pos: &Point) -> bool {
        pos.y < 0 || pos.y >= 8
    }

    fn hit_wall(&self, pos: &Point) -> bool {
        pos.x < 0 || pos.x >= 8
    }

    /// 是否在楼梯上
    fn on_floor(floors: &[Floor], pos: &Point) -> Option<Floor> {
        let floor = floors.iter().find(|f| {
            let min = f.data.iter().min_by(|x, y| x.cmp(y)).unwrap();
            let max = f.data.iter().max_by(|x, y| x.cmp(y)).unwrap();
            f.data
                .iter()
                .any(|p| min.0.x <= pos.x && pos.x <= max.0.x && p.0.y == pos.y + 1)
        });
        floor.cloned()
    }

    /// 在楼梯上的移动
    async fn moving_on_floor(&mut self, floor: &Floor, app: &mut App<'_>) {
        match &floor.r#type {
            FloorType::Normal => {}
            FloorType::Fragile(t) => {
                let mut fds = floor.data.clone();
                for _ in 0..3 {
                    for fd in fds.iter_mut() {
                        fd.1 = BinaryColor::from(fd.1).invert().into();
                    }
                    app.ledc.write_pixels(fds.clone());
                    Timer::after_millis(50).await;
                }

                for fd in fds.iter_mut() {
                    fd.1 = BinaryColor::Off.into();
                }
                app.ledc.write_pixels(fds);

                Timer::after_millis(*t).await;
            }
            FloorType::Conveyor(cd) => {
                // 玩家操纵时不受传送带影响
                if app.ad == Ad::Left || app.ad == Ad::Right {
                    return;
                }

                // 传送带旋转动画:两端常绿,中间从一端向另一端扫动
                let mut cols: Vec<Point> = floor.data.iter().map(|p| p.0).collect();
                cols.sort_by_key(|p| p.x);
                if cols.len() >= 3 {
                    let ints: Vec<Point> = cols[1..cols.len() - 1].to_vec();
                    for _ in 0..2 {
                        match cd {
                            ConveyorDir::Clockwise => {
                                // 中间从左到右扫动
                                for i in 0..ints.len() {
                                    let px = ints
                                        .iter()
                                        .enumerate()
                                        .map(|(j, p)| {
                                            let c = if j <= i {
                                                Rgb888::CSS_WHITE
                                            } else {
                                                RgbColor::GREEN
                                            };
                                            Pixel(*p, c)
                                        })
                                        .collect::<Vec<_>>();
                                    app.ledc.write_pixels(px);
                                    Timer::after_millis(30).await;
                                }
                            }
                            ConveyorDir::Counterclockwise => {
                                // 中间从右到左扫动
                                for i in (0..ints.len()).rev() {
                                    let px = ints
                                        .iter()
                                        .enumerate()
                                        .map(|(j, p)| {
                                            let c = if j >= i {
                                                Rgb888::CSS_WHITE
                                            } else {
                                                RgbColor::GREEN
                                            };
                                            Pixel(*p, c)
                                        })
                                        .collect::<Vec<_>>();
                                    app.ledc.write_pixels(px);
                                    Timer::after_millis(30).await;
                                }
                            }
                        }
                    }
                    // 恢复为绿色
                    app.ledc.write_pixels(floor.data.clone());
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
                self.man.pos.y -= *h as i32;
            }
        };
    }

    pub fn draw(&mut self, app: &mut App<'_>) {
        app.ledc.clear_with_color(BinaryColor::Off.into());
        // 楼梯
        app.ledc.write_pixels(
            self.floors
                .iter()
                .filter(|f| f.is_some())
                .cloned()
                .flat_map(|f| f.unwrap().data),
        );

        // 人物（得分时闪白）
        let mp = self.man.pos;
        let color = if self.score_flash {
            Rgb888::CSS_WHITE
        } else {
            self.man.color
        };
        app.ledc.write_pixel(Pixel((mp.x, mp.y).into(), color));
        self.score_flash = false;
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

/// 楼梯类型
#[derive(Debug, Clone, Copy)]
enum FloorType {
    /// 正常
    Normal,
    /// 易碎(碎裂时间)
    Fragile(u64),
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

/// 楼梯
#[derive(Debug, Clone)]
struct Floor {
    /// 类型
    r#type: FloorType,
    data: Vec<Pixel<Rgb888>>,
}

impl Floor {
    fn new(ft: FloorType, data: &[Point]) -> Self {
        match ft {
            FloorType::Normal => Self {
                r#type: ft,
                data: data
                    .iter()
                    .map(|p| Pixel((p.x, p.y).into(), RgbColor::WHITE))
                    .collect::<Vec<_>>(),
            },
            FloorType::Fragile(_) => Self {
                r#type: ft,
                data: data
                    .iter()
                    .map(|p| Pixel((p.x, p.y).into(), Rgb888::new(0x80, 0x80, 0x80)))
                    .collect::<Vec<_>>(),
            },
            FloorType::Conveyor(_) => Self {
                r#type: ft,
                data: data
                    .iter()
                    .map(|p| Pixel((p.x, p.y).into(), RgbColor::GREEN))
                    .collect::<Vec<_>>(),
            },
            FloorType::Spring(_) => Self {
                r#type: ft,
                data: data
                    .iter()
                    .map(|p| Pixel((p.x, p.y).into(), RgbColor::YELLOW))
                    .collect::<Vec<_>>(),
            },
        }
    }
}

#[derive(Debug)]
struct FloorGen {
    pos: Point,
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

    /// 随机生成楼梯
    fn random(level: usize, rng: &mut Rng) -> Option<Floor> {
        // 概率生成楼梯
        let per = CubeRng(rng.random() as u64).random_range(1..=10);
        if per < 7 {
            return None;
        }

        // 楼梯长度随等级递减（增加难度）
        let max_len = if level <= 30 {
            5
        } else if level <= 100 {
            4
        } else {
            3
        };
        let len = CubeRng(rng.random() as u64).random_range(3..=max_len);
        let start_x = CubeRng(rng.random() as u64).random_range(0..=(8 - len)) as i32;
        let mut data = Vec::<Point>::with_capacity(len);
        for i in 0..len {
            data.push(Point::new(start_x + i as i32, 0));
        }

        // 随机选择楼梯类型,概率参照 RFC: 正常70% / 易碎10% / 传送带10% / 弹簧10%
        let r = CubeRng(rng.random() as u64).random_range(1..=10);
        let floor = match r {
            1 => Floor::new(FloorType::Fragile(500), &data),
            2 => {
                let dir = if CubeRng(rng.random() as u64).random_range(0..=1) == 0 {
                    ConveyorDir::Clockwise
                } else {
                    ConveyorDir::Counterclockwise
                };
                Floor::new(FloorType::Conveyor(dir), &data)
            }
            3 => Floor::new(FloorType::Spring(2), &data),
            _ => Floor::new(FloorType::Normal, &data),
        };
        Some(floor)
    }

    /// 生成楼梯，y坐标为8
    fn floor(&mut self, level: usize, rng: &mut Rng) -> Option<Floor> {
        let mut floor = Self::random(level, rng);
        if let Some(ref mut floor) = floor {
            floor.data.iter_mut().for_each(|f| f.0.y = 8);
        }
        floor
    }

    fn floors(&mut self, level: usize, rng: &mut Rng) -> VecDeque<Option<Floor>> {
        let mut floors = VecDeque::<Option<Floor>>::new();
        floors.push_back(None);
        floors.push_back(None);
        let mut floor = Self::random(level, rng);
        if let Some(ref mut floor) = floor {
            floor.data.iter_mut().for_each(|f| f.0.y = 8);
        }
        floors.push_back(floor);

        let span = CubeRng(rng.random() as u64).random_range(2..=6);
        for _ in 0..span {
            floors.push_back(None);
        }
        let mut floor = Self::random(level, rng);
        if let Some(ref mut floor) = floor {
            floor.data.iter_mut().for_each(|f| f.0.y += span as i32);
        }
        floors.push_back(floor);

        floors

        // (0..2)
        //     .map(|_| {
        //         let span = unsafe {
        //             CubeRng(rng().random() as u64).random_range(2..=6) as usize
        //         };
        //         let mut floor = Self::random(level);
        //         floor.data.iter_mut().for_each(|f| f.0.y += span as i32);
        //         Some(floor)
        //     })
        //     .collect::<VecDeque<_>>()
    }
}

/// 方块人
#[derive(Debug)]
struct CubeMan {
    /// 位置
    pos: Point,
    /// 移动速度
    move_speed: f32,
    /// 下落速度
    fall_speed: f32,
    /// 颜色
    color: Rgb888,
}

impl CubeMan {
    fn new(pos: Point) -> Self {
        Self {
            pos,
            fall_speed: 1.0,
            move_speed: 1.0,
            color: Rgb888::CSS_ORANGE_RED,
        }
    }

    fn next_pos(&self, app: &mut App<'_>) -> Point {
        let mut pos = self.pos;
        match app.ad {
            Ad::Right => pos.x += 1,
            Ad::Left => pos.x -= 1,
            _ => {}
        };
        pos
    }

    fn r#move(&mut self, app: &mut App<'_>) {
        self.pos = self.next_pos(app);
    }

    /// 下落
    fn fall(&mut self) {
        self.pos.y += 1;
    }

    /// 向上
    fn up(&mut self) {
        self.pos.y -= 1;
    }
}
