- 功能名称: 对打球
- 开始时间: 2024-10-14

# 摘要

用Rust设计一款"对打球"的功能，运行在 esp32c3 上，显示在`8*8`的 ws2812 点阵上。

# 目的

Rust、esp32c3、ws2812 的学习使用；

# 解释

## ESP-NOW简介

ESP-NOW 是一种由乐鑫公司定义的无连接 Wi-Fi 通信协议。在 ESP-NOW中，应用程序数据被封装在各个供应商的动作帧中，然后在无连接的情况下，从一个 Wi-Fi 设备传输到另一个 Wi-Fi 设备。

CTR 与 CBC-MAC 协议 (CCMP) 可用来保护动作帧的安全。ESP-NOW 广泛应用于智能照明、远程控制、传感器等领域。

# 详细设计

## 交互过程

1. 由其中一方如A发起广播数据，B接收广播数据后，B向A发送单播确认信息，A收到确认信息后开始通讯。

其中A发送的广播数据格式如下：

```
01  00             101010101010101
^   ^              ^
|   |              |
|   |              |
|   |              |
|   |              |
表  00表示结束     游戏中产生的数据
示  01表示游戏中
哪
款
游
戏
```

2. 谁先响应A，A就和谁通信，停止广播，发送数据
3. 开始数据交互
4. 游戏结束，计算结果，显示结果，停止发送数据

其中：

| 代码 | 说明   |
| ---- | ------ |
| 00   | 无     |
| 01   | 对打球 |

# 未解决的问题

xxxxxxxxxxxxxxxxxxxx

# 缺点

为什么不能这样做？

# 替代品

未调查

# 未来展望

无

# 参考链接

- https://docs.espressif.com/projects/esp-idf/zh_CN/latest/esp32c3/api-reference/network/esp_now.html