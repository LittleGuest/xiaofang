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

## 界面设计

```Text
00000000
00011100
00000000
00001111
00000000
11110000
00000000
00111110
```

## 方块人设计

红色像素代表方块人

## 楼梯设计

- 楼梯的长度最小为 3,最长长度 5
- 楼梯的类型
  - 正常的楼梯,白色表示
  - 陷阱楼梯，红色表示
    - 易碎楼梯，灰色表示: 用闪烁来表示易碎,当人物站在该楼梯上时,该楼梯急速闪烁 3 次后碎裂,人物继续往下掉;
    - 传送带楼梯，绿色表示:
      - 顺时针旋转: 两端不闪烁,中间从左到右闪烁,即该楼梯的长度最少是 4
      - 逆时针旋转: 两端不闪烁,中间从右到左闪烁,即该楼梯的长度最少是 4
    - 弹簧楼梯: 黄色表示,当人物站在该楼梯上时,将会被弹起 2 格高度

楼梯的不同类型根据设置的频率随机生成:

- 正常的楼梯 : 70%
- 陷阱楼梯 : 30%
  - 易碎楼梯: 10%
  - 传送带楼梯: 10%
  - 弹簧楼梯: 10%

## 地图数据

整个地图的坐标以左上角为原点,向右为 x,向下为 y

## 游戏逻辑设计

楼梯生成在视野之外，楼梯向上运动；

- 这里需要注意楼梯的生成规则，楼梯和楼梯之间至少间隔一个人物的高度。
- 视野中有多个楼梯的数据，当最上面的楼梯掉出视野之外时，将其销毁。

人向下自由落体运动，当人掉出视野之外，游戏结束；

当人碰到楼梯时，随着楼梯一起向上运动，如果超出视野之外，游戏结束；

## 其它设计

- 人物最开始是站在正常的楼梯上.
- 人物的下落速度可以设置.最好根据游戏进度来设置下落速度,越往后越快

### 弹簧反弹效果

控制反弹效果的渐变

## 实现

### 地板

```Rust
/// 传送带旋转方向
enum ConveyorDir{
  /// 顺时针
  Clockwise,
  /// 逆时针
  Counterclockwise
}

/// 地板类型
enum FloorType{
  /// 正常
  Normal,
  /// 易碎(碎裂时间)
  Fragile(u32),
  /// 传送带(传送带旋转方向)
  Conveyor(ConveyorDir),
  /// 弹簧(反弹的高度)
  Spring(u8),
}

/// 地板
struct Floor{
  /// 类型
  r#type: FloorType,
  /// 颜色
  color:Rgb888,
  data: Vec<Position>,
}

impl Floor{
  fn random() -> Self {}
}
```

### 方块人

```Rust
struct CubeMan{
  /// 位置
  pos:Position,
  /// 移动速度
  move_speed: f32,
  /// 下落速度
  fall_speed: f32,
  /// 颜色
  color:Rgb888,
}

impl CubeMan{
  fn new() -> Self {}
  fn next_pos(&self) -> Position {}
  /// 左右移动
  fn r#move(&mut self) {}
  /// 下落
  fn fall(&mut self) {}
}
```

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- 历史最高分动画,音乐
- 移动音效,得分音效和画面效果,死亡音效

# Future possibilities

[future-possibilities]: #future-possibilities

无
