declare -A _KX_MISSING=(
    # --- procps ---
    [ps]=procps
    [pkill]=procps
    [pgrep]=procps
    [top]=procps
    [free]=procps
    [uptime]=procps
    [watch]=procps
    [vmstat]=procps
    [w]=procps
    [pmap]=procps
    [slabtop]=procps
    [sysctl]=procps
    [pwdx]=procps
    [tload]=procps
    [skill]=procps
    [snice]=procps

    # --- util-linux ---
    [dmesg]=util-linux
    [lscpu]=util-linux
    [lsmem]=util-linux
    [getopt]=util-linux
    [more]=util-linux
    [whereis]=util-linux
    [rev]=util-linux
    [lsblk]=util-linux
    [blkid]=util-linux
    [fdisk]=util-linux
    [cfdisk]=util-linux
    [sfdisk]=util-linux
    [mount]=util-linux
    [umount]=util-linux
    [hwclock]=util-linux
    [findmnt]=util-linux
    [mountpoint]=util-linux
    [swapon]=util-linux
    [swapoff]=util-linux
    [losetup]=util-linux
    [flock]=util-linux
    [setsid]=util-linux
    [unshare]=util-linux
    [nsenter]=util-linux
    [lsns]=util-linux
    [setpriv]=util-linux
    [prlimit]=util-linux
    [taskset]=util-linux
    [chrt]=util-linux
    [ionice]=util-linux
    [script]=util-linux
    [logger]=bsdutils
    [wall]=util-linux
    [mesg]=util-linux
    [last]=util-linux
    [lastb]=util-linux
    [renice]=util-linux
    [su]=util-linux
    [runuser]=util-linux
    [agetty]=util-linux
    [blkdiscard]=util-linux
    [blkzone]=util-linux
    [chmem]=util-linux
    [choom]=util-linux
    [eject]=util-linux
    [fallocate]=util-linux
    [isosize]=util-linux
    [kill]=util-linux
    [look]=bsdextrautils
    [lslocks]=util-linux
    [lslogins]=util-linux
    [namei]=util-linux
    [partx]=util-linux
    [pivot_root]=util-linux
    [readprofile]=util-linux
    [resizepart]=util-linux
    [rfkill]=util-linux
    [rtcwake]=util-linux
    [setarch]=util-linux
    [switch_root]=util-linux
    [ul]=bsdextrautils
    [uuidgen]=uuid-runtime
    [uuidparse]=util-linux
    [wipefs]=util-linux
    [zramctl]=util-linux

    # --- bsdextrautils ---
    [hexdump]=bsdextrautils
    [column]=bsdextrautils
    [col]=bsdextrautils
    [write]=bsdextrautils

    # --- proc and process utilities ---
    [killall]=psmisc
    [pstree]=psmisc
    [fuser]=psmisc
    [peekfd]=psmisc
    [prtstat]=psmisc

    # --- findutils ---
    [xargs]=findutils
    [locate]=plocate
    [updatedb]=plocate
    [oldfind]=findutils

    # --- text processing ---
    [awk]=mawk
    [gawk]=gawk
    [nawk]=mawk
    [diff]=diffutils
    [cmp]=diffutils
    [sdiff]=diffutils
    [less]=less
    [parallel]=parallel
    [sponge]=moreutils
    [ts]=moreutils
    [vipe]=moreutils
    [ifdata]=moreutils
    [errno]=moreutils
    [pee]=moreutils
    [zrun]=moreutils
    [iconv]=libc-bin
    [recode]=recode
    [dos2unix]=dos2unix
    [unix2dos]=dos2unix
    [fmt]=coreutils
    [fold]=coreutils
    [expand]=coreutils
    [unexpand]=coreutils
    [nl]=coreutils
    [paste]=coreutils
    [join]=coreutils
    [comm]=coreutils
    [csplit]=coreutils
    [split]=coreutils
    [tac]=coreutils
    [shuf]=coreutils
    [numfmt]=coreutils
    [factor]=coreutils
    [seq]=coreutils
    [yes]=coreutils

    # --- standalone system utilities ---
    [lsof]=lsof
    [tree]=tree
    [unzip]=unzip
    [zip]=zip
    [curl]=curl
    [wget]=wget
    [vim]=vim
    [vi]=vim-tiny
    [nano]=nano
    [tar]=tar
    [gzip]=gzip
    [gunzip]=gzip
    [zcat]=gzip
    [bzip2]=bzip2
    [bunzip2]=bzip2
    [xz]=xz-utils
    [unxz]=xz-utils
    [zstd]=zstd
    [unzstd]=zstd
    [lz4]=lz4
    [lzop]=lzop
    [cpio]=cpio
    [pigz]=pigz
    [7z]=7zip
    [7zz]=7zip
    [rar]=unrar
    [unrar]=unrar
    [arj]=arj
    [lha]=lha
    [cabextract]=cabextract
    [rpm2cpio]=rpm2cpio
    [rsync]=rsync
    [timeout]=coreutils
    [file]=file
    [which]=debianutils
    [hostname]=hostname
    [run-parts]=debianutils
    [clear]=ncurses-bin
    [reset]=ncurses-bin
    [tput]=ncurses-bin
    [infocmp]=ncurses-bin
    [xxd]=xxd
    [mcookie]=util-linux

    # --- shells ---
    [bash]=bash
    [dash]=dash
    [zsh]=zsh
    [fish]=fish
    [ksh]=ksh
    [mksh]=mksh
    [tcsh]=tcsh
    [csh]=tcsh
    [busybox]=busybox

    # --- system management ---
    [systemctl]=systemd
    [journalctl]=systemd
    [timedatectl]=systemd
    [hostnamectl]=systemd
    [loginctl]=systemd
    [resolvectl]=systemd-resolved
    [networkctl]=systemd
    [bootctl]=systemd
    [systemd-analyze]=systemd
    [systemd-run]=systemd
    [systemd-cgls]=systemd
    [systemd-cgtop]=systemd
    [service]=init-system-helpers
    [update-rc.d]=init-system-helpers
    [invoke-rc.d]=init-system-helpers
    [crontab]=cron
    [at]=at
    [atrm]=at
    [atq]=at
    [logrotate]=logrotate
    [chpasswd]=passwd
    [newgrp]=passwd
    [nologin]=login
    [login]=login
    [shutdown]=systemd-sysv
    [reboot]=systemd-sysv
    [poweroff]=systemd-sysv
    [halt]=systemd-sysv

    # --- users and permissions ---
    [adduser]=adduser
    [deluser]=adduser
    [useradd]=passwd
    [userdel]=passwd
    [usermod]=passwd
    [groupadd]=passwd
    [groupdel]=passwd
    [groupmod]=passwd
    [passwd]=passwd
    [chsh]=passwd
    [chfn]=passwd
    [sudo]=sudo
    [visudo]=sudo
    [doas]=opendoas
    [chage]=passwd
    [groups]=coreutils
    [who]=coreutils
    [users]=coreutils
    [lastlog]=login
    [newusers]=passwd
    [vipw]=passwd
    [vigr]=passwd
    [pwck]=passwd
    [grpck]=passwd

    # --- kernel modules ---
    [lsmod]=kmod
    [modprobe]=kmod
    [insmod]=kmod
    [rmmod]=kmod
    [modinfo]=kmod
    [depmod]=kmod

    # --- hardware diagnostics ---
    [lspci]=pciutils
    [setpci]=pciutils
    [lsusb]=usbutils
    [usb-devices]=usbutils
    [lshw]=lshw
    [dmidecode]=dmidecode
    [sensors]=lm-sensors
    [sensors-detect]=lm-sensors
    [smartctl]=smartmontools
    [hdparm]=hdparm
    [nvme]=nvme-cli
    [ethtool]=ethtool
    [inxi]=inxi
    [hwinfo]=hwinfo
    [acpi]=acpi
    [upower]=upower
    [udevadm]=udev
    [udevinfo]=udev
    [udisksctl]=udisks2
    [boltctl]=bolt

    # --- filesystem management ---
    [mkfs.ext4]=e2fsprogs
    [mkfs.ext3]=e2fsprogs
    [mkfs.ext2]=e2fsprogs
    [mke2fs]=e2fsprogs
    [fsck]=e2fsprogs
    [e2fsck]=e2fsprogs
    [tune2fs]=e2fsprogs
    [filefrag]=e2fsprogs
    [dumpe2fs]=e2fsprogs
    [debugfs]=e2fsprogs
    [resize2fs]=e2fsprogs
    [parted]=parted
    [partprobe]=parted
    [fstrim]=util-linux
    [mkfs.vfat]=dosfstools
    [mkfs.fat]=dosfstools
    [fsck.vfat]=dosfstools
    [mkfs.ntfs]=ntfs-3g
    [ntfsfix]=ntfs-3g
    [ntfs-3g]=ntfs-3g
    [mkfs.btrfs]=btrfs-progs
    [btrfs]=btrfs-progs
    [mkfs.xfs]=xfsprogs
    [xfs_repair]=xfsprogs
    [xfs_info]=xfsprogs
    [mkfs.f2fs]=f2fs-tools
    [f2fs-tools]=f2fs-tools

    # --- net-tools ---
    [ifconfig]=net-tools
    [netstat]=net-tools
    [route]=net-tools
    [arp]=net-tools
    [rarp]=net-tools
    [nameif]=net-tools

    # --- iproute2 ---
    [ip]=iproute2
    [ss]=iproute2
    [tc]=iproute2
    [bridge]=iproute2
    [rtmon]=iproute2
    [lnstat]=iproute2
    [nstat]=iproute2
    [devlink]=iproute2

    # --- iputils ---
    [ping]=iputils-ping
    [arping]=iputils-arping
    [tracepath]=iputils-tracepath
    [clockdiff]=iputils-clockdiff

    # --- DNS utilities ---
    [dig]=bind9-dnsutils
    [nslookup]=bind9-dnsutils
    [host]=bind9-host
    [delv]=bind9-dnsutils
    [nsupdate]=bind9-dnsutils

    # --- networking ---
    [telnet]=inetutils-telnet
    [traceroute]=traceroute
    [mtr]=mtr-tiny
    [nc]=netcat-openbsd
    [ncat]=ncat
    [socat]=socat
    [tcpdump]=tcpdump
    [nmap]=nmap
    [whois]=whois
    [iperf3]=iperf3
    [iptables]=iptables
    [ip6tables]=iptables
    [nft]=nftables
    [ufw]=ufw
    [iw]=iw
    [iwconfig]=wireless-tools
    [iwlist]=wireless-tools
    [iwgetid]=wireless-tools
    [nmcli]=network-manager
    [nmtui]=network-manager
    [wpa_cli]=wpasupplicant
    [wpa_passphrase]=wpasupplicant
    [dhclient]=isc-dhcp-client
    [dhcping]=dhcping
    [arp-scan]=arp-scan
    [fping]=fping
    [hping3]=hping3
    [sshuttle]=sshuttle
    [autossh]=autossh
    [mosh]=mosh
    [ftp]=tnftp
    [tftp]=tftp-hpa
    [lftp]=lftp

    # --- remote access ---
    [ssh]=openssh-client
    [scp]=openssh-client
    [sftp]=openssh-client
    [ssh-keygen]=openssh-client
    [ssh-copy-id]=openssh-client
    [ssh-agent]=openssh-client
    [sshd]=openssh-server
    [sftp-server]=openssh-sftp-server

    # --- standalone network applications ---
    [aria2c]=aria2
    [rclone]=rclone
    [yt-dlp]=yt-dlp
    [transmission-cli]=transmission-cli
    [transmission-daemon]=transmission-daemon
    [axel]=axel

    # --- package management ---
    [apt]=apt
    [apt-get]=apt
    [apt-cache]=apt
    [apt-mark]=apt
    [apt-config]=apt
    [apt-cdrom]=apt
    [dpkg]=dpkg
    [dpkg-query]=dpkg
    [dpkg-deb]=dpkg
    [dpkg-divert]=dpkg
    [dpkg-statoverride]=dpkg
    [dpkg-trigger]=dpkg
    [update-alternatives]=dpkg
    [dpkg-reconfigure]=debconf
    [debconf-show]=debconf
    [debconf-communicate]=debconf
    [aptitude]=aptitude
    [tasksel]=tasksel
    [apt-file]=apt-file
    [apt-listchanges]=apt-listchanges
    [needrestart]=needrestart
    [add-apt-repository]=software-properties-common
    [gdebi]=gdebi-core
    [nala]=nala

    # --- development and languages ---
    [node]=nodejs
    [npm]=npm
    [npx]=npm
    [corepack]=node-corepack
    [python3]=python3
    [python]=python-is-python3
    [pip3]=python3-pip
    [pip]=python3-pip
    [venv]=python3-venv
    [virtualenv]=virtualenv
    [perl]=perl
    [ruby]=ruby
    [gem]=ruby
    [php]=php-cli
    [composer]=composer
    [java]=default-jre-headless
    [javac]=default-jdk
    [jshell]=default-jdk
    [gcc]=build-essential
    [cc]=gcc
    [make]=build-essential
    [cmake]=cmake
    [ninja]=ninja-build
    [meson]=meson
    [clang]=clang
    [pkg-config]=pkg-config
    [git]=git
    [git-lfs]=git-lfs
    [svn]=subversion
    [hg]=mercurial
    [go]=golang-go
    [rustc]=rustc
    [cargo]=cargo
    [rustfmt]=rustfmt
    [clippy-driver]=clippy
    [docker]=docker.io
    [podman]=podman
    [buildah]=buildah
    [skopeo]=skopeo
    [kubectl]=kubernetes-client
    [helm]=helm
    [terraform]=terraform
    [ansible]=ansible
    [vagrant]=vagrant
    [packer]=packer
    [gradle]=gradle
    [mvn]=maven
    [ant]=ant
    [dotnet]=dotnet-sdk-8.0
    [mono]=mono-runtime
    [msbuild]=mono-devel

    # --- binary inspection and debugging ---
    [strings]=binutils
    [readelf]=binutils
    [objdump]=binutils
    [nm]=binutils
    [strip]=binutils
    [ar]=binutils
    [as]=binutils
    [ld]=binutils
    [addr2line]=binutils
    [gdb]=gdb
    [gdbserver]=gdbserver
    [valgrind]=valgrind
    [objcopy]=binutils
    [size]=binutils
    [patchelf]=patchelf
    [checksec]=checksec
    [openssl]=openssl
    [gpg]=gnupg
    [gpgv]=gpgv
    [strace]=strace
    [ltrace]=ltrace
    [getcap]=libcap2-bin
    [setcap]=libcap2-bin
    [capsh]=libcap2-bin

    # --- JSON, XML and data processing ---
    [jq]=jq
    [yq]=yq
    [xmllint]=libxml2-utils
    [xmlstarlet]=xmlstarlet
    [envsubst]=gettext-base
    [msgfmt]=gettext
    [locale]=libc-bin
    [localedef]=libc-bin
    [getent]=libc-bin
    [ldconfig]=libc-bin

    # --- databases ---
    [sqlite3]=sqlite3
    [mysql]=mariadb-client
    [mariadb]=mariadb-client
    [psql]=postgresql-client
    [pg_dump]=postgresql-client
    [redis-cli]=redis-tools
    [mongosh]=mongodb-mongosh
    [mysqladmin]=mariadb-client
    [mysqldump]=mariadb-client

    # --- modern terminal utilities ---
    [htop]=htop
    [btop]=btop
    [ncdu]=ncdu
    [duf]=duf
    [tmux]=tmux
    [screen]=screen
    [byobu]=byobu
    [fzf]=fzf
    [rg]=ripgrep
    [bat]=bat
    [batcat]=bat
    [eza]=eza
    [fd]=fd-find
    [fdfind]=fd-find
    [ag]=silversearcher-ag
    [ranger]=ranger
    [lf]=lf
    [nnn]=nnn
    [mc]=mc
    [tldr]=tldr
    [tealdeer]=tealdeer
    [rz]=lrzsz
    [sz]=lrzsz
    [dialog]=dialog
    [whiptail]=whiptail
    [man]=man-db
    [apropos]=man-db
    [whatis]=man-db
    [info]=info
    [most]=most
    [chafa]=chafa
    [figlet]=figlet
    [toilet]=toilet
    [lolcat]=lolcat
    [cowsay]=cowsay
    [fortune]=fortune-mod
    [cal]=ncal
    [ncal]=ncal

    # --- diagnostics and monitoring ---
    [iotop]=iotop
    [iostat]=sysstat
    [mpstat]=sysstat
    [sar]=sysstat
    [pidstat]=sysstat
    [powertop]=powertop
    [dstat]=dstat
    [atop]=atop
    [nmon]=nmon
    [glances]=glances

    # --- security and certificates ---
    [certbot]=certbot
    [gpg2]=gnupg
    [age]=age
    [ssh-keyscan]=openssh-client
    [pwgen]=pwgen
    [mkpasswd]=whois
    [chroot]=coreutils
    [apparmor_status]=apparmor-utils
    [aa-status]=apparmor-utils
    [aa-enforce]=apparmor-utils
    [aa-complain]=apparmor-utils
    [firejail]=firejail
    [bubblewrap]=bubblewrap

    # --- multimedia ---
    [ffmpeg]=ffmpeg
    [ffprobe]=ffmpeg
    [sox]=sox
    [convert]=imagemagick
    [identify]=imagemagick
    [magick]=imagemagick
    [montage]=imagemagick
    [mogrify]=imagemagick
    [rsvg-convert]=librsvg2-bin
    [pdftotext]=poppler-utils
    [pdfinfo]=poppler-utils
    [pdftoppm]=poppler-utils
    [gs]=ghostscript
    [exiftool]=libimage-exiftool-perl
    [mediainfo]=mediainfo

    # --- office and document tools ---
    [pandoc]=pandoc
    [libreoffice]=libreoffice
    [soffice]=libreoffice
    [antiword]=antiword
    [odt2txt]=odt2txt

    # --- diagnostics and hardware ---

    # --- localization ---
    [locale-gen]=locales
    [update-locale]=locales

    # --- miscellaneous ---
    [bash-completion]=bash-completion
    [logsave]=e2fsprogs
    [scriptreplay]=util-linux
    [chcon]=coreutils
    [runcon]=coreutils
    [getfacl]=acl
    [setfacl]=acl
    [getfattr]=attr
    [setfattr]=attr
    [attr]=attr
    [rsyslogd]=rsyslog
    [cron]=cron
    [crond]=cron
    [anacron]=anacron
    [atd]=at
    [getconf]=libc-bin
)
