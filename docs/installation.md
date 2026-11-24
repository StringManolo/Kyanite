# Installation

Kyanite runs on Android, Windows, Linux, Termux and Docker. Pick the system you have:

| You have... | Go to |
| --- | --- |
| An Android phone (easiest) | [Android](#android) |
| A Windows PC | [Windows](#windows) |
| A Linux PC | [Linux](#linux) |
| Termux on Android | [Termux](#termux) |
| Docker | [Docker](#docker) |

## Which files are there?

Kyanite is published on the [Kyanite Releases page](https://github.com/StringManolo/Kyanite/releases). The Android app is published separately, in the [KyaniteAndroid releases](https://github.com/StringManolo/KyaniteAndroid/releases/latest).

| What | Files | Used for |
| --- | --- | --- |
| Root filesystem archive | `kyanite-rootfs-aarch64.tar.xz` (ARM64), `kyanite-rootfs-x86_64.tar.xz` (AMD64), `kyanite-rootfs-armv7.tar.xz` (32-bit ARM) | Linux, Termux and Docker. Kyanite runs inside a folder, without a virtual machine. |
| x86_64 disk images | `.img`, `.vdi`, `.vmdk`, `.iso` | Virtual machines and USB sticks. See [x86_64 images](x86_64-images.md). |
| Android app | 64-bit and 32-bit `.apk` | Android |

## Android

The simplest way to get started on an Android device is to install the Kyanite application. Download the version for your phone:

- [Kyanite for Android, 64-bit](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk): most phones.
- [Kyanite for Android, 32-bit](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/kyanite_v7.apk): older phones.

Not sure which one? Try the 64-bit one first. If Android refuses to install it, use the 32-bit one. Some phones have a 64-bit processor but a 32-bit Android system, and they need the 32-bit version. The [Kyanite website](https://stringmanolo.github.io/KyaniteAndroid/en/) also suggests one for you. The 32-bit version does not include the virtual machine mode.

- [KyaniteAndroid source code and documentation](https://github.com/StringManolo/KyaniteAndroid)

The Android application provides its own setup and launch experience. See also the [Android guide](android.md).

## Windows

On Windows, Kyanite runs as a virtual machine, or from a USB stick on a real PC. Download the x86_64 disk images from the release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest). To know what each file is, see [x86_64 images](x86_64-images.md).

### VirtualBox

1. Download `kyanite-x86_64.vdi` and install [VirtualBox](https://www.virtualbox.org/).
2. Click **New**. Name it `Kyanite`, choose type **Linux** and version **Other Linux (64-bit)**. If the wizard offers an unattended installation, skip it.
3. Give it at least 1024 MB of memory.
4. In the hard disk step choose **Use an existing virtual hard disk file**, click the folder icon, press **Add** and select `kyanite-x86_64.vdi`.
5. Finish the wizard, select the machine and press **Start**.

### VMware

1. Download `kyanite-x86_64.vmdk` and install VMware Workstation or VMware Player.
2. Choose **File → New Virtual Machine** and select **Custom (advanced)**.
3. Choose **I will install the operating system later**. As guest operating system select **Linux** and **Other Linux 64-bit** (the exact name changes between VMware versions).
4. Give it at least 1 GB of memory.
5. When it asks for the disk, choose **Use an existing virtual disk**, select `kyanite-x86_64.vmdk` and keep the existing format if asked.
6. Finish and press **Power on this virtual machine**.

### QEMU

1. Install QEMU for Windows from the [official download page](https://www.qemu.org/download/#windows).
2. Download `kyanite-x86_64.img` (or `kyanite-x86_64.img.zip` and extract it).
3. Open PowerShell in the folder with the image and run:

```powershell
& "C:\Program Files\qemu\qemu-system-x86_64.exe" -m 1G -drive file=kyanite-x86_64.img,format=raw
```

It is slow without acceleration. Optionally, turn on the Windows feature *Windows Hypervisor Platform* and add `-accel whpx` to the command.

### Rufus (a real PC from a USB stick)

1. Download `kyanite-x86_64-hw-live.iso` (a live system with an installer) or `kyanite-x86_64-hw.img.zip` (extract it first), and download [Rufus](https://rufus.ie/).
2. Plug in a USB stick. **Everything on it will be erased.**
3. Open Rufus. In **Device** choose your USB stick, press **SELECT** and choose the file (for an `.img`, pick "All files" in the file dialog).
4. Press **START**. When Rufus asks, choose **DD Image mode**, and confirm.
5. Restart the PC, open its boot menu and start from the USB stick.

Before booting a real PC, read [x86_64 images](x86_64-images.md): Secure Boot must be off, and the installer has not been tested on real hardware yet.

## Linux

### PRoot (no root needed)

PRoot runs Kyanite as a normal user, inside a folder. It works on most Linux systems (the host must allow `ptrace`).

1. Install `proot` with your package manager, for example `sudo apt install proot` on Debian or Ubuntu.
2. Open the [Kyanite Releases page](https://github.com/StringManolo/Kyanite/releases) and download the archive for your architecture:

| Architecture | Archive |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |
| 32-bit ARM | `kyanite-rootfs-armv7.tar.xz` |

Download the matching `.sha256` and `.sha256.sig` files too.

3. Check the SHA256 checksum from the directory containing the downloaded files:

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Use the checksum filename matching your downloaded archive. To verify the Ed25519 signature as well, follow [Verifying releases](../SECURITY.md#verifying-releases).

4. Create a directory and extract the archive (replace the archive name with the one matching your architecture):

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

5. Start Kyanite. A basic launch command looks like this:

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

A launcher may need additional bindings depending on the host environment.

### QEMU

QEMU runs the x86_64 disk image as a virtual machine.

```sh
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw
```

Add `-enable-kvm` if your CPU and kernel support it: it is much faster. The image is in the release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

### USB stick (a real PC)

Download `kyanite-x86_64-hw-live.iso` (or `kyanite-x86_64-hw.img.zip` and extract it) from the release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest), and write it to a USB stick with balenaEtcher or `dd`:

```sh
lsblk                                     # find your USB stick, for example /dev/sdX
sudo dd if=kyanite-x86_64-hw-live.iso of=/dev/sdX bs=4M status=progress conv=fsync
```

Replace `/dev/sdX` with your USB stick. **Everything on it is erased, and the wrong device can destroy your data.** Then read [x86_64 images](x86_64-images.md) before booting.

## Termux

You need [Termux](https://termux.dev/), installed from F-Droid or from the Termux GitHub releases.

### proot-distro (recommended)

This needs PRoot-Distro 5 or newer. The quickest way is the installer script: it downloads the right archive for your phone, checks its SHA256 checksum, installs Kyanite and creates a `kyanite` command.

```sh
pkg update && pkg upgrade
curl -fsSL https://raw.githubusercontent.com/StringManolo/Kyanite/main/install-termux.sh -o install-termux.sh
sh install-termux.sh
```

Then type `kyanite` to open a shell, or `kyanite COMMAND` to run a single command.

The same thing, step by step:

```sh
pkg install proot-distro curl
ARCH=aarch64        # use armv7 on a 32-bit phone
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz.sha256
sha256sum -c kyanite-rootfs-$ARCH.tar.xz.sha256
proot-distro install ./kyanite-rootfs-$ARCH.tar.xz --name kyanite
proot-distro login kyanite
```

To get the `kyanite` command by hand:

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

To remove it: `proot-distro remove kyanite && rm "$PREFIX/bin/kyanite"`.

### QEMU

This is experimental, and slow: QEMU emulates a whole PC to run the x86_64 image.

```sh
pkg install qemu-system-x86-64-headless unzip curl
curl -LO https://github.com/StringManolo/Kyanite/releases/download/qemuTest/kyanite-x86_64.img.zip
unzip kyanite-x86_64.img.zip
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -vnc 127.0.0.1:0
```

Open a VNC viewer app on the phone and connect to `127.0.0.1:5900`.

For a graphical window instead of VNC, this is the author's own setup:

1. Install Termux and Termux:X11.
2. Run in Termux: `termux-x11 :0 -listen tcp -ac`
3. In [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android), with an Alpine rootfs, run:

```sh
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. Open the Termux:X11 app.

## Docker

This is experimental. Use the archive that matches your CPU (`aarch64` on ARM computers such as Apple Silicon Macs). Download it and check its checksum first, as shown in the PRoot steps of the [Linux](#linux) section.

```sh
docker import kyanite-rootfs-x86_64.tar.xz kyanite
docker run --rm -it kyanite /bin/sh -l
```

## Startup behavior

`/bin/sh` is a symlink to `/sbin/init`. When the launcher starts `/bin/sh`, Kyanite init starts Brush as an interactive login shell.

For more about the shell and login environment, see the [usage guide](usage.md).
