#![doc = include_str!("../../../rfcs/007_sokoban.md")]

use alloc::vec::Vec;

use embassy_time::Timer;
use embedded_graphics_core::{
    Pixel,
    pixelcolor::{BinaryColor, Rgb888},
    prelude::WebColors,
};

use crate::{
    Ad, App, Point, buzzer,
    map::{Map, MapCell, Vision},
    player::Player,
};

/// 预设的XSB关卡列表
const SOKOBAN_LEVELS: &[&str] = &[
    // Level 1: 最简单的入门关（1箱1目标）
    "
########
#------#
#-$.@--#
#------#
#------#
#------#
#------#
########
",
    // Level 2: 2箱2目标
    "
########
#------#
#-$.@--#
#--.$--#
#--.---#
#------#
#------#
########
",
    // Level 3: 原始关卡
    "
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
",
    // Level 4: 3箱3目标
    "
----#####----------
----#---#----------
----#$--#----------
--###--$##---------
--#--$-$-#---------
###-#-##-#---######
#---#-##-#####--..#
#-$--$----------..#
#####-###-#@##--..#
----#-----#########
----#######--------
",
];

/// 预设的LURD解答关卡列表(运行时用from_lurd还原成地图)
const SOKOBAN_LURD_LEVELS: &[&str] = &[
    // Classic level 1 (Thinking Rabbit)
    "ullluuuLUllDlldddrRRRRRRRRRRdrUllllllluuululldDDuu\
lldddrRRRRRRRRRRRRlllllllluuulLulDDDuulldddrRRRRRR\
RRRRRllllllluuulluuurDDuullDDDDDuulldddrRRRRRRRRRR\
uRRlDllllllluuuLLulDDDuulldddrRRRRRRRRRRdRRlUlllll\
lllllllulldRRRRRRRRRRRRRuRDldR",
];

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
    /// 当前关卡编号
    level: usize,
}

impl Default for Sokoban {
    fn default() -> Self {
        Self::new()
    }
}

impl Sokoban {
    pub fn new() -> Self {
        Self::new_with_level(0)
    }

    /// 按关卡序号生成地图:预设的XSB关卡用尽后,从LURD解答还原关卡
    fn level_map(level: usize) -> SokobanMap {
        if level < SOKOBAN_LEVELS.len() {
            SokobanMap::from_xsb(SOKOBAN_LEVELS[level])
        } else {
            let idx = (level - SOKOBAN_LEVELS.len()) % SOKOBAN_LURD_LEVELS.len();
            SokobanMap::from_lurd(SOKOBAN_LURD_LEVELS[idx])
        }
    }

    pub fn new_with_level(level: usize) -> Self {
        let map = Self::level_map(level);
        let width = map.map.width;
        let height = map.map.height;
        let player = Player::new((map.player.0.0.x, map.player.0.0.y).into());
        let mut vision = Vision::new(width, height, player.pos);
        vision.update_data(&map.map);
        Sokoban {
            map,
            player,
            vision,
            waiting_time: 300,
            game_over: false,
            level,
        }
    }

    /// 重载当前关卡
    fn reload_level(&mut self) {
        let map = Self::level_map(self.level);
        self.player = Player::new((map.player.0.0.x, map.player.0.0.y).into());
        self.vision = Vision::new(map.map.width, map.map.height, self.player.pos);
        self.vision.update_data(&map.map);
        self.map = map;
        self.game_over = false;
    }

    pub async fn run(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.ad = Ad::default();

        loop {
            Timer::after_millis(self.waiting_time).await;

            if self.game_over {
                // 过关庆祝
                buzzer::sokoban_complete().await;
                for _ in 0..3 {
                    app.ledc.clear_with_color(Rgb888::CSS_GREEN);
                    Timer::after_millis(200).await;
                    app.ledc.clear();
                    Timer::after_millis(200).await;
                }
                Timer::after_millis(500).await;

                // 进入下一关
                self.level += 1;
                self.reload_level();
                continue;
            }

            // 下甩暂停
            app.check_pause().await;

            app.acc_direction();

            if !self.hit_wall(app) {
                // 不撞墙，是否在推动箱子，能否推动箱子，能一起移动
                let can_push = self.push_box(app);
                if can_push {
                    let moved = self.player.r#move(app.ad);
                    if moved {
                        buzzer::sokoban_move().await;
                    }
                    // 玩家移动之后视野数据改变
                    self.vision.update(app.ad, &self.map.map);
                    self.check_complete();
                }
            }
            self.draw(app);
        }
    }

