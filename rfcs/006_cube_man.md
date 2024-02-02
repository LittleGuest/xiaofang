- Feature Name: 是方块人就下一百层
- Start Date: 2024-02-02

# Summary

[summary]: #summary

用 Rust 设计一款"是方块人就下一百层"的游戏,运行在 esp32c3 上,显示在 8\*8 的 ws2812 点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3 嵌入式学习,ws2812 的使用.

# Detailed design

[detailed-design]: #detailed-design

## 方块人设计

红色像素代表方块人

## 楼层设计

- 楼层的长度最小为 2,最长长度 5
- 楼层的类型
  - 正常的楼层
  - 陷阱楼层
    - 易碎楼层: 用闪烁来表示易碎,当人物站在该楼层上时,该楼层急速闪烁 2s 后碎裂,人物继续往下掉;
    - 传送带楼层:
      - 顺时针旋转: 两端不闪烁,中间从左到右闪烁,即该楼层的长度最少是 4
      - 逆时针旋转: 两端不闪烁,中间从右到左闪烁,即该楼层的长度最少是 4
    - 弹簧楼层: 用黄色表示,当人物站在该楼层上时,将会被弹起 3 格高度

楼层的不同类型根据设置的频率随机生成:

- 正常的楼层 : 70%
- 陷阱楼层 : 30%
  - 易碎楼层: 10%
  - 传送带楼层: 10%
  - 弹簧楼层: 10%

## 其它设计

人物最开始是站在正常的楼层上.
人物的下落速度可以设置.

### 弹簧反弹效果

使用 pwm 控制反弹效果的渐变

# Unresolved questions

[unresolved-questions]: #unresolved-questions

尚未解决的问题

# Future possibilities

[future-possibilities]: #future-possibilities

未来的可能性
