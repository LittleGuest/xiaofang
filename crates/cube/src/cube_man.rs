#![doc = include_str!("../../../rfcs/006_cube_man.md")]

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

use crate::{Ad, App, buzzer};

/// 是方块人就下一百层
#[derive(Debug)]
pub struct CubeManGame {
    man: CubeMan,
    floors: VecDeque<Option<Floor>>,
    depth: usize,
    score: u8,
    pub highest: u8,
    game_over: bool,
    /// ms
    waiting_time: u64,
    /// 得分闪烁剩余帧数
    score_flash: u8,
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
            depth: 0,
            score: 0,
            highest: 0,
            game_over: false,
            waiting_time: 230,
            score_flash: 0,
        }
    }

    pub async fn run(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.ad = Ad::default();

        loop {
            if self.game_over {
                buzzer::cube_man_die().await;
                // 死亡画面效果:方块人变红闪烁后消失
                let mp = self.man.pos;
                let mp = Point::new(mp.x, mp.y.clamp(0, 7));
                for _ in 0..3 {
                    app.ledc.clear();
                    app.ledc.write_pixel(Pixel(mp, Rgb888::CSS_RED));
                    Timer::after_millis(100).await;
                    app.ledc.clear();
                    Timer::after_millis(100).await;
                }
                app.ledc.draw_score(self.score);
                Timer::after_millis(1500).await;
                if self.score > self.highest {
                    self.highest = self.score;
                    // 破纪录音乐 + 动画
                    buzzer::cube_man_record().await;
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
                    .push_back(FloorGen::floor(self.depth, &mut app.rng, &self.floors));
                self.floors.iter_mut().for_each(|f| {
                    if let Some(f) = f {
                        f.data.iter_mut().for_each(|f| f.0.y -= 1);
                    }
                });
            }
            self.r#move(app).await;
            if !self.game_over {
                self.draw(app);
            }
            // 下落速度随游戏进度加快:用 fall_speed 驱动帧间隔(越往后越快,下限 80ms)
            self.man.fall_speed = 1.0 + self.depth as f32 * 0.02;
            self.waiting_time = (230.0 / self.man.fall_speed) as u64;
            if self.waiting_time < 80 {
                self.waiting_time = 80;
            }

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
                &self.floors.iter().filter_map(|f| f.clone()).collect::<Vec<_>>(),
                &self.man.pos,
            ) {
                // 随楼梯一起向上运动
                self.man.up();
                self.calc_score();
                self.score_flash = 4;
                buzzer::cube_man_score().await;
                let fragile = matches!(floor.r#type, FloorType::Fragile(_));
                self.moving_on_floor(&floor, app).await;
                // 易碎楼梯碎裂后从地图移除,人物继续往下掉
                if fragile {
                    self.floors = self
                        .floors
                        .iter()
                        .map(|f| {
                            if f.as_ref().is_some_and(|fl| fl.data == floor.data) {
                                None
                            } else {
                                f.clone()
                            }
                        })
                        .collect();
                }
            } else {
                self.man.fall();
            }
        }
        // 掉出视野(落到底部/随楼梯升至顶部之上)则游戏结束
        if self.outside(&self.man.pos) {
            self.game_over = true;
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
        floors
            .iter()
            .find(|f| {
                // f.data 理论上非空; 空时跳过该楼梯, 避免 min/max panic
                let (Some(min), Some(max)) = (
                    f.data.iter().min_by(|x, y| x.cmp(y)),
                    f.data.iter().max_by(|x, y| x.cmp(y)),
                ) else {
                    return false;
                };
                f.data
                    .iter()
                    .any(|p| min.0.x <= pos.x && pos.x <= max.0.x && p.0.y == pos.y + 1)
            })
            .cloned()
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
                                            let c = if j <= i { Rgb888::CSS_WHITE } else { RgbColor::GREEN };
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
                                            let c = if j >= i { Rgb888::CSS_WHITE } else { RgbColor::GREEN };
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
                        if self.man.pos.x > 0 {
                            self.man.pos.x -= 1;
                        }
                    }
                }
            }
            FloorType::Spring(h) => {
                // 弹簧反弹:渐变(先加速上升,顶点停留后再下落)
                self.spring_bounce(*h as i32, app).await;
            }
        };
    }

    /// 弹簧反弹效果:弹性上升(逐步加速)再于顶点短暂停留,之后交由重力自然回落
    async fn spring_bounce(&mut self, height: i32, app: &mut App<'_>) {
        let steps = height.max(1);
        let mut delay = 32u64;
        for _ in 0..steps {
            self.man.pos.y -= 1;
            self.draw(app);
            Timer::after_millis(delay).await;
            delay = (delay / 2).max(5); // 加速上升(渐变)
        }
        // 顶点短暂停留,制造"被弹起"的滞空感
        Timer::after_millis(70).await;
    }

    pub fn draw(&mut self, app: &mut App<'_>) {
        app.ledc.clear_with_color(BinaryColor::Off.into());
        // 楼梯
        app.ledc.write_pixels(
            self.floors
                .iter()
                .filter_map(|f| f.as_ref())
                .flat_map(|f| f.data.iter().cloned()),
        );

        // 人物（得分时连续闪烁变白）
        let mp = self.man.pos;
        let color = if self.score_flash > 0 {
            self.score_flash -= 1;
            Rgb888::CSS_WHITE
        } else {
            self.man.color
        };
        app.ledc.write_pixel(Pixel((mp.x, mp.y).into(), color));
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

/// 楼梯生成器(纯函数式,不保存状态)
#[derive(Debug)]
struct FloorGen;

impl FloorGen {
    fn init() -> VecDeque<Option<Floor>> {
        let mut floors = VecDeque::<Option<Floor>>::new();
        (0..8).for_each(|_| floors.push_back(None));
        floors
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
        // 随机选择楼梯类型,概率参照 RFC: 正常70% / 易碎10% / 传送带10% / 弹簧10%
        let r = CubeRng(rng.random() as u64).random_range(1..=10);

        let mut len = CubeRng(rng.random() as u64).random_range(3..=max_len);
        // 传送带楼梯长度至少为4(两端不闪烁,中间从左到右/右到左扫动)
        if r == 2 {
            if max_len < 4 {
                return None; // 当前等级上限不足4,无法生成合法传送带,跳过本帧
            }
            len = len.max(4);
        }

        let start_x = CubeRng(rng.random() as u64).random_range(0..=(8 - len)) as i32;
        let mut data = Vec::<Point>::with_capacity(len);
        for i in 0..len {
            data.push(Point::new(start_x + i as i32, 0));
        }

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

    /// 生成楼梯，y坐标为8；与前一个楼梯至少间隔一个人物高度(>=1 空行)
    fn floor(level: usize, rng: &mut Rng, floors: &VecDeque<Option<Floor>>) -> Option<Floor> {
        // 从生成位(栈底)向上找最近的真实楼梯，索引距离 >= 2 才允许生成，保证中间留有空行
        let dist = floors
            .iter()
            .rev()
            .position(|f| f.is_some())
            .map_or(usize::MAX, |p| p + 1);
        if dist < 2 {
            return None;
        }
        let mut floor = Self::random(level, rng);
        if let Some(ref mut floor) = floor {
            floor.data.iter_mut().for_each(|f| f.0.y = 8);
        }
        floor
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
        // 移动速度控制单次输入前进的格数(越大移得越快)
        let step = self.move_speed.max(1.0) as i32;
        match app.ad {
            Ad::Right => self.pos.x = (self.pos.x + step).min(7),
            Ad::Left => self.pos.x = (self.pos.x - step).max(0),
            _ => {}
        }
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