    /// 推动箱子
    fn push_box(&mut self, app: &mut App<'_>) -> bool {
        let Point { x, y } = self.player.next_pos(app.ad);
        let boxs = self.map.boxs.clone();
        for (cp, ct) in self.map.boxs.iter_mut() {
            // 下一个位置是箱子且能推动则推箱子
            if TargetType::Box.eq(ct) && cp.0.x == x && cp.0.y == y {
                // 再下一个位置
                let mut boxp = cp.0;
                match app.ad {
                    Ad::Front => boxp.y -= 1,
                    Ad::Right => boxp.x += 1,
                    Ad::Back => boxp.y += 1,
                    Ad::Left => boxp.x -= 1,
                    _ => {}
                };
                let is_box = boxs
                    .iter()
                    .any(|m| matches!(m.1, TargetType::Box) && m.0.0.x == boxp.x && m.0.0.y == boxp.y);
                let is_wall = self
                    .map
                    .map
                    .data
                    .iter()
                    .any(|m| matches!(m.1, TargetType::Wall) && m.0.0.x == boxp.x && m.0.0.y == boxp.y);
                if is_box || is_wall {
                    return false;
                }

                // 推动箱子
                match app.ad {
                    Ad::Front => cp.0.y -= 1,
                    Ad::Right => cp.0.x += 1,
                    Ad::Back => cp.0.y += 1,
                    Ad::Left => cp.0.x -= 1,
                    _ => {}
                };
            }
        }
        true
    }

