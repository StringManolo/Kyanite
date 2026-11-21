# Known limitations

Kyanite is experimental and is not yet feature-complete. This document
collects the limitations currently described by the project.

## Runtime and platform

-   Kyanite does not include a kernel. It is a root filesystem currently
    run in userspace through PRoot.
-   The rootfs has been tested through DebianLiveCD4Android.
-   `proot-distro`, QEMU, x86 hosts and udocker have not yet been
    verified.
-   On Android 8 and later, some files under `/proc` may not be readable
    by applications. Android launchers may need to bind replacement
    files.

## Package manager

-   `kpkg` does not execute Debian maintainer scripts (`preinst`,
    `postinst`, `prerm`, `postrm`). Packages that require these actions
    need a Kyanite hook or manual setup.
-   `Recommends` and `Suggests` are not installed automatically. A hook
    may add a dependency when it is required for a supported use case.
-   Preinstalled components such as `libc6` and `vim-tiny` are updated
    through a Kyanite base release, not through ordinary `kpkg upgrade`.
-   The base glibc runtime and `vim-tiny` currently come from the build
    runner's Ubuntu distribution, while `kpkg` installs packages from
    Debian stable. Their glibc versions may differ.

## Certificates

The native `update-ca-certificates` implementation generates the
`/etc/ssl/certs/ca-certificates.crt` bundle. It does not create the
`<hash>.0` links used by programs that rely on `CApath`.

## Base image

-   The `x86_64` rootfs is currently roughly twice the size of the
    `aarch64` rootfs. The cause is under investigation.
-   The remaining GNU `env` and `vim-tiny` components are planned for
    replacement with Rust implementations.

## Security considerations

PRoot is not a virtual machine and should not be treated as a security
boundary. The rootfs shares the host kernel and depends on the host
launcher and its configuration.

Review [SECURITY.md](../SECURITY.md) before using Kyanite for
security-sensitive workloads. Do not assume that the experimental status
or Rust implementation of base components makes the complete system
vulnerability-free.
