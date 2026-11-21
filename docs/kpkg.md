# kpkg

`kpkg` is Kyanite's lightweight package manager. It provides an
apk-style command-line interface while installing packages from Debian
repositories. It also supports an apt-compatible mode.

## Quick start

``` sh
kpkg update
kpkg search curl
kpkg add curl
kpkg info curl
kpkg del curl
```

`kpkg update` fetches and verifies repository indexes. `kpkg add`
installs a package and its dependencies.

## Commands

  -----------------------------------------------------------------------
  Command                             Purpose
  ----------------------------------- -----------------------------------
  `kpkg update`                       Fetch and verify package indexes;
                                      check for a Kyanite base release

  `kpkg search <term>`                Search package names and
                                      descriptions

  `kpkg add <pkg>...`                 Install packages and dependencies

  `kpkg del <pkg>...`                 Remove packages

  `kpkg purge <pkg>...`               Remove packages, including modified
                                      configuration files

  `kpkg autoremove`                   Remove dependencies that are no
                                      longer needed

  `kpkg upgrade`                      Update the Kyanite base, if an
                                      update is pending, and upgrade
                                      installed packages

  `kpkg fix <pkg>...`                 Reinstall packages to repair them

  `kpkg info [pkg]`                   Show package information; supports
                                      `-L`, `-W`, `-R` and `-r` queries

  `kpkg list [pattern]`               List available packages; `-I` lists
                                      installed and `-u` lists upgradable
                                      packages

  `kpkg policy <pkg>...`              Show installed and candidate
                                      versions

  `kpkg configure [pkg]...`           Re-run post-install hooks for
                                      installed packages

  `kpkg audit`                        Verify installed files against
                                      recorded SHA256 sums

  `kpkg cache clean\|size`            Clean or inspect the downloaded
                                      `.deb` cache

  `kpkg base`                         Show the Kyanite base version and
                                      any pending update
  -----------------------------------------------------------------------

Run `kpkg --help` for the command syntax supported by the installed
version.

## APT-compatible mode

Kyanite provides `apt`, `apt-get` and `apt-cache` as symlinks to `kpkg`.
When invoked through one of these names, or with `KPKG_SYNTAX=apt`, the
program displays apt-style help and asks for confirmation before
changes. Use `-y` to skip confirmation.

Examples:

``` sh
apt update
apt install curl
apt remove curl
apt autoremove
apt list --installed
apt show curl
```

A shell alias such as `alias apt=kpkg` does not change the executable
name, so it does not activate apt mode. Use the symlinks or:

``` sh
alias apt='KPKG_SYNTAX=apt kpkg'
```

## Options

  ------------------------------------------------------------------------------
  Option                              Description
  ----------------------------------- ------------------------------------------
  `-s`, `--simulate`                  Show planned actions without changing the
                                      system

  `-f`, `--force`                     Overwrite unowned or foreign files and
                                      ignore reverse dependencies

  `-y`, `--yes`                       Skip confirmation prompts in apt mode

  `--root <dir>`                      Operate on another root; defaults to `/`

  `--arch <arch>`                     Select Debian architecture; defaults to
                                      the build architecture

  `--mirror <url>`                    Set the repository mirror; defaults to
                                      `https://deb.debian.org/debian`

  `--security-mirror <url>`           Set the security mirror; defaults to
                                      `https://deb.debian.org/debian-security`

  `--suite <name>`                    Set the Debian suite; defaults to `stable`

  `--keyring <file>`                  Use a specified OpenPGP keyring

  `--no-security`                     Disable the `<suite>-security` repository;
                                      not recommended
  ------------------------------------------------------------------------------

## Repository and package behavior

The package index is built from Debian `stable`, `stable-updates` and
`stable-security`. For each package, `kpkg` selects the highest
available version, allowing security updates to be installed when Debian
publishes them rather than waiting for a point release.

Repository metadata is verified with OpenPGP. Package downloads are
checked for size and SHA256 against the signed index before being
written to their final location.

Dependency resolution supports:

-   `Pre-Depends`
-   `Depends`, including alternatives (`a | b`) and version constraints
-   Virtual packages through `Provides`
-   `Conflicts` and `Breaks`

`Recommends` and `Suggests` are not installed automatically. Version
comparison follows the dpkg algorithm.

## Package setup and hooks

`kpkg` deliberately does not execute Debian maintainer scripts such as
`preinst`, `postinst`, `prerm` or `postrm`. Packages that need setup use
declarative Kyanite hooks or may require manual configuration.

See the [Package hooks guide](package-hooks.md).

## Base packages and state

Packages shipped in the rootfs are declared in
`/etc/kpkg/assume-installed` so that `kpkg` does not try to install them
again. The current list includes `libc6`, `libgcc-s1` and `coreutils`.

Package-manager state is stored in `/var/lib/kpkg`. Downloaded `.deb`
files are cached in `/var/cache/kpkg/pkgs`.

## Base updates

`kpkg update` checks for the latest eligible Kyanite GitHub release.
`kpkg upgrade` applies a pending base update as well as upgrading
installed packages.

Only release tags matching `vX.Y.Z` are considered. Development builds
identified by a commit hash or `git describe` output do not update
themselves. The base update process verifies the Ed25519 signature and
SHA256 checksum, stages the update, and applies files from a manifest.
Modified files are preserved; the incoming version is saved alongside
them with a `.kyanite-new` suffix.

The release `CHANGELOG` describes operations required for changes that
cannot be represented as ordinary file updates.

## Cache

Use `kpkg cache size` to inspect the cache and `kpkg cache clean` to
remove cached package archives.
