- Feature Name: 贪吃蛇
- Start Date: 2023-12-20

# Summary

[summary]: #summary

用Rust设计一款"贪吃蛇"的游戏,运行在esp32c3上,显示在8*8的ws2812点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3嵌入式学习,ws2812的使用.

# Detailed design

[detailed-design]: #detailed-design

- 初始蛇身有头有尾,移动方向默认向上

## 蛇设计

```Rust
struct Snake {
    direction: Direction,
    head: Position,
    body: LinkedList<Position>,
}
```

蛇本身有移动方向direction,头部head的坐标和身体body的坐标(其中包含头部,在LinkedList的首部),


# Unresolved questions

[unresolved-questions]: #unresolved-questions

- 吃掉食物的动画和音效
- 历史最高分动画,音乐
- 移动音效,得分音效和画面效果,死亡音效


# Future possibilities

[future-possibilities]: #future-possibilities

无
