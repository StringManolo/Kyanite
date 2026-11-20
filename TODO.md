# TODO

### Investigate
- Check why x86 build is double the size of arm

### Security
- Check for env replacement for a Rust version 
- Check all files 1 by 1.
- Make sure last security patches of kpkg overide stable
- Add opengpg check of signatures pre https
- Add autopdate from nightly realeases created from github actions (only when a core component updates)

### Quality Of Life
- Add README.md
- Add better logs to init
- Add preconfigured scripts
- Add MOTD
- Add custom PS1 + alias + good default brush config (.bashrc)
- Ship vi (check if rust ports are available) by default
- Check if installing other shells messes with init and/or brush
- Ship as own APK for Android with DebianLiveCD4Android as base APK.
- Add proot-distro install script in the repo
- Add screenshot

### Stability
- Test on Qemu
- Test x86
- Test on udocker
- Test on proot distro (pr to termux distros?)
- Test sh and env bash scripts (needs some thinking here about glue by default by allowing user to swap to bash, maybe a security warning with a list of options build into Kpkg)
- /bin/sh points to the init directory. Any Debian package with #!/bin/sh would launch the init directory instead of a shell. That's why the MOTD runs its scripts with /bin/brush. A future solution is to have the init directory, if it's not PID 1, execute brush with the received arguments.
- The workflow uses Ubuntu packages, not Debian. It runs on ubuntu-latest, and `apt-get download` downloads Ubuntu's glibc, but `kpkg` installs from Debian stable. If Debian stable has a newer version of glibc, there might be incompatibilities. The solution would be to build it inside a `debian:stable` container.
