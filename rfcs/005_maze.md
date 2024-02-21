- Feature Name: 迷宫游戏
- Start Date: 2024-01-06

# Summary

[summary]: #summary

用 Rust 设计一款"迷宫"的游戏,运行在 esp32c3 上,显示在 8*8 的 ws2812 点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3 嵌入式学习,ws2812 的使用.

# Detailed design

[detailed-design]: #detailed-design

## 界面设计

```Text
00000000
01010110
01011010
01000010
00111010
00100010
01101110
00000000
```

## 迷宫设计

左上角为坐标原点,向右为 x 轴方向,向下为 y 轴方向,所有的坐标都为全局坐标.

## 玩家设计

```Rust
struct Player {
    pos: Position,
    old_pos: Position,
    color: Rgb888,
}
```

玩家的像素颜色用红色表示,

如果地图大小大于 8*8,led 是显示不完整的,就要添加一个视野的效果,地图的内容根据视野来加载

## 视野设计

```Rust
struct Vision {
    /// 视野左上角坐标
    pos: Position,
    /// 视野数据
    data: [[Option<Position>; 8]; 8],
}
```

### 视野移动

那么视野怎么移动呢,是根据玩家的位置来移动视野的,但是玩家的位置是不能超过视野范围的
因为当玩家向前移动时,需要把地图的内容加载到视野中,就是说当玩家移动到视野范围-1 或-2 时,
视野范围就需要跟着玩家一起向前运动
当视野范围的边界的地图的边界重叠时,视野范围不动,玩家再向前移动,可以靠近边界进行移动.

### 视野数据

初始视野位置是由玩家的位置决定,位于玩家左上角,x,y 相距 3 个像素的位置;

玩家移动之后视野数据就会发生改变

## 迷宫地图

```Rust
struct MazeMap {
    /// 宽度
    width: usize,
    /// 长度
    height: usize,
    /// 地图数据
    data: Vec<Vec<Option<Position>>>,
    /// 地图颜色
    color: Rgb888,
    /// 起点
    spos: Position,
    /// 终点
    epos: Position,
    /// 终点颜色
    color_epos: Rgb888,
}
```

这里使用第三方库,使用算法生成迷宫地图

游戏的起点位置和玩家的位置是一致的,结束位置现随机生成,后续使用算法根据初始位置计算结束位置.

结束位置用绿色表示.

## 坐标转换

需要将地图坐标转换为 led 坐标

# Drawbacks

[drawbacks]: #drawbacks

- 迷宫结束位置随机生成,可能起始位置间隔很近;

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- 开始动画和音乐
- 结束动画和音乐

# Future possibilities

[future-possibilities]: #future-possibilities

- 亲手实现迷宫生成算法
- 使用不同的算法来生成迷宫
