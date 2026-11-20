# Security Policy

## Scope

This document covers the components developed in this repository (`kyanite-init`, `kpkg` and the rootfs build pipeline) and how Kyanite assembles third-party components. Vulnerabilities in upstream projects (Brush, uutils coreutils, glibc, Debian packages) should be reported to those projects; please tell us as well if Kyanite ships an affected version.

## Reporting a vulnerability

Please do **not** open a public issue for security problems.

Use GitHub's private reporting: go to the **Security** tab of the repository and choose **Report a vulnerability**. Include:

- A description of the issue and its impact
- Steps to reproduce, or a proof of concept
- Affected version, architecture and environment (for example, proot on Android)

## Security goals

- Reduce the attack surface of the base system by using memory-safe Rust implementations for init, package manager, shell and core utilities.
- Keep the base system small so that it can be audited.
- Make package installation conservative: verified downloads, restricted extraction, no arbitrary code execution during installation.

## Threat model

Kyanite currently runs as a root filesystem under `proot` (for example in Termux). Be aware of what that implies:

- `proot` is a compatibility layer based on `ptrace`, not a security boundary. Kyanite does not add isolation between the rootfs and the host user's other data.
- The "root" user inside the rootfs is not real root on the host.
- Kyanite's protections focus on the integrity of what gets installed into the rootfs and on reducing memory-safety bugs in the base userland.

Running Rust code reduces whole classes of memory-safety bugs; it does not make a program free of vulnerabilities (logic errors, unsafe blocks and dependencies still matter).

## Protections in kpkg

Package index:

- `InRelease` is fetched with a size limit; `Packages.xz` is checked against the size and hash listed in the release index.
- Decompression is limited so that a malicious index cannot exhaust memory or disk.
- Package names, versions, filenames and hashes read from the index are validated before use. Filenames must live under `pool/` and cannot contain `..`.

Package download:

- Each `.deb` is downloaded in streaming mode with a size cap (the size announced by the index), written to a temporary file and moved into the cache only after its SHA256 matches the index. A mismatch aborts the operation with a security alert.

Extraction:

- Paths are normalized and must remain inside the target root; symlink resolution is checked so an existing link cannot redirect a write outside the root.
- Symlinks whose target would escape the root are rejected.
- Per-entry and per-package size limits protect against decompression bombs.
- setuid, setgid and sticky bits are never applied.
- Files are first extracted to temporary locations and committed afterwards with atomic renames.
- Files owned by another package or not owned by any package are not overwritten unless `--force` is given.

Operation:

- No maintainer scripts (`preinst`, `postinst`, `prerm`, `postrm`) are executed.
- A lock file prevents concurrent `kpkg` runs from corrupting the database.
- `kpkg audit` compares every installed file against the SHA256 recorded at install time and reports modified (`M`) or deleted (`D`) files.

## Known limitations

These are known gaps, tracked in the project roadmap.

| Limitation | Impact | Status |
|---|---|---|
| The OpenPGP signature of `InRelease` is not verified. `kpkg update` prints a warning about it | Trust rests on HTTPS and the hashes in the release index. A compromised mirror or a TLS-level attacker could serve a forged index and malicious packages | Planned |
| `--mirror` accepts `http://` URLs | Without signature verification, a plain HTTP mirror offers no integrity guarantee against a network attacker. Use HTTPS mirrors | Open |
| Maintainer scripts are not executed | Not a vulnerability by itself, but packages may be left unconfigured; when support is added, it will need its own security review | Planned |
| `env` comes from GNU coreutils (glibc build) rather than a Rust implementation | Small C component in the base system | Planned |
| `/etc/kpkg/assume-installed` is trusted | Anyone who can write the file can make `kpkg` skip dependencies | By design |
| Not all files of the rootfs have been audited yet | Unknown issues may remain | Planned |
| Preinstalled binaries are statically linked musl builds and are not updated by `kpkg` | Fixes for bundled components require a new rootfs release | Open |

## Supported versions

Kyanite is pre-1.0. Security fixes are applied to the `main` branch and published in the next release; older releases are not patched.

## Verifying releases

Every release archive has a matching `.sha256` file:

```sh
sha256sum -c kyanite-rootfs-<arch>.tar.xz.sha256
```

Note that this guards against corruption and mismatched downloads; the hash is published alongside the archive, so it is not a substitute for a signature.
