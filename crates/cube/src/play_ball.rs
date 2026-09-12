#![doc = include_str!("../../../rfcs/010_play_ball.md")]

use crate::{Ad, App, buzzer};
use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    Pixel,
    geometry::Point,
    pixelcolor::{Rgb888, WebColors},
};
use esp_radio::esp_now::{BROADCAST_ADDRESS, EspNowWifiInterface, PeerInfo};

/// 游戏码: 对打球
const GAME_CODE: u8 = 0x01;
/// 状态: 结束
const ST_END: u8 = 0x00;
/// 状态: 寻找中(Host 广播)
const ST_SEEKING: u8 = 0x01;
/// 状态: 加入确认(Client 单播回复)
const ST_JOIN: u8 = 0x02;
/// 状态: START/游戏中
const ST_GAME: u8 = 0x03;

/// 球速度档(定点×8, 每 tick 位移)
const SPEED_LEVELS: [i32; 3] = [16, 24, 32];

/// 定点缩放(格坐标 ×8)
const FIXED: i32 = 8;
/// 球场边界(7 格)
const FIELD: i32 = 7 * FIXED;

/// 握手超时(ms)
const HANDSHAKE_TIMEOUT: u64 = 5000;
/// 断线超时(ms)
const DISCONNECT_TIMEOUT: u64 = 500;

/// 角色: 先进入 PlayBall 菜单者为 Host, 后进入者为 Client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Host,
    Client,
}

/// 模拟球的结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimEvent {
    /// 无事件
    None,
    /// 击球
    Hit,
    /// 得分
    Score,
}

/// 对打球
pub struct PlayBall {
    role: Role,
    peer: [u8; 6],
    my_paddle: u8,
    peer_paddle: u8,
    ball: (i32, i32),
    vel: (i32, i32),
    speed_level: usize,
    score: (u8, u8),
    server_is_host: bool,
    last_rx: Instant,
    game_over: bool,
    /// 得分时全屏闪色一帧
    score_flash: bool,
    /// 得分闪绿(我方得分) 还是闪红(对方得分)
    flash_green: bool,
}

impl PlayBall {
    pub fn new() -> Self {
        Self {
            role: Role::Host,
            peer: [0; 6],
            my_paddle: 3,
            peer_paddle: 3,
            ball: (FIELD / 2, 28),
            vel: (0, 0),
            speed_level: 0,
            score: (0, 0),
            server_is_host: true,
            last_rx: Instant::now(),
            game_over: false,
            score_flash: false,
            flash_green: true,
        }
    }

    pub async fn run(&mut self, app: &mut App<'_>) {
        app.ledc.clear();
        app.ad = Ad::default();

        // 握手: 确定角色并建立连接
        if self.handshake(app).await.is_err() {
            // 超时无对手: 蜂鸣提示并回菜单
            buzzer::pong_over().await;
            app.ledc.clear();
            return;
        }

        buzzer::pong_connect().await;
        if self.role == Role::Host {
            info!(
                "对打球 Host, peer: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                self.peer[0],
                self.peer[1],
                self.peer[2],
                self.peer[3],
                self.peer[4],
                self.peer[5]
            );
        } else {
            info!(
                "对打球 Client, peer: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                self.peer[0],
                self.peer[1],
                self.peer[2],
                self.peer[3],
                self.peer[4],
                self.peer[5]
            );
        }

        loop {
            Timer::after_millis(50).await;

            app.acc_direction();
            app.check_pause().await;
            self.move_paddle(&app.ad);

            // 收包与断线检测
            if !self.recv_packets(app) {
                self.disconnect(app).await;
                return;
            }

            if self.role == Role::Host {
                // Host 权威模拟球
                match self.simulate() {
                    SimEvent::Hit => buzzer::pong_hit().await,
                    SimEvent::Score => buzzer::pong_score().await,
                    SimEvent::None => {}
                }
                if self.score.0 >= 5 || self.score.1 >= 5 {
                    // 先得 5 分胜: 发送结束帧
                    self.game_over = true;
                    self.send_end(app).await;
                } else {
                    self.send_game_state(app).await;
                }
            } else {
                self.send_my_paddle(app).await;
            }

            if self.game_over {
                self.show_result(app).await;
                return;
            }

            self.draw(app);
        }
    }

