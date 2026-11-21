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
- Authenticate everything that reaches the system: repository metadata (OpenPGP) and base-system updates (Ed25519).

## Threat model

Kyanite currently runs as a root filesystem under `proot` (for example in Termux). Be aware of what that implies:

- `proot` is a compatibility layer based on `ptrace`, not a security boundary. Kyanite does not add isolation between the rootfs and the host user's other data.
- The "root" user inside the rootfs is not real root on the host.
- Kyanite's protections focus on the integrity of what gets installed into the rootfs and on reducing memory-safety bugs in the base userland.

Running Rust code reduces whole classes of memory-safety bugs; it does not make a program free of vulnerabilities (logic errors, unsafe blocks and dependencies still matter).

## Protections in kpkg

Package index:

- `InRelease` is verified with OpenPGP against the Debian archive keyring shipped in the rootfs. Verification is fail-closed: with no keyring or an invalid signature, nothing from the mirror is used. Only the signed text is parsed, never the raw file.
- The signed `Release` must match the requested suite, must not be expired (`Valid-Until`) and must not be older than the last one accepted (replay/rollback protection).
- `Packages.xz` is checked against the size and hash listed in the signed release; decompression is limited so that a malicious index cannot exhaust memory or disk.
- Package names, versions, filenames and hashes read from the index are validated before use. Filenames must live under `pool/` and cannot contain `..`.
- For stable suites, `<codename>-security` is mandatory: if it cannot be fetched and verified, `kpkg update` fails instead of silently dropping security updates (`--no-security` opts out explicitly). The highest version across `stable`, `stable-updates` and `stable-security` is selected.
- The OpenPGP verifier is a small, purpose-built implementation (RSA keys and signatures with SHA-256/384/512, which is what Debian uses); the RSA operation itself comes from the `rsa` crate.

Package download:

- Each `.deb` is downloaded in streaming mode with a size cap (the size announced by the signed index), written to a temporary file and moved into the cache only after its SHA256 matches the index. A mismatch aborts the operation with a security alert.

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

Hooks:

- Package-specific setup is done by declarative hooks, never by running package scripts. A hook can only use a closed set of actions, all confined to the target root (paths are normalized and symlink targets checked; no setuid/setgid/sticky modes; directories are never removed). Hook directories are trusted like any other root-owned configuration.

Base system updates:

- `kpkg update` offers updates only for `vX.Y.Z` release tags; development builds are never updated.
- The release checksum is signed with Ed25519; `kpkg` verifies the signature with the keys in `/etc/kpkg/kyanite-release.pub` (several keys allow rotation), then the SHA256 of the archive, before anything is extracted. Downgrades are refused.
- The archive is unpacked into a staging directory on the same filesystem with the same restricted extraction used for packages, and every file is re-checked against the manifest before it replaces its target.
- Files are replaced atomically; the version marker is written last, so an interrupted update resumes on the next `kpkg upgrade`. Files you modified are never overwritten (the new version is saved as `<file>.kyanite-new`), and files owned by `kpkg` packages are skipped.

## Known limitations

These are known gaps, tracked in the project roadmap.

| Limitation | Impact | Status |
|---|---|---|
| `--mirror` accepts `http://` URLs | Metadata is signed, so a network attacker cannot forge packages, but can read what you download and can withhold updates (a stale index is detected only through the `Valid-Until` and date checks). Prefer HTTPS | Open |
| Key expiry and revocation are not checked | A compromised or retired Debian archive key stays trusted until the keyring shipped in the rootfs is updated | Open |
| The base-update signing key lives in a GitHub Actions secret | Someone who compromises the repository's workflows or secrets can sign a malicious release | Open |
| First installation trusts the downloaded rootfs | `/etc/kpkg/kyanite-release.pub` and the Debian keyring are part of the rootfs, so the first download is trusted via HTTPS and the `.sha256`/`.sig` files | By design |
| Base updates have no automatic rollback | They are idempotent and resumable, but a bad release must be fixed by a newer one | Open |
| Hooks cover only the packages that have one | Other packages that need setup may be left unconfigured; a package can also ship a file at a hook path, like any other file | Open |
| Maintainer scripts are not executed | Not a vulnerability by itself, but packages may be left unconfigured | By design |
| `env` comes from GNU coreutils (glibc build) rather than a Rust implementation | Small C component in the base system | Planned |
| `/etc/kpkg/assume-installed` is trusted | Anyone who can write the file can make `kpkg` skip dependencies | By design |
| Not all files of the rootfs have been audited yet | Unknown issues may remain | Planned |
| Preinstalled components (statically linked musl binaries, `libc6`, `vim-tiny`, ...) are not updated by `kpkg upgrade` | Fixes for bundled components require a new signed rootfs release | Mitigated (base updates), but a release must be published |

## Supported versions

Kyanite is pre-1.0. Security fixes are applied to the `main` branch and published in the next release; older releases are not patched.

## Verifying releases

Every release archive has a matching `.sha256` file and an Ed25519 signature of it:

```sh
sha256sum -c kyanite-rootfs-<arch>.tar.xz.sha256

# Verify the signature with the public key from rootfs-overlay/etc/kpkg/kyanite-release.pub
PUBHEX=<64-hex-character public key>
printf '302a300506032b6570032100%s' "$PUBHEX" | xxd -r -p > release.pub.der
openssl pkeyutl -verify -pubin -inkey release.pub.der -keyform DER -rawin \
  -in kyanite-rootfs-<arch>.tar.xz.sha256 -sigfile kyanite-rootfs-<arch>.tar.xz.sha256.sig
```

The `.sha256` guards against corruption; the signature authenticates it. `kpkg upgrade` performs both checks automatically before applying a base update.
