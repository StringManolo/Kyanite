# Package hooks

Kyanite does not run Debian package maintainer scripts. Instead, package
setup that is required for supported packages is described by
declarative hooks.

This approach avoids executing arbitrary package-provided shell scripts
during installation. It also means that packages requiring
maintainer-script behavior may need a Kyanite hook or manual setup.

## Hook locations

System-provided hooks are stored in:

``` text
/usr/share/kpkg/hooks/<package>.hook
```

Local overrides are stored in:

``` text
/etc/kpkg/hooks.d/
```

The exact hook filename and syntax should follow the examples shipped
with the Kyanite source and root filesystem.

## Available action families

The hook engine uses a closed set of actions confined to the target root
filesystem:

-   `mkdir`
-   `symlink`
-   `link-default`
-   `copy-default`
-   `remove`
-   `chmod`
-   `builtin`

Hooks can also add or ignore dependencies.

The action set is intentionally limited. A hook is not a shell script
and cannot run arbitrary commands.

## Example: CA certificates

The `ca-certificates` package uses native Kyanite functionality to
rebuild:

``` text
/etc/ssl/certs/ca-certificates.crt
```

The `curl` hook adds a dependency on `ca-certificates`, so HTTPS
certificate trust is available when curl is installed.

The current native implementation generates the certificate bundle but
does not create the `<hash>.0` links used by applications that rely on
`CApath`. See [Known limitations](limitations.md).

## Re-running hooks

Use:

``` sh
kpkg configure
```

to re-run hooks for installed packages, or specify package names:

``` sh
kpkg configure curl ca-certificates
```

This is useful after correcting a hook or when a previous configuration
step did not complete.

## Adding or changing hooks

Before adding a hook:

1.  Identify the setup normally performed by the package's maintainer
    scripts.
2.  Determine which parts are required for Kyanite's supported use
    cases.
3.  Express the required setup using only supported declarative actions
    or an existing native `builtin`.
4.  Add or adjust dependencies where necessary.
5.  Test installation, reconfiguration and removal behavior.
6.  Document any behavior that cannot be reproduced and any manual steps
    users may need.

Do not assume that every Debian package can be made fully functional
with a hook. Some packages depend on service managers, system users,
generated configuration, triggers or other maintainer-script behavior
that Kyanite may not implement.

For implementation details and the authoritative hook grammar, consult
the hook parser and examples in the source tree. This guide
intentionally does not invent a grammar beyond the documented action
set.