    /// 握手: 先监听一段时间, 听到广播则成为 Client; 否则成为 Host 广播寻找
    async fn handshake(&mut self, app: &mut App<'_>) -> Result<(), ()> {
        let deadline = Instant::now() + Duration::from_millis(HANDSHAKE_TIMEOUT);
        let mut anim = 0u8;

        // 监听窗口(1s): 已有 Host 广播则直接成为 Client
        let listen_until = Instant::now() + Duration::from_millis(1000);
        while Instant::now() < listen_until {
            let rx = select(
                app.esp_now.as_mut().unwrap().receive_async(),
                Timer::after_millis(100),
            )
            .await;
            if let Either::First(rx) = rx {
                if rx.data() == [GAME_CODE, ST_SEEKING] && rx.info.src_address != app.my_mac {
                    self.role = Role::Client;
                    self.peer = rx.info.src_address;
                    self.add_peer(app);
                    self.send_join(app).await;
                    self.last_rx = Instant::now();
                    return Ok(());
                }
            }
            self.draw_seeking(app, anim);
            anim = anim.wrapping_add(1);
        }

        // 无人广播: 成为 Host, 每 200ms 广播寻找, 直到收到加入确认或超时
        self.role = Role::Host;
        while Instant::now() < deadline {
            let _ = app
                .esp_now
                .as_mut()
                .unwrap()
                .send_async(&BROADCAST_ADDRESS, &[GAME_CODE, ST_SEEKING])
                .await;

            while let Some(rx) = app.esp_now.as_mut().unwrap().receive() {
                let src = rx.info.src_address;
                if src == app.my_mac {
                    continue;
                }
                let d = rx.data();
                if d == [GAME_CODE, ST_JOIN] {
                    // 收到加入确认: 建立连接, 发 START
                    self.peer = src;
                    self.add_peer(app);
                    self.send_start(app).await;
                    self.last_rx = Instant::now();
                    return Ok(());
                } else if d == [GAME_CODE, ST_SEEKING] {
                    // 双方同时进入: MAC 小者继续做 Host, 大者让位成为 Client
                    if app.my_mac > src {
                        self.role = Role::Client;
                        self.peer = src;
                        self.add_peer(app);
                        self.send_join(app).await;
                        self.last_rx = Instant::now();
                        return Ok(());
                    }
                    // 我保持 Host: 对方让位后会回加入确认
                }
            }

            self.draw_seeking(app, anim);
            anim = anim.wrapping_add(1);
            Timer::after_millis(200).await;
        }

        Err(())
    }

    /// 把对方加入 peer 列表(发送前必须先 add_peer)
    fn add_peer(&mut self, app: &mut App<'_>) {
        let _ = app.esp_now.as_mut().unwrap().add_peer(PeerInfo {
            interface: EspNowWifiInterface::Station,
            peer_address: self.peer,
            lmk: None,
            channel: None,
            encrypt: false,
        });
    }

    /// Client → Host: 加入确认
    async fn send_join(&mut self, app: &mut App<'_>) {
        let _ = app
            .esp_now
            .as_mut()
            .unwrap()
            .send_async(&self.peer, &[GAME_CODE, ST_JOIN])
            .await;
    }

    /// Host → Client: START + 发球方
    async fn send_start(&mut self, app: &mut App<'_>) {
        // 随机决定首局发球方
        self.server_is_host = app.rng.random() % 2 == 0;
        let serve = if self.server_is_host { 0u8 } else { 1u8 };
        self.serve();
        let _ = app
            .esp_now
            .as_mut()
            .unwrap()
            .send_async(&self.peer, &[GAME_CODE, ST_GAME, serve])
            .await;
    }

    /// Host → Client: 游戏状态包(7 字节)
    /// [0]=0x01 [1]=0x03 [2]=host拍子y [3]=球x [4]=球y [5]=球速档 [6]=Host分|Client分
    async fn send_game_state(&mut self, app: &mut App<'_>) {
        let packed = [
            GAME_CODE,
            ST_GAME,
            self.my_paddle,
            (self.ball.0 / FIXED) as u8,
            (self.ball.1 / FIXED) as u8,
            self.speed_level as u8,
            (self.score.0 << 4) | self.score.1,
        ];
        let _ = app
            .esp_now
            .as_mut()
            .unwrap()
            .send_async(&self.peer, &packed)
            .await;
    }

