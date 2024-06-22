## 原创作品

[复古像素风 桌面电子沙漏，Arduino（atmega328p+mpu6050）](https://www.bilibili.com/video/BV1QP411Y7K3/?share_source=copy_web&vd_source=41da856c543dfcc8802471a83af59251)

## 复刻

"小方"复刻，使用 Rust 编写，运行在 esp32c3 上，采用 ws2812 显示，集成沙漏、骰子、卦象、贪吃蛇、迷宫等游戏

## 单机游戏

- [x] 沙漏
- [x] 骰子
- [x] 卦象
- [x] 贪吃蛇
- [x] 迷宫
- [ ] 是方块人就下一百层
- [x] 推箱子
- [ ] 躲避方块
- [ ] ...

## 联机游戏

- [ ] 对打球
- [ ] ...

## 接线

### MPU6050

| MPU6050 | MCU   |         |
| ------- | ----- | ------- |
| VCC     | 3.3V  |         |
| GND     | GND   |         |
| SCL     | GPIO5 | I2C_SCL |
| SDA     | GPIO4 | I2C_SDA |

### 无缘蜂鸣器

| 陶瓷片无缘蜂鸣器 | MCU    |
| ---------------- | ------ |
| V+               | GPIO11 |
| V-               | GND    |

### WS2812

| WS2812 | MCU   |          |
| ------ | ----- | -------- |
| IN     | GPIO3 | SPI_MOSI |
| V+     | 3.3V  |          |
| V-     | GND   |          |

### 麦克风

| -   | MCU   |      |
| --- | ----- | ---- |
| A0  | GPIO0 | ADC0 |
| V+  | 5V    |      |
| GND | GND   |      |
| D0  | -     |      |

## 三方库

- https://github.com/smart-leds-rs/ws2812-spi-rs,修改了部分源码

## 参考链接

- https://blog.theembeddedrustacean.com/esp32-embedded-rust-at-the-hal-pwm-buzzer
