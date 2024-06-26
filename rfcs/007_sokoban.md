- Feature Name: 推箱子
- Start Date: 2024-02-20

# Summary

[summary]: #summary

用 Rust 设计一款"推箱子"的游戏,运行在 esp32c3 上,显示在 8\*8 的 ws2812 点阵上.

# Motivation

[motivation]: #motivation

Rust,esp32c3 嵌入式学习,ws2812 的使用.

# Detailed design

[detailed-design]: #detailed-design

## XSB 格式和 LURD 格式简介

推箱子关卡一般用 XSB 格式来保存和交流,解答步骤则使用LURD格式.

### XSB格式简介

| 字符 | 含义                                                                                                |
| ---- | --------------------------------------------------------------------------------------------------- |
| @    | 人(man)                                                                                             |
| +    | 人在目标点(man on goal)                                                                             |
| $    | 箱子(box)                                                                                           |
| \*   | 箱子在目标点上(box on goal)                                                                         |
| #    | 墙(wall)                                                                                            |
| .    | 目标点(goal)                                                                                        |
| -    | XSB格式空格代表"地板",又因为连续多个空格在网页 或即时通讯软件中偶尔显示有问题,也用"-"或"\_"代替空格 |

#### 例子

```Text
----#####----------
----#---#----------
----#$--#----------
--###--$##---------
--#--$-$-#---------
###-#-##-#---######
#---#-##-#####--..#
#-$--$----------..#
#####-###-#@##--..#
----#-----#########
----#######--------
```

### LURD

答案用LURD格式,小写字母是移动,大写字母是推动.

| 字符 | 含义 |
| ---- | ---- |
| l或L | 左   |
| r或R | 右   |
| u或U | 上   |
| d或D | 下   |

#### 例子

```Text
ullluuuLUllDlldddrRRRRRRRRRRdrUllllllluuululldDDuu
lldddrRRRRRRRRRRRRlllllllluuulLulDDDuulldddrRRRRRR
RRRRRllllllluuulluuurDDuullDDDDDuulldddrRRRRRRRRRR
uRRlDllllllluuuLLulDDDuulldddrRRRRRRRRRRdRRlUlllll
lllllllulldRRRRRRRRRRRRRuRDldR
```

## 界面设计

```Text
01100110
10111101
10000001
10111101
10000001
11011011
01000010
01111110
```

## 游戏设计

- 红色表示玩家
- 黄色表示玩家在目标点上
- 蓝色表示箱子
- 青色表示箱子在目标点上
- 白色表示墙
- 绿色表示目标点
- 地板没有颜色

## 实现

### 地图

```Rust
struct Map {
    /// 宽度
    width: usize,
    /// 长度
    height: usize,
    /// 地图数据
    data: Vec<Vec<Option<Pixel<Rgb888>>>>,
}
```

```Rust
enum Type{
    // 人
    Man,
    // 人在目标点上
    ManOnGoal,
    // 箱子
    Box,
    // 箱子在目标点上
    BoxOnGoal,
    // 墙
    Wall,
    // 目标点
    Goal,
    // 地板
    Floor,
}
```

标记地图中的类型,表示墙,人还是目标点.从XSB生成地图

# Unresolved questions

[unresolved-questions]: #unresolved-questions

无

# Future possibilities

[future-possibilities]: #future-possibilities

- http://sokoban.cn 导入该网站生成的地图数据
- 推箱子关卡设计器和求解器
- [【算法】从推箱子的解答步骤还原关卡地图](https://www.cnblogs.com/skyivben/archive/2011/07/03/2096801.html)
