🇺🇸 English | [🇪🇸 Español](README.es.md) | [🇷🇺 Русский](README.ru.md) | [🇨🇳 简体中文](README.zh-CN.md)

# Kyanite

Kyanite is a small Linux system that is easy to carry and easy to use. Its core tools are written in Rust, a programming language that helps avoid many common memory bugs. You can install thousands of extra programs from Debian with a single command.

![Kyanite login screen](docs/screenshot.png)

[![Build Kyanite Rootfs](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml/badge.svg)](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml)

## In short

- **Small:** a compact base system, quick to download and start.
- **Safer by design:** the init system, package manager, shell and core commands are written in Rust.
- **Lots of software:** `kpkg` installs programs from Debian's repositories.
- **Familiar:** common commands and an interactive shell, ready to use.
- **Runs as an app:** it does not replace your operating system. On Android it is a normal app; on Linux it runs inside a folder.

New to Linux? The [glossary](docs/glossary.md) explains the technical words used here.

## Get Kyanite

### On Android (easiest)

1. [**Download Kyanite for Android (.apk)**](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
2. Open the downloaded file and install it (Android may ask you to allow installing apps from this source).
3. Open the app and choose how to start Kyanite on the first screen.

More details in the [Android guide](docs/android.md). Source code: [KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid).

### On Linux

Download a root filesystem archive from the [Releases page](https://github.com/StringManolo/Kyanite/releases). Three architectures are available:

- `aarch64` (ARM64)
- `x86_64` (AMD64)
- `armv7` (32-bit ARM)

Each release includes the archive, its SHA256 checksum and a signature, so you can check that the download is genuine. Follow the [installation guide](docs/installation.md) step by step.

Prefer a virtual machine or a USB stick? There are also ready-made x86_64 disk images for QEMU, USB sticks and a live ISO. See the [x86_64 images guide](docs/x86_64-images.md). For example, with QEMU:

```bash
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw
```

### On Windows

Kyanite has x86_64 disk images that work with the usual Windows programs. Download them from the release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

| Program | File |
| --- | --- |
| VirtualBox | `kyanite-x86_64.vdi` |
| VMware | `kyanite-x86_64.vmdk` |
| Rufus (to boot a real PC from a USB stick, in "DD Image" mode) | `kyanite-x86_64-hw-live.iso`, or `kyanite-x86_64-hw.img.zip` (extract it first) |

Before booting a real PC, read the [x86_64 images guide](docs/x86_64-images.md): Secure Boot must be off, and the installer has not been tested on real hardware yet.

### With proot (any Linux environment)

[proot](https://proot-me.github.io/) runs a root filesystem as a normal user: no root, no changes to your system. It works on most Linux environments, including regular distributions, Android (Termux), WSL and many containers (as long as `ptrace` is allowed). The Kyanite app for Android uses it too.

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-x86_64.tar.xz -C kyanite    # on 64-bit ARM use the aarch64 archive, on 32-bit ARM the armv7 one

proot -0 -r kyanite -b /dev -b /proc -b /sys -w /root \
  /usr/bin/env -i HOME=/root TERM="$TERM" LANG=C.UTF-8 \
  PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  /bin/sh -l
```

### In Termux

There are two ways, both without root.

**With proot-distro** (fast: it runs Kyanite directly on your phone). Run this once to register Kyanite as a custom distribution (it picks the archive that matches your phone):

```sh
pkg install proot-distro curl
PD=$(dpkg --print-architecture)                 # aarch64, arm or x86_64
case "$PD" in arm) A=armv7 ;; *) A=$PD ;; esac  # name used by the Kyanite archives
URL=https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$A.tar.xz
SHA=$(curl -fsSL "$URL.sha256" | cut -d' ' -f1)
cat > "$PREFIX/etc/proot-distro/kyanite.sh" <<EOF
DISTRO_NAME="Kyanite"
TARBALL_STRIP_OPT=0
TARBALL_URL['$PD']="$URL"
TARBALL_SHA256['$PD']="$SHA"
EOF
proot-distro install kyanite
```

Then, whenever you want to use it:

```sh
proot-distro login kyanite -- /bin/sh -l
```

**With QEMU** (slow: it emulates a whole PC, but lets you try the x86_64 images):

```sh
pkg install qemu-system-x86-64-headless unzip curl
curl -LO https://github.com/StringManolo/Kyanite/releases/download/qemuTest/kyanite-x86_64.img.zip
unzip kyanite-x86_64.img.zip
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -vnc 127.0.0.1:0
```

Then open a VNC viewer app on the phone and connect to `127.0.0.1:5900`. Other ways of showing the screen are described in the [x86_64 images guide](docs/x86_64-images.md).

## Your first steps

Once Kyanite is running, you can try a few basic commands:

```sh
pwd        # show the folder you are in
ls         # list the files in this folder
cd /etc    # move to another folder
```

To install software, use `kpkg`:

```sh
kpkg update
kpkg add curl
```

The [usage guide](docs/usage.md) and the [kpkg guide](docs/kpkg.md) have more examples.

## What is included?

| Component | What it does |
| --- | --- |
| `kyanite-init` | Starts the system and opens the shell |
| [Brush](https://github.com/reubeno/brush) | The shell: the program where you type commands |
| [uutils coreutils](https://github.com/uutils/coreutils) | Everyday commands such as `ls` and `cp`, written in Rust |
| `kpkg` | The package manager: installs and removes programs from Debian |
| glibc runtime | Lets Debian programs run |
| `vi` | A simple text editor |

Want to know how the pieces fit together? Read the [architecture guide](docs/architecture.md).

## Documentation

| Guide | What it covers |
| --- | --- |
| [Glossary](docs/glossary.md) | Technical words explained in plain language |
| [Android guide](docs/android.md) | Using Kyanite on your phone |
| [Installation](docs/installation.md) | Downloading, verifying and starting Kyanite |
| [x86_64 images](docs/x86_64-images.md) | Disk images for PCs and virtual machines (QEMU, VirtualBox, VMware) |
| [Usage](docs/usage.md) | Shell environment and customization |
| [Package manager (`kpkg`)](docs/kpkg.md) | Commands, options and package management |
| [Package hooks](docs/package-hooks.md) | How Kyanite configures packages |
| [Architecture](docs/architecture.md) | Components and system design |
| [Development](docs/development.md) | Building and working on the source |
| [Releases](docs/releases.md) | Versioning and release process |
| [Known limitations](docs/limitations.md) | Current compatibility notes and technical limitations |
| [Roadmap](docs/roadmap.md) | Areas being explored and possible future improvements |

## Security

Security is an important design goal. [SECURITY.md](SECURITY.md) (in English) describes the threat model, the protections already in place, known gaps and how to report a vulnerability.

## Contributing

Kyanite is developed in the open. The [development guide](docs/development.md) explains how to build the project and where to start.
