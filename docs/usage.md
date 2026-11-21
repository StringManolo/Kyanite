# Usage

This guide describes the shell environment included in the Kyanite root
filesystem.

## Login shell

Kyanite's `/bin/sh` is a symlink to `/sbin/init`. Init starts Brush as
an interactive login shell, configures terminal ownership and job
control, reaps orphaned processes, and exits when the shell exits.

The login environment is configured through files in the root filesystem
overlay.

## Shell configuration

  -----------------------------------------------------------------------
  Path                                Purpose
  ----------------------------------- -----------------------------------
  `/etc/profile`                      Read by Brush on login; prints the
                                      MOTD and loads `~/.bashrc`

  `/root/.bashrc`                     Prompt, history settings, aliases
                                      and helper functions

  `/etc/update-motd.d/`               MOTD scripts, executed in name
                                      order at each login

  `/etc/motd`                         Optional static text printed after
                                      the scripts; create it if desired

  `/etc/kyanite/logo.txt`             ASCII logo displayed by the logo
                                      MOTD script

  `/etc/os-release`                   Build name and version; also used
                                      by the MOTD
  -----------------------------------------------------------------------

The shipped rootfs overlay is available in the repository's
[`rootfs-overlay/`](../rootfs-overlay/) directory.

## Customizing the MOTD

MOTD scripts are run from `/etc/update-motd.d/` in name order. To
disable a script without deleting it, remove its executable bit:

``` sh
chmod -x /etc/update-motd.d/00-logo
```

To add static text after the scripts:

``` sh
printf '%s\n' 'Welcome to Kyanite' > /etc/motd
```

To silence the MOTD for a login, create:

``` sh
touch ~/.hushlogin
```

Colours are disabled when `NO_COLOR` is set or `TERM` is `dumb`.

Because `/bin/sh` points to Kyanite's init, MOTD scripts are executed
with `/bin/brush`; they do not need a `#!/bin/sh` shebang.

## Logo

Replace `/etc/kyanite/logo.txt` with your own pre-coloured text. For
example, if `chafa` is installed:

``` sh
chafa --size=36x --colors=256 logo.png > /etc/kyanite/logo.txt
```

## Aliases and helper commands

The default `.bashrc` includes a prompt, unlimited history, aliases, and
helpers such as `extract` and `gacp`.

Some commands referenced by aliases and helpers are not included in the
minimal base. Install them when needed, for example:

``` sh
kpkg add tar git curl unzip
```

See [kpkg documentation](kpkg.md) for package installation and
management.

## Environment variables

The simplified PRoot launch example sets `HOME`, `USER`, `LOGNAME`,
`TERM`, `COLORTERM`, `LANG` and `PATH` explicitly. A launcher may choose
different values depending on the host terminal and its integration
requirements.
