# Glossary

Short explanations of technical words used in the Kyanite documentation.

| Term | Meaning |
| --- | --- |
| **Linux** | A family of free operating systems. Kyanite is a small Linux system. |
| **Kernel** | The core of an operating system. It talks to the hardware. Kyanite does not include its own kernel yet; it uses the one of the device it runs on (the *host*). |
| **Root filesystem (rootfs)** | All the files that make up a Linux system: programs, settings and folders. Kyanite is distributed as a rootfs. |
| **Terminal** | The window where you type commands. |
| **Shell** | The program inside the terminal that reads your commands and runs them. Kyanite uses Brush. |
| **Command** | An instruction you type in the shell, such as `ls` to list files. |
| **Package** | A program (or a library) bundled for easy installation. |
| **Package manager** | A tool that installs, updates and removes packages. In Kyanite it is `kpkg`. |
| **Repository** | An online collection of packages. Kyanite uses Debian's repositories. |
| **Debian** | A well-known Linux distribution with a very large collection of packages. |
| **Init** | The first program started by the system. It starts the shell and keeps things running. |
| **PRoot** | A tool that runs a Linux system inside a folder, without special permissions. Kyanite currently runs this way. |
| **Rust** | A programming language designed to prevent many memory-related bugs, which are a common source of security problems. |
| **musl and glibc** | Two C libraries that programs need in order to run. Kyanite's own tools use musl; Debian packages use glibc. Both are included. |
| **Architecture** | The type of processor. `aarch64` (ARM64) is common in phones; `x86_64` (AMD64) in most PCs. |
| **Checksum (SHA256)** | A short fingerprint of a file. If the file changes, the fingerprint changes, so you can detect corrupted downloads. |
| **Signature (Ed25519)** | A way to prove that a file was published by its real author. |
| **APK** | The file format of Android apps. |
| **MOTD** | "Message of the day": the text shown when you log in. |
| **Hook** | A small Kyanite rule that sets up a package after installing it. See [Package hooks](package-hooks.md). |
| **Base** | The preinstalled part of Kyanite, updated through Kyanite releases rather than as ordinary packages. |

Back to the [documentation index](../README.md#documentation).
