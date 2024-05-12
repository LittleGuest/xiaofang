- Feature Name: 贪吃蛇
- Start Date: 2023-12-20

# Summary

[summary]: #summary

用 Rust 设计一款"贪吃蛇"的游戏,运行在 esp32c3 上,显示在 8\*8 的 ws2812 点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3 嵌入式学习,ws2812 的使用.

# Detailed design

[detailed-design]: #detailed-design

## 界面设计

```Text
00000000
01110100
01000000
01111110
00000010
00011110
00010000
00000000
```

- 初始蛇身有头有尾,移动方向默认向上

## 蛇设计

```Rust
struct Snake {
    direction: Direction,
    head: Position,
    body: LinkedList<Position>,
}
```

蛇本身有移动方向 direction,头部 head 的坐标和身体 body 的坐标(其中包含头部,在 LinkedList 的首部),

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- 吃掉食物的动画和音效
- 移动音效,得分音效和画面效果,死亡音效

# Future possibilities

[future-possibilities]: #future-possibilities

无
