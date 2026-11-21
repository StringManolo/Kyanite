# Kyanite

Kyanite is a minimal Linux root filesystem in the same category as
Alpine Linux, built around memory-safe Rust components instead of
BusyBox and a C userland. It is designed to be small, secure and
comfortable to use, and is intended to grow into a complete
distribution, kernel included.

![Kyanite login screen](docs/screenshot.png)

[![Build Kyanite
Rootfs](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml/badge.svg)](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml)

**Status:** stable enough for daily experimentation, but not yet
feature-complete. See [Known limitations](docs/limitations.md) and
[Roadmap](#roadmap).

## Why Kyanite

  --------------------------------------------------------------------------------------------------
  Problem                 Typical answer          Kyanite
  ----------------------- ----------------------- --------------------------------------------------
  BusyBox is written in C Accept it               Replaced by [uutils
  and has a long history                          coreutils](https://github.com/uutils/coreutils),
  of memory-safety bugs                           written in Rust

  Alpine packages are     Switch to Debian        Kyanite's base is small, while `kpkg` installs
  built against musl,                             Debian packages linked against glibc
  which limits                                    
  compatibility with some                         
  software                                        

  Debian rootfs images    Accept the size         Kyanite's rootfs is about 30 MB uncompressed and
  are relatively large                            about 8 MB as `tar.xz`
  --------------------------------------------------------------------------------------------------

Kyanite uses **musl for the preinstalled Rust components** (init,
package manager, shell and coreutils), which are statically linked. The
distribution also includes a glibc runtime so Debian packages installed
with `kpkg` can run.

## What is inside

  -------------------------------------------------------------------------------------
  Component                                         Role
  ------------------------------------------------- -----------------------------------
  `kyanite-init`                                    PID 1 (`/sbin/init`); starts the
                                                    login shell, manages the terminal
                                                    and reaps orphaned processes

  [Brush](https://github.com/reubeno/brush)         Interactive shell at `/bin/brush`

  [uutils                                           Rust implementation of common
  coreutils](https://github.com/uutils/coreutils)   utilities, provided through a
                                                    multicall binary

  `kpkg`                                            Rust package manager with an
                                                    apk-style interface and an
                                                    apt-compatible mode

  Debian archive keyring                            Verifies Debian repository metadata

  glibc runtime                                     Runs dynamically linked Debian
                                                    packages

  `vim-tiny`                                        Minimal `vi` implementation;
                                                    currently the principal C-based
                                                    interactive utility
  -------------------------------------------------------------------------------------

`/bin/sh` is a symlink to `/sbin/init`. Launchers that invoke `/bin/sh`
therefore enter Kyanite's init, which starts Brush.

## Supported architectures

-   `aarch64` (arm64)
-   `x86_64` (amd64)

## Getting Kyanite

### Android

Kyanite is also available as an Android application, providing a Linux
environment with an integrated terminal and system-management features.

[**Download Kyanite for Android
(.apk)**](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
· [Source code and
documentation](https://github.com/StringManolo/KyaniteAndroid)

### Root filesystem

Release archives are published on the [Releases
page](https://github.com/StringManolo/Kyanite/releases) for every
`vX.Y.Z` tag. Each release contains, per architecture:

-   `kyanite-rootfs-<arch>.tar.xz`
-   `kyanite-rootfs-<arch>.tar.xz.sha256`
-   `kyanite-rootfs-<arch>.tar.xz.sha256.sig` --- Ed25519 signature of
    the `.sha256` file

Verify the checksum before using the archive:

``` sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

To verify the signature as well, see [Verifying
releases](SECURITY.md#verifying-releases).

Every push to `main` also produces the archive and checksum as workflow
artifacts. These development artifacts are not signed.

See the [Installation Guide](docs/installation.md) for details.

## Usage

Kyanite is a root filesystem, not a bootable image. It currently runs in
userspace through `proot`.

It has been tested with
[DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android).
Compatibility with `proot-distro`, QEMU, x86 hosts and udocker has not
yet been verified.

See the [Usage Guide](docs/usage.md).

## Package management

`kpkg` installs packages from Debian repositories using a lightweight
apk-style interface. It also provides `apt`, `apt-get` and `apt-cache`
compatibility modes.

See the [kpkg documentation](docs/kpkg.md) and [Package hooks
guide](docs/package-hooks.md).

## Security

Security is a primary project goal. The threat model, implemented
protections, known gaps and vulnerability-reporting instructions are
documented in [SECURITY.md](SECURITY.md).

Kyanite is experimental and should not be assumed to provide isolation
equivalent to a virtual machine. In particular, PRoot is not a security
boundary.

## Known limitations

Some important limitations:

-   `kpkg` does not execute Debian maintainer scripts. Packages that
    need setup rely on Kyanite hooks or manual configuration.
-   `Recommends` dependencies are not installed automatically.
-   Preinstalled base components are updated through Kyanite releases,
    not by `kpkg upgrade`.
-   The glibc runtime and `vim-tiny` are currently taken from the build
    runner's Ubuntu environment, while packages are installed from
    Debian stable.
-   Kyanite has no kernel yet and runs as a root filesystem under PRoot.
-   The `x86_64` image is currently roughly twice the size of the
    `aarch64` image; the cause is under investigation.

See [Known limitations](docs/limitations.md) for details.

## Roadmap

Planned work, in rough order:

-   Broader hook coverage for common Debian packages
-   Automatic base rebuilds when Debian publishes security updates for
    bundled components
-   Replace the remaining GNU `env` and `vim-tiny` with Rust
    implementations
-   Build the glibc runtime from Debian stable packages
-   Audit every file in the rootfs
-   Improve logging in init and preconfigured scripts
-   Test QEMU, x86 hosts, udocker and `proot-distro`
-   Add a kernel and the remaining pieces of a complete distribution

## Building from source

The build is defined in
[`.github/workflows/build-rootfs.yml`](.github/workflows/build-rootfs.yml).
It targets `aarch64-unknown-linux-musl` and `x86_64-unknown-linux-musl`.

See the [Development Guide](docs/development.md).

## Releasing

Pushing a tag matching `vX.Y.Z` builds both architectures, signs the
checksums and publishes a GitHub release. Other tag formats are ignored
by the base-update mechanism.

See the [Release Guide](docs/releases.md).

## Third-party components and licenses

Kyanite bundles software from other projects, each distributed under its
own license. Refer to the upstream projects and package copyright files
for authoritative terms.

  ------------------------------------------------------------------------------------------------------------------------
  Component                  Upstream                                                              License
  -------------------------- --------------------------------------------------------------------- -----------------------
  Brush                      [reubeno/brush](https://github.com/reubeno/brush)                     MIT

  uutils coreutils           [uutils/coreutils](https://github.com/uutils/coreutils)               MIT

  glibc (`libc6`)            [Debian](https://packages.debian.org/stable/libc6)                    LGPL-2.1-or-later

  `libgcc-s1`                [Debian](https://packages.debian.org/stable/libgcc-s1)                GPL-3.0-or-later with
                                                                                                   GCC Runtime Library
                                                                                                   Exception

  GNU `env`                  [Debian coreutils](https://packages.debian.org/stable/coreutils)      GPL-3.0-or-later

  `vim-tiny`, `vim-common`   [Vim](https://www.vim.org/) /                                         Vim license
                             [Debian](https://packages.debian.org/stable/vim-tiny)                 

  `libacl1`                  [Debian](https://packages.debian.org/stable/libacl1)                  LGPL-2.1-or-later

  `libselinux1`              [Debian](https://packages.debian.org/stable/libselinux1)              Public domain

  `libpcre2-8-0`             [Debian](https://packages.debian.org/stable/libpcre2-8-0)             BSD-3-Clause

  `libtinfo6`,               [Debian](https://packages.debian.org/stable/ncurses-base)             X11/MIT-style (ncurses)
  `ncurses-base`                                                                                   

  `debian-archive-keyring`   [Debian](https://packages.debian.org/stable/debian-archive-keyring)   See package copyright
                                                                                                   file
  ------------------------------------------------------------------------------------------------------------------------

Packages installed later with `kpkg` carry the licenses of their
respective Debian packages.
