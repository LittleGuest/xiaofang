- Feature Name: 沙漏
- Start Date: 2024-01-01

# Summary

[summary]: #summary

用Rust设计一款"沙漏"的游戏,运行在esp32c3上,显示在8*8的ws2812点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3嵌入式学习,ws2812的使用.

# Detailed design

上半部分4行表示沙子,沙子由上掉下

## 下落过程

随机下落一个在最下层的像素,不能越过最下层像素(否则看起来就像中间空了一样)

当像素落下时,随机选择一列最后一个像素落下,实现堆积的效果

[detailed-design]: #detailed-design

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- 闪烁音效


# Future possibilities

[future-possibilities]: #future-possibilities

无
