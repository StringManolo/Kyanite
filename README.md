# Kyanite

Kyanite is a minimal Linux root filesystem, in the same category as the Alpine Linux rootfs, built around memory-safe Rust components instead of BusyBox and a C userland. It is designed to be small, secure and comfortable to use, and it is intended to grow into a complete distribution (kernel included).

[![Build Kyanite Rootfs](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml/badge.svg)](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml)

> **Status:** stable enough for daily experimentation, not yet feature-complete. See [Known limitations](#known-limitations) and [Roadmap](#roadmap).

## Why Kyanite

| Problem | Typical answer | Kyanite |
|---|---|---|
| BusyBox is written in C and has a long history of memory-safety bugs | Accept it | Replaced by [uutils coreutils](https://github.com/uutils/coreutils), written in Rust |
| Alpine packages are built against musl, which breaks compatibility with a lot of software and has a smaller package set | Switch to Debian | Packages installed with `kpkg` come from Debian and run on glibc |
| Debian rootfs images are large (close to 100 MB) | Accept it | About 30 MB uncompressed, about 8 MB as `tar.xz` |

Kyanite uses **musl only for the preinstalled components** (init, package manager, shell, coreutils), which are statically linked Rust binaries. Everything you install afterwards is a Debian package linked against **glibc**. The result is a small, memory-safe base with the package ecosystem and compatibility of Debian.

## What is inside

| Component | Role | Notes |
|---|---|---|
| `kyanite-init` | PID 1 (`/sbin/init`) | Rust. Starts the shell as a login shell, sets up job control and tty ownership, reaps orphaned processes, exits when the shell exits |
| [`brush`](https://github.com/reubeno/brush) | Interactive shell (`/bin/brush`) | Bash-compatible shell written in Rust |
| [`coreutils`](https://github.com/uutils/coreutils) | `ls`, `cat`, `cp`, `mv`, `rm`, `sort`, ... | One multicall binary (`/usr/bin/coreutils`) with an applet symlink per command |
| `kpkg` | Package manager (`/usr/bin/kpkg`) | Rust. apk-style CLI over Debian repositories. See [kpkg](#kpkg) |
| glibc runtime | `libc6`, `libgcc-s1`, dynamic loader, `env` | Distribution packages fetched at build time, so that packages installed with `kpkg` work out of the box |
| `vi` | `vim-tiny` (`/usr/bin/vi`, `vim`, `ex`, `view`) | The only C component of the interactive base besides glibc. No mature, vi-compatible Rust editor exists yet |

`/bin/sh` is a symlink to `/sbin/init`, so launchers that start `/bin/sh` end up in Kyanite's init, which then starts Brush.

## Supported architectures

- `aarch64` (arm64)
- `x86_64` (amd64)

## Getting Kyanite

Release archives are published on the [Releases page](https://github.com/StringManolo/Kyanite/releases) for every `v*` tag. Each release contains, per architecture:

- `kyanite-rootfs-<arch>.tar.xz`
- `kyanite-rootfs-<arch>.tar.xz.sha256`

Verify the download before using it:

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Every push to `main` also produces the same files as workflow artifacts.

## Usage

Kyanite is a rootfs, not a bootable image (yet). It currently runs in userspace through `proot`.

**Tested on:** [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android).

**Expected to work on:** `proot-distro` in Termux, which is what DebianLiveCD4Android uses internally. This has not been verified yet.

**Not tested yet:** QEMU, x86 hosts, udocker.

DebianLiveCD4Android launches the rootfs with a command equivalent to the following (simplified: it also binds `/dev/fd`, `/dev/stdin`, `/dev/stdout`, `/dev/stderr` and `/dev/shm`, and optionally shared storage):

```sh
mkdir kyanite && tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite

proot --kill-on-exit --link2symlink -0 -r kyanite \
  -b /dev -b /proc -b /sys \
  -w /root \
  /usr/bin/env -i \
    HOME=/root USER=root LOGNAME=root \
    TERM=xterm-256color COLORTERM=truecolor LANG=C.UTF-8 \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    /bin/sh -l
```

Notes:

- The launcher runs `/bin/bash` if it exists and falls back to `/bin/sh -l` otherwise. Kyanite has no Bash, so it takes the fallback: `/bin/sh` is a symlink to `/sbin/init`, which starts Brush as an interactive login shell and ignores the extra arguments.
- `/usr/bin/env` is part of the rootfs precisely because launchers use it to build a clean environment.
- On Android 8 and later several files under `/proc` are not readable by apps. DebianLiveCD4Android binds replacement files for them; if you run `proot` by hand on Android you may need to do the same.
- When the shell exits, the init exits and the session ends.

## kpkg

`kpkg` is a lightweight package manager with an [apk](https://wiki.alpinelinux.org/wiki/Alpine_Package_Keeper)-style interface that installs packages from Debian repositories.

```sh
kpkg update              # fetch and verify the package index
kpkg search <term>       # search names and descriptions
kpkg add <pkg>...        # install packages and their dependencies
kpkg del <pkg>...        # remove packages
kpkg upgrade             # upgrade all installed packages
kpkg fix <pkg>...        # reinstall (repair)
kpkg info [pkg]          # package info; -L files, -W owner of a path, -R deps, -r reverse deps
kpkg list [pattern]      # available packages; -I installed, -u upgradable
kpkg policy <pkg>...     # installed and candidate versions
kpkg audit               # verify installed files against recorded SHA256 sums
kpkg cache clean|size    # manage the downloaded .deb cache
```

Options:

| Option | Description |
|---|---|
| `-s`, `--simulate` | Show what would be done without changing anything |
| `-f`, `--force` | Overwrite unowned or foreign files, ignore reverse dependencies |
| `--root <dir>` | Operate on another root (default `/`) |
| `--arch <arch>` | Debian architecture (default: that of the build) |
| `--mirror <url>` | Repository mirror (default `https://deb.debian.org/debian`) |
| `--suite <name>` | Debian suite (default `stable`) |

Design notes:

- Dependency resolution handles `Pre-Depends`, `Depends` (including `a | b` alternatives and versioned constraints), virtual packages via `Provides`, `Conflicts` and `Breaks`. `Recommends` and `Suggests` are ignored.
- Version comparison follows the dpkg algorithm.
- Downloads are checked for size and SHA256 against the repository index before being written to their final location.
- Extraction is restricted to the target root, enforces size limits and never installs setuid/setgid/sticky bits.
- Packages that ship with the base rootfs are declared in `/etc/kpkg/assume-installed` (`libc6`, `libgcc-s1`, `coreutils`) so they are not pulled in again.
- State lives in `/var/lib/kpkg`; downloaded packages are cached in `/var/cache/kpkg/pkgs`.

## Shell environment

Kyanite ships a small, editable login experience, installed from the [`rootfs-overlay/`](rootfs-overlay/) directory of this repository.

| Path | Purpose |
|---|---|
| `/etc/profile` | Read by Brush on login. Prints the MOTD and loads `~/.bashrc` |
| `/root/.bashrc` | Prompt (`user@host:/path#`), unlimited history, aliases, `extract` and `gacp` helpers |
| `/etc/update-motd.d/` | MOTD scripts, run in name order at every login. Remove the executable bit (`chmod -x`) to disable one |
| `/etc/motd` | Optional static text, printed after the scripts (not shipped; create it if you want one) |
| `/etc/kyanite/logo.txt` | ASCII logo shown by `00-logo`. Replace it with any pre-coloured text, for example `chafa --size=36x --colors=256 logo.png > logo.txt` |
| `/etc/os-release` | Name and version of the build (also used by the MOTD) |

To silence the MOTD, run `touch ~/.hushlogin`. Colours are disabled when `NO_COLOR` is set or `TERM` is `dumb`.

Because `/bin/sh` is `kyanite-init`, MOTD scripts are executed with `/bin/brush` and do not need a `#!/bin/sh` shebang.

Some commands used by the aliases and helpers (`tar`, `git`, `curl`, `unzip`, ...) are not part of the base image; install them with `kpkg add <package>`.

## Security

Security is a primary goal of the project. The threat model, the protections implemented in `kpkg` and the known gaps are documented in [SECURITY.md](SECURITY.md), including how to report a vulnerability.

## Known limitations

- `kpkg` **does not run maintainer scripts** (`preinst`, `postinst`, ...). Packages that depend on them to be usable may need manual setup.
- The OpenPGP signature of the repository `InRelease` file is **not verified yet**. Trust currently rests on HTTPS plus the hashes in the release index. See [SECURITY.md](SECURITY.md).
- `vi` is `vim-tiny`, written in C. The rest of the interactive base is Rust.
- The glibc runtime and `vi` are taken from the build runner's distribution (Ubuntu), while `kpkg` installs from Debian `stable`. Their glibc versions may differ.
- There is no kernel yet; Kyanite runs as a rootfs under `proot`.
- The x86_64 image is roughly twice the size of the aarch64 one. The cause is under investigation.

## Roadmap

Planned work, in rough order:

- Run package maintainer scripts in `kpkg`
- OpenPGP verification of repository metadata
- Replace the remaining GNU `env` and `vim-tiny` with Rust implementations
- Build the glibc runtime from Debian `stable` packages so it matches the repositories used by `kpkg`
- Audit every file in the rootfs
- Better logging in the init and preconfigured scripts
- Testing on QEMU, x86 hosts, udocker and `proot-distro` (and a proposal to Termux's distribution list)
- Kernel and the remaining pieces of a full distribution

## Building from source

The build is defined in [`.github/workflows/build-rootfs.yml`](.github/workflows/build-rootfs.yml). In short, for each target (`aarch64-unknown-linux-musl`, `x86_64-unknown-linux-musl`) it:

0. Computes the version with `git describe --tags --always`.
1. Builds `init` and `kpkg` from this repository with Cargo.
2. Builds `brush-shell` and `coreutils` (`feat_os_unix_musl`) with `cargo install`.
3. Assembles the rootfs, adds the glibc runtime, `env` and `vi` from distribution packages, and applies `rootfs-overlay/`.
4. Compresses it to `tar.xz` and generates the SHA256 file.

To build locally you need a Rust toolchain with the musl targets installed, `musl-tools`, and for aarch64 cross-compilation `gcc-aarch64-linux-gnu`.

## Releasing

Pushing a tag that starts with `v` builds both architectures and publishes the archives and checksums as a GitHub release:

```sh
git tag -a v0.1.0 -m "Kyanite 0.1.0"
git push origin v0.1.0
```

The tag name becomes the version shown in the MOTD and in `/etc/os-release`. Builds from other commits are labelled with `git describe` (for example `v0.1.0-3-g1a2b3c4`, or just the commit hash while no tag exists).

## Third-party components and licenses

Kyanite bundles software from other projects. Each one is distributed under its own license; refer to the upstream projects for the authoritative terms. Debian-packaged components are currently fetched from the build runner's (Ubuntu) repositories, which ship the same upstream software.

| Component | Upstream | License |
|---|---|---|
| Brush | [reubeno/brush](https://github.com/reubeno/brush) | MIT |
| uutils coreutils | [uutils/coreutils](https://github.com/uutils/coreutils) | MIT |
| glibc (`libc6`) | [Debian](https://packages.debian.org/stable/libc6) | LGPL-2.1-or-later |
| `libgcc-s1` | [Debian](https://packages.debian.org/stable/libgcc-s1) | GPL-3.0-or-later with the GCC Runtime Library Exception |
| GNU `env` (from GNU coreutils) | [Debian](https://packages.debian.org/stable/coreutils) | GPL-3.0-or-later |
| `vim-tiny`, `vim-common` | [Vim](https://www.vim.org/) / [Debian](https://packages.debian.org/stable/vim-tiny) | Vim license |
| `libacl1` | [Debian](https://packages.debian.org/stable/libacl1) | LGPL-2.1-or-later |
| `libselinux1` | [Debian](https://packages.debian.org/stable/libselinux1) | Public domain |
| `libpcre2-8-0` | [Debian](https://packages.debian.org/stable/libpcre2-8-0) | BSD-3-Clause |
| `libtinfo6`, `ncurses-base` | [Debian](https://packages.debian.org/stable/ncurses-base) | X11/MIT-style (ncurses) |

Packages installed afterwards with `kpkg` carry the licenses of their respective Debian packages.
