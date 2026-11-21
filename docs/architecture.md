# Architecture

Kyanite is a minimal Linux root filesystem designed around a small set
of preinstalled Rust components and a Debian-compatible package
environment.

It is currently a userspace root filesystem, not a complete bootable
distribution: it does not ship its own kernel and is run through PRoot.

## Design overview

The base is intentionally small. Its interactive components are
primarily Rust programs, while a glibc runtime is included so that
dynamically linked Debian packages can run.

The preinstalled Rust components are statically linked against musl:

-   Kyanite init
-   `kpkg`
-   Brush
-   uutils coreutils

The rootfs also includes glibc and supporting runtime libraries for
Debian packages.

## Startup

The main startup path is:

1.  A host launcher enters the rootfs, typically through PRoot.
2.  The launcher invokes `/bin/sh`.
3.  `/bin/sh` is a symlink to `/sbin/init`.
4.  Kyanite init configures the terminal and starts Brush as a login
    shell.
5.  Init reaps orphaned processes and exits when the shell exits.

Init is PID 1 inside the Kyanite environment.

## Components

  -----------------------------------------------------------------------
  Component                           Responsibility
  ----------------------------------- -----------------------------------
  `kyanite-init`                      PID 1, login startup, terminal/job
                                      control and orphan reaping

  Brush                               Interactive shell

  uutils coreutils                    Common command-line utilities
                                      through a multicall binary

  `kpkg`                              Debian repository metadata,
                                      dependency resolution, package
                                      installation and base updates

  glibc runtime                       Runtime compatibility for
                                      dynamically linked Debian packages

  `vim-tiny`                          Minimal vi-compatible editor
  -----------------------------------------------------------------------

## musl and glibc

Kyanite uses musl for its statically linked preinstalled Rust tools.
This keeps the base components independent of the glibc runtime.

The rootfs also contains glibc for packages installed from Debian
repositories. This is a deliberate split: a small base with Rust
components, alongside compatibility with Debian's glibc-linked package
ecosystem.

At present, the glibc runtime and `vim-tiny` are obtained from the build
runner's Ubuntu distribution, while `kpkg` installs packages from Debian
stable. Their glibc versions may therefore differ.

## Package installation model

`kpkg` downloads Debian packages, verifies repository metadata and
package hashes, resolves dependencies, and extracts packages into the
target root.

It does not execute package maintainer scripts. Kyanite uses declarative
package hooks for supported setup tasks instead. See [Package
hooks](package-hooks.md).

## Rootfs updates

The Kyanite base is updated separately from ordinary Debian packages.
Eligible `vX.Y.Z` GitHub releases contain a manifest and update data.
The update process verifies the release, stages it, and applies changes
file by file while preserving locally modified files.

See [Release process](releases.md) and [kpkg base
updates](kpkg.md#base-updates).

## Security boundary

PRoot provides a userspace environment, not the isolation boundary of a
virtual machine or a separate kernel. Do not treat a Kyanite rootfs
under PRoot as a security boundary against the host or other processes
with equivalent permissions.

See [SECURITY.md](../SECURITY.md) for the project's threat model and
security notes.
