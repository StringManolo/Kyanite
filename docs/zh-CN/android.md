# Android 上的 Kyanite

本页是面向初学者的 Kyanite Android 应用简介。[KyaniteAndroid 仓库](https://github.com/StringManolo/KyaniteAndroid)是权威来源，包含完整且最新的详细信息（英文）。

## 它是什么？

一个在内置终端中运行 Kyanite 的 Android 应用。它使用 PRoot，因此不需要特殊权限。Kyanite 系统已包含在应用内部。

## 安装

1. [下载最新的 APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)。
2. 打开文件并安装。Android 可能会要求你允许从此来源安装。
3. 更新始终需要你的确认，并且必须使用与已安装版本相同的密钥签名。

## 首次启动

在你做出选择之前，不会安装或启动任何东西。首次启动会打开设置界面（⚙ 按钮始终可用）。

## 运行 Kyanite 的方式

| 模式 | 效果 |
| --- | --- |
| **Normal（普通）** | 你的更改会在会话之间保留。 |
| **Ephemeral（临时）** | 退出时全部清除。它会在磁盘上使用临时存储，因此闪存中可能残留痕迹。 |
| **VM（虚拟机）** | 实验性功能。在模拟的机器（QEMU）中运行：慢得多，但不会向磁盘写入任何内容。 |

你还可以在 **System**（你已安装的 Kyanite）和 **LiveCD**（随应用附带的系统，类似 live CD）之间选择。

## 窗口与键盘

- **全屏**或**悬浮窗口**（需要“显示在其他应用上层”权限）。悬浮窗口可以移动、调整大小、旋转、固定、设为透明和最小化。
- **额外按键栏**提供 ESC、TAB、CTRL、ALT、方向键等。轻点一次修饰键表示只用一次；长按则锁定。
- 可在设置中选择配色方案和字体大小。双指捏合可缩放文字。

## 隐私与安全选项

- PIN 锁。
- 对已存储的系统加密。
- 可禁止截屏、隐藏通知、清除剪贴板的选项。
- 可切断应用网络访问的开关。
- 定时擦除数据。

## 须知

- PRoot **不是**安全边界。如需更强的隔离，请使用 VM 模式。
- Android 12 及更高版本可能会终止后台进程。如果你的会话在后台结束，请查找“phantom process killer”相关设置。
- 进入系统后，使用 `kpkg` 安装软件。参见 [kpkg 指南](kpkg.md)。
