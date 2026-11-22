# 安装

Kyanite 以 Linux 根文件系统压缩包的形式分发，也可以通过专门的 Android 应用使用。

## Android

在 Android 设备上，最简单的入门方式是安装 Kyanite 应用。

- [下载最新的 Kyanite APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
- [KyaniteAndroid 源代码和文档](https://github.com/StringManolo/KyaniteAndroid)

Android 应用提供自己的设置和启动流程。另请参见 [Android 指南](android.md)。

## 下载 Linux 根文件系统

打开 [Kyanite Releases 页面](https://github.com/StringManolo/Kyanite/releases)，下载与你的架构对应的压缩包：

| 架构 | 压缩包 |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |

同时下载对应的 `.sha256` 和 `.sha256.sig` 文件。

## 验证下载

在存放下载文件的目录中检查 SHA256 校验和：

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

请使用与你下载的压缩包相匹配的校验和文件名。

如需同时验证 Ed25519 签名，请按照[验证发布版](../../SECURITY.md#verifying-releases)（英文）操作。

## 解压根文件系统

创建一个目录并将压缩包解压到其中：

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

请将压缩包名称替换为与你的架构对应的名称。

## 使用 PRoot 启动

一个基本的启动命令如下：

```sh
proot --kill-on-exit --link2symlink -0 -r kyanite \
  -b /dev -b /proc -b /sys \
  -w /root \
  /usr/bin/env -i \
    HOME=/root USER=root LOGNAME=root \
    TERM=xterm-256color COLORTERM=truecolor LANG=C.UTF-8 \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    /bin/sh -l
```

根据宿主环境的不同，启动器可能需要额外的绑定（`-b`）。在 Android 上，`/proc` 下的某些文件可能需要特殊处理。

## 启动行为

`/bin/sh` 是指向 `/sbin/init` 的符号链接。启动器启动 `/bin/sh` 时，Kyanite init 会以交互式登录 shell 的方式启动 Brush。

有关 shell 和登录环境的更多信息，请参见[使用指南](usage.md)。
