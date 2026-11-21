# TODO

### Investigate
- Check why x86 build is double the size of arm
- Qemu error:
```
[dlc] rootfs channel: /dev/vport1p1
[dlc] receiving rootfs into RAM...
[dlc] rootfs ready, switching root. Tip: run 'stty rows N cols M' to match your screen.
[dlc] starting /bin/sh (this is PID 1: leaving it powers the VM off)
[init] tcsetpgrp(0, 166) FALLO: Not a tty (os error 25)

       /\
      /  \
     / /\ \
    / /  \ \     K Y A N I T E
    \ \  / /     -------------
     \ \/ /      Secure. Light.
      \  /       Comfortable.
       \/

  Kyanite 7a5a906 (aarch64)

  Kernel    Linux 6.12.107+deb13-arm64
  Uptime    0h 0m
  Memory    140 MiB / 464 MiB
  Disk      114M used of 419M (28%)

  Packages  kpkg update && kpkg add <package>
  Editor    vi
  Project   https://github.com/StringManolo/Kyanite

  Edit /etc/update-motd.d/ or /etc/motd to customize. Silence with: touch ~/.hushlogin
root@kyanite:~#
```
- Also fix tty on qemu

### Security
- Check for env replacement for a Rust version 
- Check all files 1 by 1.
- Make sure last security patches of kpkg overide stable
- Add opengpg check of signatures pre https
- PRIORITY Add autopdate from nightly realeases created from github actions (only when a core component updates) (check how to automatize to rebuild on security related updates of any of the sofware) (download the tarball, readchangelogs or missmatched sha, and replace)
- Balance between using musl static vs glib dinamic for shiped bins
- Check for new features

### Quality Of Life
- Add better logs to init
- Add preconfigured scripts
- Add MOTD
- Add custom PS1 + alias + good default brush config (.bashrc)
- Ship vi (check if rust ports are available) by default
- Check if installing other shells messes with init and/or brush
- Ship as own APK for Android with DebianLiveCD4Android as base APK.
- Add proot-distro install script in the repo
- Add screenshot (generate from logo.txt, try chafta)
- Missing basics: find, grep
- Recycle bin
- CloudBackups
- Swap Kernel + Check Rufus installs
- Patch apps that work but they think they don't like nmap
- Include support for X11?

### Stability
- Test on Qemu
- Test x86
- Test on udocker
- Test on proot distro (pr to termux distros?)
- Test sh and env bash scripts (needs some thinking here about glue by default by allowing user to swap to bash, maybe a security warning with a list of options build into Kpkg)
- /bin/sh points to the init directory. Any Debian package with #!/bin/sh would launch the init directory instead of a shell. That's why the MOTD runs its scripts with /bin/brush. A future solution is to have the init directory, if it's not PID 1, execute brush with the received arguments.
- The workflow uses Ubuntu packages, not Debian. It runs on ubuntu-latest, and `apt-get download` downloads Ubuntu's glibc, but `kpkg` installs from Debian stable. If Debian stable has a newer version of glibc, there might be incompatibilities. The solution would be to build it inside a `debian:stable` container.


### Size
- Remove /usr/share/vim/vim91/doc/version9.txt (2 MB)
- Remove /usr/share/vim/vim91/doc/uganda.txt (12 KB)

- Inspect Brush (13MB) (replacement??)
- Inspect coreutils (11MB) (replacement??)

- Slowly replace or fork

### Other
- Use as base to new security based distro (based on SOSCW but made with a single install script that download the lastest versions and patches them. Every tool has its own script and they are all called from a main script)
