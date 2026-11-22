# Kyanite

Kyanite is a lightweight Linux environment built around memory-safe Rust components. It combines a small base system with access to Debian's package ecosystem through `kpkg`.

The goal is to make a Linux environment that is **small, comfortable to use and easy to manage**.

![Kyanite login screen](docs/screenshot.png)

[![Build Kyanite Rootfs](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml/badge.svg)](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml)

## Why Kyanite?

Kyanite brings together a minimal base system and familiar Linux tools:

- **Rust-based core:** the init system, package manager, shell and core utilities are written in Rust.
- **Debian packages:** install software from Debian repositories using `kpkg`.
- **Small base:** the root filesystem is designed to stay compact.
- **Familiar environment:** use common commands and an interactive shell.
- **Android app:** run Kyanite through a dedicated Android application.

## What's included?

| Component | Description |
| --- | --- |
| `kyanite-init` | Starts and manages the interactive shell |
| [Brush](https://github.com/reubeno/brush) | Interactive shell |
| [uutils coreutils](https://github.com/uutils/coreutils) | Common command-line utilities implemented in Rust |
| `kpkg` | Package manager for Debian repositories |
| glibc runtime | Runtime for dynamically linked Debian packages |
| `vi` | Text editor |

For a closer look at how the components fit together, see the [architecture guide](docs/architecture.md).

## Getting Kyanite

### Android

Get Kyanite as an Android application, with an integrated Linux environment and terminal.

[**Download Kyanite for Android (.apk)**](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk) · [Android source code and documentation](https://github.com/StringManolo/KyaniteAndroid)

### Linux root filesystem

Download a root filesystem archive from the [Kyanite Releases page](https://github.com/StringManolo/Kyanite/releases).

Available architectures:

- `aarch64` (ARM64)
- `x86_64` (AMD64)

Each release includes an archive, its SHA256 checksum and an Ed25519 signature of that checksum.

See the [installation guide](docs/installation.md) for download, verification and setup instructions.

## Getting started

Once Kyanite is running, use `kpkg` to install software:

```sh
kpkg update
kpkg add curl
```

Explore the [usage guide](docs/usage.md) and [kpkg documentation](docs/kpkg.md) for more examples.

## Documentation

| Guide | What it covers |
| --- | --- |
| [Installation](docs/installation.md) | Downloading, verifying and starting Kyanite |
| [Usage](docs/usage.md) | Shell environment and customization |
| [Package manager (`kpkg`)](docs/kpkg.md) | Commands, options and package management |
| [Package hooks](docs/package-hooks.md) | How Kyanite configures packages |
| [Architecture](docs/architecture.md) | Components and system design |
| [Development](docs/development.md) | Building and working on the source |
| [Releases](docs/releases.md) | Versioning and release process |
| [Known limitations](docs/limitations.md) | Current compatibility notes and technical limitations |
| [Roadmap](docs/roadmap.md) | Areas being explored and possible future improvements |

## Security

Security is an important design goal. Read [SECURITY.md](SECURITY.md) for the threat model, implemented protections, known security gaps and vulnerability reporting instructions.

## Contributing

Kyanite is developed in the open. The [development guide](docs/development.md) describes the build workflow and gives pointers for working with the source code.
