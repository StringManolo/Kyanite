# Installation

Kyanite is distributed as a Linux root filesystem archive. It can also be used through the dedicated Android application.

## Android

The simplest way to get started on an Android device is to install the Kyanite application.

- [Download the latest Kyanite APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
- [KyaniteAndroid source code and documentation](https://github.com/StringManolo/KyaniteAndroid)

The Android application provides its own setup and launch experience.

## Download a Linux root filesystem

Open the [Kyanite Releases page](https://github.com/StringManolo/Kyanite/releases) and download the archive for your architecture:

| Architecture | Archive |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |

Download the matching `.sha256` and `.sha256.sig` files too.

## Verify the download

Check the SHA256 checksum from the directory containing the downloaded files:

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Use the checksum filename matching your downloaded archive.

To verify the Ed25519 signature as well, follow [Verifying releases](../SECURITY.md#verifying-releases).

## Extract the root filesystem

Create a directory and extract the archive:

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

Replace the archive name with the one matching your architecture.

## Start with PRoot

A basic launch command looks like this:

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

A launcher may need additional bindings depending on the host environment. On Android, some files under `/proc` may need special handling.

## Startup behavior

`/bin/sh` is a symlink to `/sbin/init`. When the launcher starts `/bin/sh`, Kyanite init starts Brush as an interactive login shell.

For more about the shell and login environment, see the [usage guide](usage.md).
