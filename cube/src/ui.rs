/// 界面
#[derive(Debug, Default)]
pub enum Ui {
    /// 沙漏
    #[default]
    Timer,
    /// 骰子
    Dice,
    /// 贪吃蛇
    Snake,
    /// 卦象
    BaGua,
    /// 迷宫
    Maze,
    /// 方块人
    CubeMan,
    /// 推箱子
    Sokoban,
    /// 躲避方块
    DodgeCube,
    /// 声音
    Sound,
}

impl Ui {
    pub fn uis() -> [Ui; 9] {
        [
            Ui::Timer,
            Ui::Dice,
            Ui::Snake,
            Ui::BaGua,
            Ui::Maze,
            Ui::CubeMan,
            Ui::Sokoban,
            Ui::DodgeCube,
            Ui::Sound,
        ]
    }

    #[rustfmt::skip]
    pub fn ui(&self) -> [u8; 8] {
        match self {
            Ui::Maze => [
                0b00000000,
                0b01010110,
                0b01011010,
                0b01000010,
                0b00111010,
                0b00100010,
                0b01101110,
                0b00000000,
            ],
            Ui::Snake => [
                0b00000000,
                0b01110100,
                0b01000000,
                0b01111110,
                0b00000010,
                0b00011110,
                0b00010000,
                0b00000000,
            ],
            Ui::BaGua => [
                0b00000000,
                0b01100110,
                0b00000000,
                0b01100110,
                0b00000000,
                0b01111110,
                0b00000000,
                0b00000000,
            ],
            Ui::Dice => [
                0b00000000,
                0b01100110,
                0b01100110,
                0b00011000,
                0b00011000,
                0b01100110,
                0b01100110,
                0b00000000,
            ],
            Ui::Timer => [
                0b00000000,
                0b01111110,
                0b00111100,
                0b00011000,
                0b00011000,
                0b00111100,
                0b01111110,
                0b00000000,
            ],
            Ui::CubeMan => [
                0b00000000,
                0b00011100,
                0b00000000,
                0b00001111,
                0b00000000,
                0b11110000,
                0b00000000,
                0b00111110,
            ],
            Ui::Sokoban => [
                0b01100110,
                0b10111101,
                0b10000001,
                0b10111101,
                0b10000001,
                0b11011011,
                0b01000010,
                0b01111110,
            ],
            Ui::DodgeCube => [
                0b11111111,
                0b10101101,
                0b11101101,
                0b10001101,
                0b10101101,
                0b10111110,
                0b00000000,
                0b00010000,
            ],
            // Ui::Temp => [
            //     0b00000000,
            //     0b01100000,
            //     0b01101110,
            //     0b00010000,
            //     0b00010000,
            //     0b00010000,
            //     0b00001110,
            //     0b00000000,
            // ],
            Ui::Sound => [
                0b00000000,
                0b00011000,
                0b00001100,
                0b00001010,
                0b00011000,
                0b00111000,
                0b00110000,
                0b00000000,
            ],
        }
    }
}
