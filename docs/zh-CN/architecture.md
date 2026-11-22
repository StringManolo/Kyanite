# 架构

Kyanite 是一个最小化的 Linux 根文件系统，围绕少量基于 Rust 的核心组件和与 Debian 兼容的软件包环境构建。

## 系统概览

预装的 Rust 组件与 musl 静态链接：

- Kyanite init
- `kpkg`
- Brush
- uutils coreutils

根文件系统还包含一个 glibc 运行时，供动态链接的 Debian 软件包使用。

## 启动流程

基本的启动流程如下：

1. 启动器（launcher）进入根文件系统，通常使用 PRoot。
2. 启动器调用 `/bin/sh`。
3. `/bin/sh` 指向 `/sbin/init`。
4. Kyanite init 配置终端，并以登录 shell 的方式启动 Brush。
5. Init 回收孤儿进程，并在 shell 退出时退出。

Init 在 Kyanite 环境中以 PID 1 运行。

## 主要组件

| 组件 | 职责 |
| --- | --- |
| `kyanite-init` | PID 1、启动 shell、终端与作业控制、回收孤儿进程 |
| Brush | 交互式 shell |
| uutils coreutils | 常用命令行工具 |
| `kpkg` | 软件包索引、依赖解析、软件包安装和基础系统更新 |
| glibc 运行时 | 为 Debian 软件包提供运行时支持 |
| `vim-tiny` | 最小的、与 vi 兼容的编辑器 |

## musl 与 glibc

Kyanite 对其静态链接的 Rust 工具使用 musl，这使核心工具不依赖 glibc 运行时。

rootfs 还包含 glibc，以便 Debian 软件源中的软件包能够运行。这样既保持了精简的基础系统，又能使用 Debian 的软件包生态。

## 软件包安装

`kpkg` 会下载 Debian 软件包，验证软件源元数据和软件包哈希，解析依赖，并将软件包解压到目标根目录。

软件包的配置通过 Kyanite 的声明式钩子完成。参见[包钩子](package-hooks.md)。

## 基础系统更新

Kyanite 基础系统的更新与普通的 Debian 软件包升级是分开的。符合条件的 GitHub 发布版包含更新基础系统所需的文件和清单，并会保留本地修改过的文件。

参见[发布](releases.md)和 [kpkg 基础系统更新](kpkg.md#updating-the-kyanite-base)。

## 运行模型

Kyanite 目前以根文件系统的形式分发，并通过其运行环境使用宿主内核。关于平台和隔离方面的考虑，请参见[已知限制](limitations.md)和 [SECURITY.md](../../SECURITY.md)（英文）。
