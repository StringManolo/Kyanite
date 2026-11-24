🇺🇸 English | [🇪🇸 Español](es/x86_64-images.md) | [🇷🇺 Русский](ru/x86_64-images.md) | [🇨🇳 简体中文](zh-CN/x86_64-images.md)

# x86_64 images

Kyanite also ships ready-to-use disk images for x86_64: PCs, virtual machines and Windows/Linux hosts. They are published in the GitHub release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

Looking for the root filesystem archives (`aarch64` / `x86_64`) instead? See the [installation guide](installation.md).

> ⚠️ The installer included in the live ISO has **not been tested on real hardware yet** and can permanently delete all your files. If you do not know what you are doing, do not run it. Back up your data first.

## Which file do I need?

| I want to... | File |
|---|---|
| Try Kyanite in QEMU | `kyanite-x86_64.img` |
| Use VirtualBox | `kyanite-x86_64.vdi` |
| Use VMware | `kyanite-x86_64.vmdk` |
| Write a USB stick with Rufus, **no** firmware included | `kyanite-x86_64.img.zip` |
| Write a USB stick for a real PC, firmware **included** | `kyanite-x86_64-hw.img.zip` |
| Get the typical Linux live CD (with an installer script) | `kyanite-x86_64-hw-live.iso` |

## QEMU (BIOS)

<img width="720" height="391" alt="Kyanite GRUB menu in QEMU" src="https://github.com/user-attachments/assets/032289ef-c50f-4e2f-b491-79b66ea7fecb" />

This is just my setup; you can use any other.

1. Install Termux and Termux:X11.
2. Run in Termux:

```bash
termux-x11 :0 -listen tcp -ac
```

3. Run in [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) with an Alpine rootfs (this assumes `qemu-system-x86_64` is already installed):

```bash
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. Open the Termux:X11 app and you will see the BIOS.

<img width="720" height="395" alt="Kyanite booting in QEMU" src="https://github.com/user-attachments/assets/5f5197f0-e16f-4dba-b45b-0e99b841d2ba" />

## VirtualBox

Use `kyanite-x86_64.vdi`.

## VMware

Use `kyanite-x86_64.vmdk`.

## Rufus (USB stick)

Extract the zip first and write the `.img` in **DD Image** mode. The image lives on its own virtual drive.

- `kyanite-x86_64.img.zip`: **no** firmware included.
- `kyanite-x86_64-hw.img.zip`: firmware **included**.

The images are distributed inside a zip because they are about 90% smaller compressed: they contain a virtual disk with more than 500 MB of zeros as extra storage. Once extracted they can take up to 1.6 GB.

## Real hardware (the `hw` images)

The *hardware* image set is meant for real PCs and includes `linux-firmware`.

| File | What it is |
|---|---|
| `kyanite-x86_64-hw-live.iso` | Live system with an installer. Write it to a USB stick with Rufus in "DD Image" mode (or balenaEtcher / `dd`). Boot the PC from it in UEFI or BIOS mode. |
| `kyanite-x86_64-hw.img` | The same system already installed on a disk image. Write it to a USB stick with Rufus in "DD Image" mode, or use it as a virtual disk. |

### Before you boot

- **Secure Boot must be disabled** (the bootloader and the kernel are not signed).
- If the NVMe/SATA disk is not visible, switch the SATA mode from RAID/Intel RST to AHCI in the firmware setup.
- Keyboard layout: the GRUB menu has a "keyboard layout" submenu (es, de, fr, it, gb, pt, br-abnt2).

### Installing from the live system

Run `/usr/local/sbin/kyanite-install`. It offers two options:

1. **Erase the whole disk** and install Kyanite (BIOS and UEFI).
2. **Install alongside the existing system** (UEFI only, GPT disk, with an EFI System Partition and 2 GiB of free space). This keeps Windows and your data:
   - Shrink the Windows partition from Windows first (Disk Management).
   - If the disk is encrypted with BitLocker, suspend it first.
   - Kyanite gets its own partition and reuses the existing EFI System Partition. The GRUB menu gets a "Windows" entry. You can also choose Windows or Kyanite from the firmware boot menu.

### Not included

- Secure Boot support.
- Automatic network configuration (DHCP).
- Dual boot with BIOS/MBR Windows.
