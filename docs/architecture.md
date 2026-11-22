# Architecture

Kyanite is a minimal Linux root filesystem built around a small set of Rust-based core components and a Debian-compatible package environment.

## System overview

The preinstalled Rust components are statically linked against musl:

- Kyanite init
- `kpkg`
- Brush
- uutils coreutils

The root filesystem also includes a glibc runtime for dynamically linked Debian packages.

## Startup sequence

The basic startup flow is:

1. A launcher enters the root filesystem, typically using PRoot.
2. The launcher invokes `/bin/sh`.
3. `/bin/sh` points to `/sbin/init`.
4. Kyanite init configures the terminal and starts Brush as a login shell.
5. Init reaps orphaned processes and exits when the shell exits.

Init runs as PID 1 inside the Kyanite environment.

## Main components

| Component | Responsibility |
| --- | --- |
| `kyanite-init` | PID 1, shell startup, terminal and job control, orphan reaping |
| Brush | Interactive shell |
| uutils coreutils | Common command-line utilities |
| `kpkg` | Package indexes, dependency resolution, package installation and base updates |
| glibc runtime | Runtime support for Debian packages |
| `vim-tiny` | Minimal vi-compatible editor |

## musl and glibc

Kyanite uses musl for its statically linked Rust tools. This keeps the core utilities independent of the glibc runtime.

The rootfs also includes glibc so packages from Debian repositories can run. This combines a compact base with access to Debian's package ecosystem.

## Package installation

`kpkg` downloads Debian packages, verifies repository metadata and package hashes, resolves dependencies and extracts packages into the target root.

Package setup is handled through declarative Kyanite hooks. See [Package hooks](package-hooks.md).

## Base updates

Kyanite base updates are separate from ordinary Debian package upgrades. Eligible GitHub releases contain the files and manifest needed to update the base while preserving locally modified files.

See [Releases](releases.md) and [kpkg base updates](kpkg.md#updating-the-kyanite-base).

## Runtime model

Kyanite is currently distributed as a root filesystem and uses the host kernel through its runtime environment. For platform and isolation considerations, see [Known limitations](limitations.md) and [SECURITY.md](../SECURITY.md).
