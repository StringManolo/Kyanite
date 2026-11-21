# Development

This guide summarizes the current build workflow and points to the
authoritative build configuration.

## Source and build workflow

The build is defined in:

``` text
.github/workflows/build-rootfs.yml
```

The workflow builds two musl targets:

-   `aarch64-unknown-linux-musl`
-   `x86_64-unknown-linux-musl`

At a high level, the workflow:

1.  Computes the build version with `git describe --tags --always`.
2.  Builds Kyanite init and `kpkg` with Cargo.
3.  Builds Brush and uutils coreutils for the musl targets.
4.  Assembles the root filesystem.
5.  Adds the glibc runtime, `env`, `vi`, Debian archive keyring and
    required support files.
6.  Applies `rootfs-overlay/`.
7.  Generates the base manifest.
8.  Creates the compressed rootfs archive and SHA256 checksum.
9.  On release tags, signs the checksum.

The workflow file is the source of truth for exact commands, versions
and build dependencies.

## Build prerequisites

The documented local prerequisites are:

-   Rust toolchain
-   Rust musl targets for the architectures being built
-   `musl-tools`
-   `gcc-aarch64-linux-gnu` for aarch64 cross-compilation

Additional tools may be required by the workflow or by the host
distribution. Check the workflow before setting up a new build
environment.

## Rootfs overlay

The `rootfs-overlay/` directory contains files installed into the
generated root filesystem, including shell configuration and Kyanite
configuration.

When changing files in the overlay, verify that:

-   paths are correct relative to the root filesystem;
-   executable permissions are preserved where required;
-   scripts and configuration match the programs included in the base;
-   generated manifests and update behavior remain correct.

## Testing changes

Test changes in a disposable rootfs before relying on them. For
package-manager changes, test at least the relevant install, upgrade,
removal, repair and audit paths.

For base updates, test both unmodified files and locally modified files
to confirm that the update mechanism preserves user changes as intended.

## Version labels

Builds use `git describe --tags --always`. A build from a commit after a
release tag may be labelled with a value such as `v0.1.0-3-g1a2b3c4`.
When no tag is available, the version may be the commit hash.

Development builds are not eligible for automatic base updates. Only
release tags matching `vX.Y.Z` are considered.

## Further reading

-   [Architecture](architecture.md)
-   [Package manager](kpkg.md)
-   [Package hooks](package-hooks.md)
-   [Release process](releases.md)
