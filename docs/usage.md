# Usage

This guide covers the interactive shell environment included with Kyanite and the main ways to customize it.

## Starting a session

Kyanite's `/bin/sh` points to `/sbin/init`. Init configures the terminal and starts Brush as an interactive login shell.

When the shell exits, init exits and the session ends.

## Shell configuration files

| Path | Purpose |
| --- | --- |
| `/etc/profile` | Login profile; prints the MOTD and loads `~/.bashrc` |
| `/root/.bashrc` | Prompt, history settings, aliases and helper functions |
| `/etc/update-motd.d/` | Scripts that generate the message of the day |
| `/etc/motd` | Optional static text printed after the MOTD scripts |
| `/etc/kyanite/logo.txt` | ASCII logo displayed by the logo MOTD script |
| `/etc/os-release` | Distribution name and build version |

The default files are maintained in the repository's [`rootfs-overlay/`](../rootfs-overlay/) directory.

## Customize the message of the day

MOTD scripts run in filename order from `/etc/update-motd.d/`.

To disable a script while keeping the file:

```sh
chmod -x /etc/update-motd.d/00-logo
```

To add a static message:

```sh
printf '%s\\n' 'Welcome to Kyanite' > /etc/motd
```

To silence the MOTD for your login:

```sh
touch ~/.hushlogin
```

Colours are disabled when `NO_COLOR` is set or `TERM` is `dumb`.

MOTD scripts are executed with `/bin/brush`, so they do not need a `#!/bin/sh` shebang.

## Customize the logo

Replace `/etc/kyanite/logo.txt` with your own pre-coloured text.

For example, if `chafa` is installed:

```sh
chafa --size=36x --colors=256 logo.png > /etc/kyanite/logo.txt
```

## Aliases and helper commands

The default `.bashrc` includes a prompt, history configuration, aliases and helper functions such as `extract` and `gacp`.

Some commands used by the helpers are installed separately. For example:

```sh
kpkg add tar git curl unzip
```

See the [kpkg guide](kpkg.md) for package installation and management.

## Environment

A launcher normally sets environment variables such as `HOME`, `USER`, `LOGNAME`, `TERM`, `LANG` and `PATH`. The exact values can vary depending on the terminal and host integration.
