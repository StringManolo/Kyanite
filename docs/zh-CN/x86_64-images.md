🇨🇳 简体中文 | [🇺🇸 English](../x86_64-images.md) | [🇪🇸 Español](../es/x86_64-images.md) | [🇷🇺 Русский](../ru/x86_64-images.md)

# x86_64 镜像

Kyanite 还提供可直接使用的 x86_64 磁盘镜像，适用于 PC、虚拟机以及 Windows/Linux 主机。镜像发布在 GitHub 的这个 release 中：[x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest)。

想找的是根文件系统压缩包（`aarch64` / `x86_64`）？请看[安装指南](installation.md)。

> ⚠️ live ISO 中自带的安装程序**尚未在真实硬件上测试**，可能会永久删除你的所有文件。如果你不清楚自己在做什么，请不要运行它。请先备份你的数据。

## 我需要哪个文件？

| 我想要…… | 文件 |
|---|---|
| 在 QEMU 中试用 Kyanite | `kyanite-x86_64.img` |
| 使用 VirtualBox | `kyanite-x86_64.vdi` |
| 使用 VMware | `kyanite-x86_64.vmdk` |
| 用 Rufus 写入 U 盘，**不含**固件 | `kyanite-x86_64.img.zip` |
| 为真实 PC 写入 U 盘，**含**固件 | `kyanite-x86_64-hw.img.zip` |
| 常见的 Linux live CD（带安装脚本） | `kyanite-x86_64-hw-live.iso` |

## QEMU（BIOS）

<img width="720" height="391" alt="Kyanite GRUB menu in QEMU" src="https://github.com/user-attachments/assets/032289ef-c50f-4e2f-b491-79b66ea7fecb" />

这只是我自己的配置，你也可以使用其他方式。

1. 安装 Termux 和 Termux:X11。
2. 在 Termux 中运行：

```bash
termux-x11 :0 -listen tcp -ac
```

3. 在 [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) 中，使用 Alpine rootfs 运行（假设已安装 `qemu-system-x86_64`）：

```bash
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. 打开 Termux:X11 应用，即可看到 BIOS。

<img width="720" height="395" alt="Kyanite booting in QEMU" src="https://github.com/user-attachments/assets/5f5197f0-e16f-4dba-b45b-0e99b841d2ba" />

## VirtualBox

使用 `kyanite-x86_64.vdi`。

## VMware

使用 `kyanite-x86_64.vmdk`。

## Rufus（U 盘）

请先解压 zip，再以 **DD 镜像** 模式写入 `.img`。该镜像位于它自己的虚拟磁盘上。

- `kyanite-x86_64.img.zip`：**不含**固件。
- `kyanite-x86_64-hw.img.zip`：**含**固件。

镜像放在 zip 中分发，是因为压缩后体积约小 90%：它们包含一个虚拟磁盘，其中有超过 500 MB 的零作为额外存储空间。解压后最多可能占用 1.6 GB。

## 真实硬件（`hw` 镜像）

*hardware* 镜像套装面向真实 PC，并包含 `linux-firmware`。

| 文件 | 说明 |
|---|---|
| `kyanite-x86_64-hw-live.iso` | 带安装程序的 live 系统。用 Rufus 的“DD 镜像”模式（或 balenaEtcher / `dd`）写入 U 盘，然后用 UEFI 或 BIOS 模式从它启动 PC。 |
| `kyanite-x86_64-hw.img` | 同一系统，已安装在磁盘镜像中。用 Rufus 的“DD 镜像”模式写入 U 盘，或将其用作虚拟磁盘。 |

### 启动之前

- **必须关闭 Secure Boot**（引导程序和内核均未签名）。
- 如果看不到 NVMe/SATA 磁盘，请在固件设置中把 SATA 模式从 RAID/Intel RST 改为 AHCI。
- 键盘布局：GRUB 菜单中有“keyboard layout”子菜单（es、de、fr、it、gb、pt、br-abnt2）。

### 从 live 系统安装

运行 `/usr/local/sbin/kyanite-install`。它提供两种方式：

1. **清除整个磁盘**并安装 Kyanite（支持 BIOS 和 UEFI）。
2. **与现有系统并存安装**（仅限 UEFI、GPT 磁盘，需要有 EFI 系统分区和 2 GiB 空闲空间）。这样可以保留 Windows 和你的数据：
   - 请先在 Windows 中压缩 Windows 分区（磁盘管理）。
   - 如果磁盘使用 BitLocker 加密，请先暂停保护。
   - Kyanite 会获得自己的分区，并复用现有的 EFI 系统分区。GRUB 菜单会增加一个“Windows”条目。你也可以在固件的启动菜单中选择 Windows 或 Kyanite。

### 不包含

- Secure Boot 支持。
- 自动网络配置（DHCP）。
- 与 BIOS/MBR 模式 Windows 的双系统启动。
