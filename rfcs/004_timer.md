- Feature Name: 沙漏
- Start Date: 2024-01-01

# Summary

[summary]: #summary

用 Rust 设计一款"沙漏"的游戏,运行在 esp32c3 上,显示在 8\*8 的 ws2812 点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3 嵌入式学习,ws2812 的使用.

# Detailed design

[detailed-design]: #detailed-design

## 界面设计

```Text
00000000
01111110
00111100
00011000
00011000
00111100
01111110
00000000
```

上半部分 4 行表示沙子,沙子由上掉下

## 下落过程

随机下落一个在最下层的像素,不能越过最下层像素(否则看起来就像中间空了一样)

当像素落下时,随机选择一列最后一个像素落下,实现堆积的效果;

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- 闪烁音效
- 下落动画

# Future possibilities

[future-possibilities]: #future-possibilities

无