    /// 检查是否过关：所有箱子都在目标点上
    fn check_complete(&mut self) {
        let goals = self.map.goals.iter().map(|b| b.0.0).collect::<Vec<_>>();
        let all = self.map.boxs.iter().all(|b| goals.contains(&b.0.0));
        self.game_over = all;
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
        // 箱子
        let goals = self.map.goals.iter().map(|m| m.0.0).collect::<Vec<_>>();
        for b in self.map.boxs.iter().map(|b| b.0) {
            // 青色表示箱子在目标点上
            let color = if goals.contains(&b.0) { Rgb888::CSS_CYAN } else { b.1 };
            let pp = Pixel(((b.0.x - vp.x), (b.0.y - vp.y)).into(), color);
            pixels.push(pp);
        }
        // 人物
        let pp = {
            let pp = self.player.pos;
            // 黄色表示玩家在目标点上
            let color = if goals.contains(&pp.into()) {
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
    fn hit_wall(&mut self, app: &mut App<'_>) -> bool {
        let Point { x, y } = self.player.next_pos(app.ad);
        let overlapping = x <= 0 || y <= 0 || x >= self.map.map.width as i32 - 1 || y >= self.map.map.height as i32 - 1;
        if overlapping {
            return true;
        }
        // 检测玩家下一个位置是否有墙
        self.map
            .map
            .data
            .iter()
            .any(|c| c.1 == TargetType::Wall && c.0.0.x == x && c.0.0.y == y)
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

/// LURD还原时地图格子的位标志
const LURD_FLOOR: u8 = 1;
const LURD_GOAL: u8 = 2;
const LURD_BOX: u8 = 4;
/// 被墙包围的不可达空格(砖块),游戏中和墙等价
const LURD_BRICK: u8 = 8;

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
    /// 根据XSB生成地图
    ///
    /// `-`、`_`和空格都表示地板,地板没有颜色,不进入地图数据
    fn from_xsb(xsb: &str) -> Self {
        let mut map = Self::default();
        // 宽高从行结构计算,不依赖数据格(地板行/列可能没有墙和目标点)
        let mut width = 0usize;
        let mut height = 0usize;
        for (y, line) in xsb.trim().lines().enumerate() {
            let y = y as i32;
            height += 1;
            width = width.max(line.chars().count());
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
                        map.map.data.push(goal);
                    }
                    '$' => {
                        map.boxs.push((Pixel((x, y).into(), Rgb888::CSS_BLUE), TargetType::Box));
                    }
                    '*' => {
                        map.boxs.push((Pixel((x, y).into(), Rgb888::CSS_BLUE), TargetType::Box));
                        let goal = (Pixel((x, y).into(), Rgb888::CSS_GREEN), TargetType::Goal);
                        map.goals.push(goal);
                        map.map.data.push(goal);
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
                    '-' | '_' | ' ' => {}
                    _ => {}
                }
            }
        }
        map.map.height = height;
        map.map.width = width;
        map
    }

    /// 根据LURD解答还原关卡地图
    ///
    /// 从解答的最后一步往前倒推,推算出正推的初始状态(玩家位置、箱子位置、目标点)。
    /// 解答必须是合法的,否则还原出的地图无意义。
    fn from_lurd(lurd: &str) -> Self {
        // 倒推计算玩家活动的边界范围
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (0i32, 0i32, 0i32, 0i32);
        let (mut x, mut y) = (0i32, 0i32);
        for c in lurd.chars().rev() {
            match c {
                'l' | 'L' => x += 1,
                'r' | 'R' => x -= 1,
                'u' | 'U' => y += 1,
                'd' | 'D' => y -= 1,
                _ => {}
            }
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        // 地图四周加3格墙边距,防止倒推时越界
        let width = (max_x - min_x + 1 + 6) as usize;
        let height = (max_y - min_y + 1 + 6) as usize;
        let mut cells = alloc::vec![0u8; width * height];
        let idx = |x: i32, y: i32| y as usize * width + x as usize;
        let (mut man_x, mut man_y) = (3 - min_x, 3 - min_y);
        cells[idx(man_x, man_y)] |= LURD_FLOOR;

        /// 倒推"左右推动":玩家反向移动,箱子反向移回,标记目标点/箱子/地板
        fn push_horizontal(cells: &mut [u8], width: usize, man_x: &mut i32, man_y: i32, step: i32) {
            *man_x += step;
            let i = man_y as usize * width + *man_x as usize;
            if cells[i] & LURD_BOX != 0 {
                cells[i] &= !LURD_BOX;
            } else {
                cells[i] |= LURD_GOAL;
            }
            *man_x -= step;
            cells[man_y as usize * width + *man_x as usize] |= LURD_BOX;
            *man_x -= step;
            cells[man_y as usize * width + *man_x as usize] |= LURD_FLOOR;
        }

        /// 倒推"上下推动"
        fn push_vertical(cells: &mut [u8], width: usize, man_x: i32, man_y: &mut i32, step: i32) {
            *man_y += step;
            let i = *man_y as usize * width + man_x as usize;
            if cells[i] & LURD_BOX != 0 {
                cells[i] &= !LURD_BOX;
            } else {
                cells[i] |= LURD_GOAL;
            }
            *man_y -= step;
            cells[*man_y as usize * width + man_x as usize] |= LURD_BOX;
            *man_y -= step;
            cells[*man_y as usize * width + man_x as usize] |= LURD_FLOOR;
        }

        // 倒推执行解答:小写字母是移动,大写字母是推动
        for c in lurd.chars().rev() {
            match c {
                'l' => {
                    man_x += 1;
                    cells[idx(man_x, man_y)] |= LURD_FLOOR;
                }
                'r' => {
                    man_x -= 1;
                    cells[idx(man_x, man_y)] |= LURD_FLOOR;
                }
                'u' => {
                    man_y += 1;
                    cells[idx(man_x, man_y)] |= LURD_FLOOR;
                }
                'd' => {
                    man_y -= 1;
                    cells[idx(man_x, man_y)] |= LURD_FLOOR;
                }
                'L' => push_horizontal(&mut cells, width, &mut man_x, man_y, -1),
                'R' => push_horizontal(&mut cells, width, &mut man_x, man_y, 1),
                'U' => push_vertical(&mut cells, width, man_x, &mut man_y, -1),
                'D' => push_vertical(&mut cells, width, man_x, &mut man_y, 1),
                _ => {}
            }
        }

        // 标记砖块:完全被墙包围的空格(不可达区域),用于后续裁剪出最简地图
        if width > 2 && height > 2 {
            let solid = |v: u8| v & (LURD_FLOOR | LURD_GOAL | LURD_BOX) == 0;
            for yy in 1..height - 1 {
                for xx in 1..width - 1 {
                    let i = yy * width + xx;
                    if cells[i] != 0 {
                        continue;
                    }
                    if solid(cells[i - width - 1])
                        && solid(cells[i - width])
                        && solid(cells[i - width + 1])
                        && solid(cells[i - 1])
                        && solid(cells[i + 1])
                        && solid(cells[i + width - 1])
                        && solid(cells[i + width])
                        && solid(cells[i + width + 1])
                    {
                        cells[i] = LURD_BRICK;
                    }
                }
            }
        }

        // 裁剪:去掉全为砖块的边缘行列(砖块不可达,可安全移除)
        let mut x0 = 1usize;
        let mut y0 = 1usize;
        let mut x1 = width.saturating_sub(2);
        let mut y1 = height.saturating_sub(2);
        while x0 <= x1 && (y0..=y1).all(|yy| cells[yy * width + x0] == LURD_BRICK) {
            x0 += 1;
        }
        while x0 <= x1 && (y0..=y1).all(|yy| cells[yy * width + x1] == LURD_BRICK) {
            x1 -= 1;
        }
        while y0 <= y1 && (x0..=x1).all(|xx| cells[y0 * width + xx] == LURD_BRICK) {
            y0 += 1;
        }
        while y0 <= y1 && (x0..=x1).all(|xx| cells[y1 * width + xx] == LURD_BRICK) {
            y1 -= 1;
        }
        if x1 < x0 || y1 < y0 {
            return Self::default();
        }

        let mut map = Self::default();
        for yy in y0..=y1 {
            for xx in x0..=x1 {
                let cell = cells[yy * width + xx];
                let px = (xx - x0) as i32;
                let py = (yy - y0) as i32;
                if cell & LURD_GOAL != 0 {
                    // 目标点
                    if man_x == xx as i32 && man_y == yy as i32 {
                        // 人在目标点上,目标点也要记录
                        map.player = (Pixel((px, py).into(), Rgb888::CSS_RED), TargetType::Man);
                        let goal = (Pixel((px, py).into(), Rgb888::CSS_GREEN), TargetType::Goal);
                        map.goals.push(goal);
                        map.map.data.push(goal);
                    } else if cell & LURD_BOX != 0 {
                        map.boxs
                            .push((Pixel((px, py).into(), Rgb888::CSS_BLUE), TargetType::Box));
                        let goal = (Pixel((px, py).into(), Rgb888::CSS_GREEN), TargetType::Goal);
                        map.goals.push(goal);
                        map.map.data.push(goal);
                    } else {
                        let goal = (Pixel((px, py).into(), Rgb888::CSS_GREEN), TargetType::Goal);
                        map.goals.push(goal);
                        map.map.data.push(goal);
                    }
                } else if cell & LURD_FLOOR != 0 {
                    // 地板
                    if man_x == xx as i32 && man_y == yy as i32 {
                        map.player = (Pixel((px, py).into(), Rgb888::CSS_RED), TargetType::Man);
                    } else if cell & LURD_BOX != 0 {
                        map.boxs
                            .push((Pixel((px, py).into(), Rgb888::CSS_BLUE), TargetType::Box));
                    }
                } else {
                    // 墙
                    map.map
                        .data
                        .push((Pixel((px, py).into(), Rgb888::CSS_WHITE), TargetType::Wall));
                }
            }
        }
        map.map.width = x1 - x0 + 1;
        map.map.height = y1 - y0 + 1;
        map
    }
}
