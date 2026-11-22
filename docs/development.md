# Development

This guide introduces the Kyanite source layout and build workflow. The GitHub Actions workflow is the authoritative source for the exact build commands and dependency versions.

## Build workflow

The build is defined in:

```text
.github/workflows/build-rootfs.yml
```

It targets:

- `aarch64-unknown-linux-musl`
- `x86_64-unknown-linux-musl`

At a high level, the workflow:

1. Computes the version with `git describe --tags --always`.
2. Builds Kyanite init and `kpkg` with Cargo.
3. Builds Brush and uutils coreutils for the musl targets.
4. Assembles the root filesystem.
5. Adds the runtime libraries and supporting packages.
6. Applies `rootfs-overlay/`.
7. Generates the base manifest.
8. Creates the compressed archive and SHA256 checksum.
9. Signs the checksum for release builds.

## Build prerequisites

The currently documented local prerequisites are:

- A Rust toolchain
- The required Rust musl targets
- `musl-tools`
- `gcc-aarch64-linux-gnu` for aarch64 cross-compilation

Check the workflow for any additional tools or packages needed by the current build.

## Root filesystem overlay

The `rootfs-overlay/` directory contains files copied into the generated root filesystem, including configuration and shell environment files.

When modifying the overlay, check file paths, permissions and how the changes interact with the base manifest and update mechanism.

## Testing changes

Use a disposable root filesystem when testing changes.

For package-manager work, test the relevant installation, upgrade, removal, repair, configuration and audit paths. For base-update work, test both unmodified files and files changed by the user.

## Version labels

Builds use `git describe --tags --always`. A commit after a release tag may be labelled with a value such as `v0.1.0-3-g1a2b3c4`. When no tag is available, the version may be the commit hash.

Only release tags matching `vX.Y.Z` are eligible for automatic base updates.

## Related documentation

- [Architecture](architecture.md)
- [Package manager](kpkg.md)
- [Package hooks](package-hooks.md)
- [Release process](releases.md)
- [Roadmap and investigations](roadmap.md)
