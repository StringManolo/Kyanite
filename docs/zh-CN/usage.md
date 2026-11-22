# 使用

本指南介绍 Kyanite 自带的交互式 shell 环境，以及主要的自定义方式。

## 启动会话

Kyanite 的 `/bin/sh` 指向 `/sbin/init`。Init 会配置终端，并以交互式登录 shell 的方式启动 Brush。

shell 退出时，init 也随之退出，会话结束。

## Shell 配置文件

| 路径 | 用途 |
| --- | --- |
| `/etc/profile` | 登录配置文件；输出 MOTD 并加载 `~/.bashrc` |
| `/root/.bashrc` | 提示符、历史记录设置、别名和辅助函数 |
| `/etc/update-motd.d/` | 生成每日消息的脚本 |
| `/etc/motd` | 可选的静态文本，在 MOTD 脚本之后输出 |
| `/etc/kyanite/logo.txt` | 由 logo MOTD 脚本显示的 ASCII 标志 |
| `/etc/os-release` | 发行版名称和构建版本 |

默认文件维护在仓库的 [`rootfs-overlay/`](../../rootfs-overlay/) 目录中。

## 自定义每日消息

MOTD 脚本按文件名顺序从 `/etc/update-motd.d/` 运行。

要在保留文件的同时禁用某个脚本：

```sh
chmod -x /etc/update-motd.d/00-logo
```

要添加一条静态消息：

```sh
printf '%s\n' 'Welcome to Kyanite' > /etc/motd
```

要让你登录时不显示 MOTD：

```sh
touch ~/.hushlogin
```

当设置了 `NO_COLOR` 或 `TERM` 为 `dumb` 时，颜色会被禁用。

MOTD 脚本通过 `/bin/brush` 执行，因此不需要 `#!/bin/sh` shebang 行。

## 自定义标志

用你自己的、已经带颜色的文本替换 `/etc/kyanite/logo.txt`。

例如，如果已安装 `chafa`：

```sh
chafa --size=36x --colors=256 logo.png > /etc/kyanite/logo.txt
```

## 别名与辅助命令

默认的 `.bashrc` 包含提示符、历史记录配置、别名以及 `extract` 和 `gacp` 等辅助函数。

这些辅助函数用到的一些命令需要单独安装。例如：

```sh
kpkg add tar git curl unzip
```

有关软件包的安装和管理，请参见 [kpkg 指南](kpkg.md)。

## 环境

启动器通常会设置 `HOME`、`USER`、`LOGNAME`、`TERM`、`LANG` 和 `PATH` 等环境变量。具体取值可能因终端和宿主集成方式而异。
