# Package hooks

Kyanite uses declarative hooks for package setup instead of executing Debian package maintainer scripts.

Hooks describe a limited set of operations that `kpkg` can perform inside the target root filesystem. They can also add or ignore package dependencies.

## Hook locations

System-provided hooks are stored in:

```text
/usr/share/kpkg/hooks/<package>.hook
```

Local hook overrides are stored in:

```text
/etc/kpkg/hooks.d/
```

## Supported actions

The hook engine currently provides a closed set of actions:

- `mkdir`
- `symlink`
- `link-default`
- `copy-default`
- `remove`
- `chmod`
- `builtin`

Hooks can also add or ignore dependencies.

These actions are declarative; a hook is not an arbitrary shell script.

## Example: CA certificates

The `ca-certificates` package uses native Kyanite functionality to rebuild:

```text
/etc/ssl/certs/ca-certificates.crt
```

The `curl` hook adds a dependency on `ca-certificates`, so the certificate bundle is available when curl is installed.

## Re-running hooks

Run the setup hooks for installed packages with:

```sh
kpkg configure
```

To configure specific packages:

```sh
kpkg configure curl ca-certificates
```

## Developing a hook

When adding a hook, first identify the setup performed by the package's Debian maintainer scripts. Decide which parts are needed in Kyanite, then express those operations using supported hook actions or an existing native `builtin`.

A useful test cycle includes:

1. Installing the package into a clean rootfs.
2. Checking the resulting files and configuration.
3. Running `kpkg configure` again.
4. Removing the package and checking cleanup behavior.
5. Testing any dependencies introduced by the hook.

Consult the hook examples and implementation in the source tree for the exact grammar. This guide lists the action families but does not define parser syntax.
