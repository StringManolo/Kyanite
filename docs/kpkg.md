# kpkg

`kpkg` is Kyanite's package manager. It provides an apk-style command-line interface while installing packages from Debian repositories. It also offers an apt-compatible mode.

## Quick start

```sh
kpkg update
kpkg search curl
kpkg add curl
kpkg info curl
```

## Commands

| Command | Description |
| --- | --- |
| `kpkg update` | Fetch and verify package indexes; check for a Kyanite base update |
| `kpkg search <term>` | Search package names and descriptions |
| `kpkg add <pkg>...` | Install packages and dependencies |
| `kpkg del <pkg>...` | Remove packages |
| `kpkg purge <pkg>...` | Remove packages, including modified configuration files |
| `kpkg autoremove` | Remove dependencies that are no longer needed |
| `kpkg upgrade` | Apply a pending Kyanite base update and upgrade installed packages |
| `kpkg fix <pkg>...` | Reinstall packages to repair them |
| `kpkg info [pkg]` | Show package information |
| `kpkg list [pattern]` | List available packages |
| `kpkg policy <pkg>...` | Show installed and candidate versions |
| `kpkg configure [pkg]...` | Re-run package setup hooks |
| `kpkg audit` | Verify installed files against recorded SHA256 sums |
| `kpkg cache clean` | Remove cached package archives |
| `kpkg cache size` | Show cache usage |
| `kpkg base` | Show the Kyanite base version and pending update information |

`kpkg info` supports additional file and dependency queries:

- `-L`: list files
- `-W`: show the owner of a path
- `-R`: show dependencies
- `-r`: show reverse dependencies

For the exact syntax available in your build, run:

```sh
kpkg --help
```

## APT-compatible mode

Kyanite provides `apt`, `apt-get` and `apt-cache` as symlinks to `kpkg`.

When invoked through one of these names, or with `KPKG_SYNTAX=apt`, the program displays apt-style help and asks for confirmation before changes. Use `-y` to skip confirmation.

```sh
apt update
apt install curl
apt remove curl
apt autoremove
apt list --installed
apt show curl
```

A shell alias such as `alias apt=kpkg` does not change the executable name. To activate apt mode with an alias, use:

```sh
alias apt='KPKG_SYNTAX=apt kpkg'
```

## Options

| Option | Description |
| --- | --- |
| `-s`, `--simulate` | Show planned actions without changing anything |
| `-f`, `--force` | Overwrite unowned or foreign files and ignore reverse dependencies |
| `-y`, `--yes` | Skip confirmation prompts in apt mode |
| `--root <dir>` | Operate on another root; defaults to `/` |
| `--arch <arch>` | Set the Debian architecture |
| `--mirror <url>` | Set the Debian repository mirror |
| `--security-mirror <url>` | Set the Debian security repository mirror |
| `--suite <name>` | Set the Debian suite; defaults to `stable` |
| `--keyring <file>` | Use a specified OpenPGP keyring |
| `--no-security` | Disable the `<suite>-security` repository |

## Repositories and package selection

The package index is built from Debian `stable`, `stable-updates` and `stable-security`. For each package, `kpkg` selects the highest available version.

Repository metadata is verified with OpenPGP. Package downloads are checked for size and SHA256 against the signed index before being written to their final location.

Dependency resolution supports:

- `Pre-Depends`
- `Depends`, including alternatives (`a | b`) and version constraints
- Virtual packages through `Provides`
- `Conflicts` and `Breaks`

Version comparison follows the dpkg algorithm.

## Package setup

Kyanite uses declarative hooks for package setup. Read the [package hooks guide](package-hooks.md) for the hook locations, supported actions and development notes.

## Package state and cache

Package-manager state is stored in `/var/lib/kpkg`. Downloaded `.deb` files are cached in `/var/cache/kpkg/pkgs`.

Packages included in the base root filesystem are listed in `/etc/kpkg/assume-installed` so that `kpkg` does not install them again.

## Updating the Kyanite base

`kpkg update` checks for the latest eligible Kyanite GitHub release. `kpkg upgrade` applies a pending base update as well as upgrading installed packages.

Only release tags matching `vX.Y.Z` are considered. Development builds identified by a commit hash or `git describe` output are not eligible for automatic base updates.

The updater verifies the release signature and SHA256 checksum, unpacks into a staging directory and applies files using a manifest. Locally modified files are preserved; the incoming version is saved alongside them with a `.kyanite-new` suffix.

The release `CHANGELOG` documents operations needed for changes that cannot be represented as ordinary file updates.
