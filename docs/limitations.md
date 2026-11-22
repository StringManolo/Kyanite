# Known limitations and compatibility notes

This page collects current implementation details that may affect compatibility or expected behavior. It is intended as a technical reference.

## Runtime and platform

- Kyanite is distributed as a root filesystem and does not currently include its own kernel.
- The current runtime uses PRoot and shares the host kernel.
- The rootfs has been tested through [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android).
- `proot-distro`, QEMU and udocker still need further testing.
- On Android 8 and later, some files under `/proc` may not be readable by applications. Launchers may need to bind replacement files.

## Package manager

- `kpkg` does not execute Debian maintainer scripts (`preinst`, `postinst`, `prerm`, `postrm`). Packages that require those actions need a Kyanite hook or manual setup.
- `Recommends` and `Suggests` are not installed automatically.
- Preinstalled base components are updated through Kyanite releases rather than ordinary package upgrades.
- The build currently obtains some runtime components from the Ubuntu build environment while `kpkg` installs packages from Debian stable. Their versions may differ.

## Certificates

The native `update-ca-certificates` implementation generates `/etc/ssl/certs/ca-certificates.crt`. It does not create the `<hash>.0` links used by applications that rely on `CApath`.

## Shell and init behavior

`/bin/sh` points to the Kyanite init executable. This is part of the current startup design. A package script that invokes `/bin/sh` may therefore interact with init rather than a conventional shell when run outside the expected startup path. The MOTD scripts are explicitly run with `/bin/brush`.

Shell compatibility and the behavior of packages that assume a conventional `/bin/sh` are areas for further testing.

## Security and isolation

PRoot is not a virtual machine or an independent kernel security boundary. The Kyanite environment shares the host kernel and depends on the host launcher and its configuration.

See [SECURITY.md](../SECURITY.md) for the threat model, protections and security reporting instructions.
