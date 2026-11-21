# Installation

Kyanite is distributed as a root filesystem archive. It is not currently
a bootable disk image and does not include its own kernel. The supported
runtime described here is PRoot.

## Download a release

Open the [Kyanite Releases
page](https://github.com/StringManolo/Kyanite/releases) and download the
archive for your architecture:

-   `kyanite-rootfs-aarch64.tar.xz` for ARM64
-   `kyanite-rootfs-x86_64.tar.xz` for AMD64

Download the corresponding `.sha256` file as well. Release checksums are
signed; see [Verifying releases](../SECURITY.md#verifying-releases) for
signature verification.

## Verify the archive

From the directory containing the archive and checksum:

``` sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Use the checksum filename matching the archive you downloaded. Do not
continue if verification fails.

For signature verification, follow the instructions in
[SECURITY.md](../SECURITY.md#verifying-releases).

## Extract the root filesystem

Create a directory and extract the archive into it:

``` sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

Replace the archive name with the one matching your architecture.

## Start with PRoot

A simplified launch command is:

``` sh
proot --kill-on-exit --link2symlink -0 -r kyanite \
  -b /dev -b /proc -b /sys \
  -w /root \
  /usr/bin/env -i \
    HOME=/root USER=root LOGNAME=root \
    TERM=xterm-256color COLORTERM=truecolor LANG=C.UTF-8 \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    /bin/sh -l
```

A real launcher may need additional bindings, depending on the host. On
Android, some files under `/proc` may not be readable by applications; a
launcher may need to bind replacement files. The Android application
provides its own integration and launch experience.

## What happens at startup

`/bin/sh` is a symlink to `/sbin/init`. When a launcher starts
`/bin/sh`, Kyanite's init starts Brush as an interactive login shell.
Init manages the terminal and exits when the shell exits.

`/usr/bin/env` is included because launchers use it to create a clean
environment.

## Supported and unverified environments

Kyanite has been tested through
[DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android).

The following environments are expected to be possible but have not yet
been verified:

-   `proot-distro` in Termux
-   QEMU
-   x86 hosts
-   udocker

Do not interpret an unverified environment as officially supported.

## Android application

An Android application is also available:

-   [Download the latest
    APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
-   [Android application source and
    documentation](https://github.com/StringManolo/KyaniteAndroid)
