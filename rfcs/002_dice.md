- Feature Name: 骰子
- Start Date: 2023-12-16

# Summary

[summary]: #summary

用 Rust 设计一款"骰子"的游戏,运行在 esp32c3 上,显示在 8\*8 的 ws2812 点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3 嵌入式学习,ws2812 的使用.

# Detailed design

[detailed-design]: #detailed-design

## 界面设计

```Text
00000000
01100110
01100110
00011000
00011000
01100110
01100110
00000000
```

## 数字设计

晃动屏幕,将随机生成 6 个数字中的一个;

### 1

```Text
00000000
00011000
00111100
01111110
01111110
00111100
00011000
00000000
```

### 2

```Text
00000110
00001111
00001111
00000110
01100000
11110000
11110000
01100000
```

### 3

```Text
00011000
00111100
00111100
00011000
11000011
11100111
11100111
11100111
```

### 4

```Text
11100111
11100111
11100111
00000000
00000000
11100111
11100111
11100111
```

### 5

```Text
11100111
11100111
11011011
00111100
00111100
11011011
11100111
11100111
```

### 6

```Text
11100111
11100111
00000000
11100111
11100111
00000000
11100111
11100111
```

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- 摇动的动画

# Future possibilities

[future-possibilities]: #future-possibilities

无