    /// Client → Host: 自己的拍子 y
    async fn send_my_paddle(&mut self, app: &mut App<'_>) {
        let _ = app
            .esp_now
            .as_mut()
            .unwrap()
            .send_async(&self.peer, &[GAME_CODE, ST_GAME, self.my_paddle, 0, 0])
            .await;
    }

    /// Host → Client: 结束帧 + 比分
    async fn send_end(&mut self, app: &mut App<'_>) {
        let _ = app
            .esp_now
            .as_mut()
            .unwrap()
            .send_async(
                &self.peer,
                &[GAME_CODE, ST_END, (self.score.0 << 4) | self.score.1],
            )
            .await;
    }

    /// 收包并更新状态。返回 false 表示掉线
    fn recv_packets(&mut self, app: &mut App<'_>) -> bool {
        loop {
            let Some(rx) = app.esp_now.as_mut().unwrap().receive() else {
                break;
            };
            if rx.info.src_address != self.peer {
                continue;
            }
            let d = rx.data();
            match d {
                [GAME_CODE, ST_GAME, _, ..] if d.len() >= 3 => {
                    if self.role == Role::Host {
                        // Client → Host: [01][03][拍子y][0][0]
                        self.peer_paddle = d[2];
                    } else if d.len() >= 7 {
                        // Host → Client: 球状态 + 比分
                        self.peer_paddle = d[2];
                        let prev = self.score;
                        self.score = (d[6] & 0x0F, d[6] >> 4);
                        if self.score.0 + self.score.1 > prev.0 + prev.1 {
                            // 刚有得分: 我方得分闪绿, 对方得分闪红
                            self.score_flash = true;
                            self.flash_green = self.score.0 > prev.0;
                        }
                        if self.score.0 >= 5 || self.score.1 >= 5 {
                            self.game_over = true;
                        }
                        self.ball = ((d[3] as i32) * FIXED, (d[4] as i32) * FIXED);
                        self.speed_level = d[5] as usize;
                    }
                    self.last_rx = Instant::now();
                }
                [GAME_CODE, ST_END, _] if d.len() >= 3 => {
                    // Host → Client: 结束帧
                    self.score = (d[2] & 0x0F, d[2] >> 4);
                    self.game_over = true;
                    self.last_rx = Instant::now();
                }
                _ => {}
            }
        }
        Instant::now() - self.last_rx < Duration::from_millis(DISCONNECT_TIMEOUT)
    }

    /// 移动自己的拍子(前倾=上, 后倾=下)
    fn move_paddle(&mut self, ad: &Ad) {
        match ad {
            Ad::Front => self.my_paddle = self.my_paddle.saturating_sub(1),
            Ad::Back => self.my_paddle = self.my_paddle.saturating_add(1).min(6),
            _ => {}
        }
    }

    /// Host 权威模拟球(定点 ×8)
    fn simulate(&mut self) -> SimEvent {
        self.ball.0 += self.vel.0;
        self.ball.1 += self.vel.1;

        // 上下墙反弹
        if self.ball.1 < 0 {
            self.ball.1 = 0;
            self.vel.1 = -self.vel.1;
        } else if self.ball.1 > FIELD {
            self.ball.1 = FIELD;
            self.vel.1 = -self.vel.1;
        }

        let gx = self.ball.0 / FIXED;
        let gy = self.ball.1 / FIXED;
        let my = self.my_paddle as i32;
        let peer = self.peer_paddle as i32;

        // 拍子反弹(列 0 我方 / 列 7 对方, 各占两格)
        if self.vel.0 < 0 && gx == 0 && (gy == my || gy == my + 1) {
            self.ball.0 = FIXED;
            return self.bounce();
        }
        if self.vel.0 > 0 && gx == 7 && (gy == peer || gy == peer + 1) {
            self.ball.0 = FIELD - FIXED;
            return self.bounce();
        }

        // 出界判分
        if self.ball.0 < 0 {
            // 我方漏接: 对方得分
            self.score.1 += 1;
            self.score_flash = true;
            self.flash_green = false;
            self.serve();
            SimEvent::Score
        } else if self.ball.0 > FIELD {
            // 对方漏接: 我方得分
            self.score.0 += 1;
            self.score_flash = true;
            self.flash_green = true;
            self.serve();
            SimEvent::Score
        } else {
            SimEvent::None
        }
    }

