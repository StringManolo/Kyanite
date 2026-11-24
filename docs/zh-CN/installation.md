# 安装

Kyanite 可以在 Android、Windows、Linux、Termux 和 Docker 上运行。请选择你使用的系统：

| 你使用的是…… | 前往 |
| --- | --- |
| Android 手机（最简单） | [Android](#android) |
| Windows 电脑 | [Windows](#windows) |
| Linux 电脑 | [Linux](#linux) |
| Android 上的 Termux | [Termux](#termux) |
| Docker | [Docker](#docker) |

## 有哪些文件？

Kyanite 发布在 [Kyanite 的 Releases 页面](https://github.com/StringManolo/Kyanite/releases)。Android 应用单独发布，在 [KyaniteAndroid 的 Releases](https://github.com/StringManolo/KyaniteAndroid/releases/latest)。

| 内容 | 文件 | 用途 |
| --- | --- | --- |
| 根文件系统压缩包 | `kyanite-rootfs-aarch64.tar.xz`（ARM64）、`kyanite-rootfs-x86_64.tar.xz`（AMD64）、`kyanite-rootfs-armv7.tar.xz`（32 位 ARM） | Linux、Termux 和 Docker。Kyanite 在一个文件夹中运行，无需虚拟机。 |
| x86_64 磁盘镜像 | `.img`、`.vdi`、`.vmdk`、`.iso` | 虚拟机和 U 盘。参见 [x86_64 镜像](x86_64-images.md)。 |
| Android 应用 | 64 位和 32 位 `.apk` | Android |

## Android

在 Android 设备上，最简单的方式是安装 Kyanite 应用。请下载适合你手机的版本：

- [Kyanite for Android，64 位](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)：大多数手机。
- [Kyanite for Android，32 位](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/kyanite_v7.apk)：较旧的手机。

不确定该选哪个？先试 64 位版本。如果 Android 拒绝安装，就用 32 位版本。有些手机的处理器是 64 位的，但 Android 系统是 32 位的，它们需要 32 位版本。[Kyanite 网站](https://stringmanolo.github.io/KyaniteAndroid/zh-CN/)也会为你推荐。32 位版本不包含虚拟机模式。

- [KyaniteAndroid 源代码和文档](https://github.com/StringManolo/KyaniteAndroid)

Android 应用有自己的设置和启动流程。另请参阅 [Android 指南](android.md)。

## Windows

在 Windows 上，Kyanite 可以作为虚拟机运行，也可以从 U 盘在真实电脑上启动。请从 release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest) 下载 x86_64 磁盘镜像。想了解每个文件是什么，请参阅 [x86_64 镜像](x86_64-images.md)。

### VirtualBox

1. 下载 `kyanite-x86_64.vdi`，并安装 [VirtualBox](https://www.virtualbox.org/)。
2. 点击**新建**。名称填 `Kyanite`，类型选**Linux**，版本选 **Other Linux (64-bit)**。如果向导提供无人值守安装，请跳过。
3. 至少分配 1024 MB 内存。
4. 在硬盘步骤中选择**使用已有的虚拟硬盘文件**，点击文件夹图标，按**添加**，然后选择 `kyanite-x86_64.vdi`。
5. 完成向导，选中这台虚拟机，点击**启动**。

### VMware

1. 下载 `kyanite-x86_64.vmdk`，并安装 VMware Workstation 或 VMware Player。
2. 选择 **File → New Virtual Machine**，然后选择 **Custom (advanced)**。
3. 选择 **I will install the operating system later**。客户机操作系统选择 **Linux** 和 **Other Linux 64-bit**（确切名称因 VMware 版本而异）。
4. 至少分配 1 GB 内存。
5. 当向导询问磁盘时，选择 **Use an existing virtual disk**，选中 `kyanite-x86_64.vmdk`，如果询问格式，请保持现有格式。
6. 完成设置，点击 **Power on this virtual machine**。

### QEMU

1. 从[官方下载页面](https://www.qemu.org/download/#windows)安装 Windows 版 QEMU。
2. 下载 `kyanite-x86_64.img`（或者下载 `kyanite-x86_64.img.zip` 并解压）。
3. 在镜像所在的文件夹中打开 PowerShell，运行：

```powershell
& "C:\Program Files\qemu\qemu-system-x86_64.exe" -m 1G -drive file=kyanite-x86_64.img,format=raw
```

没有加速时运行较慢。你也可以启用 Windows 功能 *Windows Hypervisor Platform*，并在命令中加入 `-accel whpx`。

### Rufus（从 U 盘启动真实电脑）

1. 下载 `kyanite-x86_64-hw-live.iso`（带安装程序的 live 系统）或 `kyanite-x86_64-hw.img.zip`（需先解压），并下载 [Rufus](https://rufus.ie/)。
2. 插入 U 盘。**U 盘上的所有内容都会被清除。**
3. 打开 Rufus。在**设备**中选择你的 U 盘，点击**选择**并选中该文件（如果是 `.img`，请在文件对话框中选择“所有文件”）。
4. 点击**开始**。当 Rufus 询问时，选择 **DD 镜像模式**并确认。
5. 重启电脑，打开启动菜单，从 U 盘启动。

在真实电脑上启动之前，请先阅读 [x86_64 镜像](x86_64-images.md)：必须关闭 Secure Boot，且安装程序尚未在真实硬件上测试。

## Linux

### PRoot（无需 root）

PRoot 以普通用户身份在一个文件夹中运行 Kyanite。它适用于大多数 Linux 系统（宿主系统必须允许 `ptrace`）。

1. 用包管理器安装 `proot`，例如在 Debian 或 Ubuntu 上运行 `sudo apt install proot`。
2. 打开 [Kyanite 的 Releases 页面](https://github.com/StringManolo/Kyanite/releases)，下载与你的架构对应的压缩包：

| 架构 | 压缩包 |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |
| 32 位 ARM | `kyanite-rootfs-armv7.tar.xz` |

同时下载对应的 `.sha256` 和 `.sha256.sig` 文件。

3. 在包含已下载文件的目录中检查 SHA256 校验和：

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

请使用与你下载的压缩包相对应的校验和文件名。若还要验证 Ed25519 签名，请参阅[验证发布版本](../../SECURITY.md#verifying-releases)（英文）。

4. 创建一个目录并把压缩包解压到其中（请把压缩包名称换成与你的架构对应的名称）：

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

5. 启动 Kyanite。一个基本的启动命令如下：

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

根据宿主环境的不同，启动器可能需要额外的绑定（`-b`）。

### QEMU

QEMU 把 x86_64 磁盘镜像作为虚拟机运行。

```sh
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw
```

如果你的 CPU 和内核支持，请加上 `-enable-kvm`，这样快得多。镜像在 release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest) 中。

### U 盘（真实电脑）

从 release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest) 下载 `kyanite-x86_64-hw-live.iso`（或下载 `kyanite-x86_64-hw.img.zip` 并解压），然后用 balenaEtcher 或 `dd` 写入 U 盘：

```sh
lsblk                                     # 找到你的 U 盘，例如 /dev/sdX
sudo dd if=kyanite-x86_64-hw-live.iso of=/dev/sdX bs=4M status=progress conv=fsync
```

请把 `/dev/sdX` 换成你的 U 盘。**U 盘上的所有内容都会被清除，选错设备可能会毁掉你的数据。**启动之前请先阅读 [x86_64 镜像](x86_64-images.md)。

## Termux

你需要 [Termux](https://termux.dev/)，可从 F-Droid 或 Termux 的 GitHub Releases 安装。

### proot-distro（推荐）

需要 PRoot-Distro 5 或更新版本。最快的方式是使用安装脚本：它会下载适合你手机的压缩包，检查其 SHA256 校验和，安装 Kyanite 并创建 `kyanite` 命令。

```sh
pkg update && pkg upgrade
curl -fsSL https://raw.githubusercontent.com/StringManolo/Kyanite/main/install-termux.sh -o install-termux.sh
sh install-termux.sh
```

然后输入 `kyanite` 打开 shell，或输入 `kyanite 命令` 来运行单个命令。

同样的操作，分步进行：

```sh
pkg install proot-distro curl
ARCH=aarch64        # 32 位手机请使用 armv7
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz.sha256
sha256sum -c kyanite-rootfs-$ARCH.tar.xz.sha256
proot-distro install ./kyanite-rootfs-$ARCH.tar.xz --name kyanite
proot-distro login kyanite
```

手动创建 `kyanite` 命令：

```sh
cat > "$PREFIX/bin/kyanite" <<LAUNCHER
#!$PREFIX/bin/sh
if [ "\$#" -eq 0 ]; then
  exec proot-distro login kyanite
else
  exec proot-distro login kyanite -- "\$@"
fi
LAUNCHER
chmod 755 "$PREFIX/bin/kyanite"
```

卸载：`proot-distro remove kyanite && rm "$PREFIX/bin/kyanite"`。

### QEMU

这是实验性的方式，而且很慢：QEMU 需要模拟一整台电脑来运行 x86_64 镜像。

```sh
pkg install qemu-system-x86-64-headless unzip curl
curl -LO https://github.com/StringManolo/Kyanite/releases/download/qemuTest/kyanite-x86_64.img.zip
unzip kyanite-x86_64.img.zip
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -vnc 127.0.0.1:0
```

在手机上打开 VNC 查看器应用，连接到 `127.0.0.1:5900`。

如果想用图形窗口代替 VNC，下面是作者自己的配置：

1. 安装 Termux 和 Termux:X11。
2. 在 Termux 中运行：`termux-x11 :0 -listen tcp -ac`
3. 在 [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) 中，使用 Alpine rootfs 运行：

```sh
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. 打开 Termux:X11 应用。

## Docker

这是实验性的方式。请使用与你的 CPU 对应的压缩包（ARM 电脑，例如 Apple Silicon 的 Mac，使用 `aarch64`）。请先下载并检查校验和，方法见 [Linux](#linux) 一节中的 PRoot 步骤。

```sh
docker import kyanite-rootfs-x86_64.tar.xz kyanite
docker run --rm -it kyanite /bin/sh -l
```

## 启动行为

`/bin/sh` 是指向 `/sbin/init` 的符号链接。当启动器运行 `/bin/sh` 时，Kyanite init 会把 Brush 作为交互式登录 shell 启动。

关于 shell 和登录环境的更多信息，请参阅[使用指南](usage.md)。
