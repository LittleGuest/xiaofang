- Feature Name: 躲避方块
- Start Date: 2024-05-19

# Summary

用 Rust 设计一款"躲避方块"的游戏,运行在 esp32c3 上,显示在 8\*8 的 ws2812 点阵上.

# Motivation

Rust,esp32c3 嵌入式学习,ws2812 的使用.

# Detailed design

## 界面设计

```Text
11111111
10101101
11101101
10001101
10101101
10111110
00000000
00010000
```

## 游戏设计

### 人物

红色表示，可以上下左右移动进行躲避，以增加趣味性。

### 障碍物

- 定义好障碍物的形状，随机生成落下
- 障碍物之间的距离最少间隔一个像素

# Unresolved questions

- 游戏音乐

# Future possibilities

无