    /// 击球: 反弹并提速
    fn bounce(&mut self) -> SimEvent {
        if self.speed_level < SPEED_LEVELS.len() - 1 {
            self.speed_level += 1;
        }
        let s = SPEED_LEVELS[self.speed_level];
        self.vel.0 = if self.vel.0 < 0 { s } else { -s };
        self.vel.1 = if self.vel.1 < 0 { -s / 2 } else { s / 2 };
        SimEvent::Hit
    }

    /// 发球: 球回到中心, 向接发球方发出, 双方交替发球
    fn serve(&mut self) {
        self.ball = (FIELD / 2, 28);
        self.speed_level = 0;
        let s = SPEED_LEVELS[0];
        let dir = if self.server_is_host { 1 } else { -1 };
        self.vel = (dir * s, s / 2);
        self.server_is_host = !self.server_is_host;
    }

    /// 渲染(双方相同布局): 列 0 我方拍子, 列 7 对方拍子, 球红色
    fn draw(&mut self, app: &mut App<'_>) {
        let ledc = &mut app.ledc;

        if self.score_flash {
            // 得分方全屏闪色一帧
            ledc.clear_with_color(if self.flash_green {
                Rgb888::CSS_GREEN
            } else {
                Rgb888::CSS_RED
            });
            self.score_flash = false;
            return;
        }

        ledc.clear();
        let bx = (self.ball.0 / FIXED) as i32;
        let by = (self.ball.1 / FIXED) as i32;
        let my = self.my_paddle as i32;
        let peer = self.peer_paddle as i32;
        ledc.write_pixels([
            // 我方拍子(列 0)
            Pixel(Point::new(0, my), Rgb888::CSS_WHITE),
            Pixel(Point::new(0, my + 1), Rgb888::CSS_WHITE),
            // 对方拍子(列 7)
            Pixel(Point::new(7, peer), Rgb888::CSS_WHITE),
            Pixel(Point::new(7, peer + 1), Rgb888::CSS_WHITE),
            // 球(红色)
            Pixel(Point::new(bx, by), Rgb888::CSS_RED),
        ]);
    }

    /// 寻找中动画: 双拍固定, 球来回滚动
    fn draw_seeking(&mut self, app: &mut App<'_>, step: u8) {
        let ledc = &mut app.ledc;
        ledc.clear();
        ledc.write_pixels([
            Pixel(Point::new(0, 2), Rgb888::CSS_WHITE),
            Pixel(Point::new(0, 3), Rgb888::CSS_WHITE),
            Pixel(Point::new(7, 4), Rgb888::CSS_WHITE),
            Pixel(Point::new(7, 5), Rgb888::CSS_WHITE),
            Pixel(Point::new((step % 6 + 1) as i32, 3), Rgb888::CSS_RED),
        ]);
    }

    /// 比赛结束: 显示比分与胜负动画
    async fn show_result(&mut self, app: &mut App<'_>) {
        buzzer::pong_over().await;
        let color = if self.score.0 >= self.score.1 {
            Rgb888::CSS_GREEN
        } else {
            Rgb888::CSS_RED
        };
        for _ in 0..3 {
            app.ledc.clear_with_color(color);
            Timer::after_millis(200).await;
            app.ledc.clear();
            Timer::after_millis(200).await;
        }
        // 显示比分(十位=我方, 个位=对方)
        app.ledc.draw_score(self.score.0 * 10 + self.score.1);
        Timer::after_millis(2000).await;
    }

    /// 掉线处理
    async fn disconnect(&mut self, app: &mut App<'_>) {
        buzzer::pong_over().await;
        for _ in 0..3 {
            app.ledc.clear_with_color(Rgb888::CSS_RED);
            Timer::after_millis(150).await;
            app.ledc.clear();
            Timer::after_millis(150).await;
        }
        Timer::after_millis(1000).await;
    }
}
