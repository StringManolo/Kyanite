#!/bin/sh
# Kyanite installer. Runs inside the Alpine installer layer (chroot), launched by installer-wrapper.sh.
#
# Usage: kyanite-install.sh [--auto auto|/dev/sdX] [--mode wipe|alongside|manual] [--keymap LAYOUT]
#                           [--root DEV] [--esp DEV] [--format-root yes|no] [--format-esp yes|no]
#
# Modes
#   wipe       Automatic. Erase a whole disk. GPT: bios_grub + EFI System Partition + ext4 root.
#              Bootable in BIOS and UEFI. Kernel and initramfs live in the ESP.                (default)
#   alongside  Automatic. Keep the existing system (for example Windows) and install in the free space.
#              UEFI + GPT disk : new ext4 root partition, existing EFI System Partition reused.
#              BIOS + MBR disk : new primary ext4 root partition, GRUB goes to the MBR.
#              Existing partitions are never modified. A GRUB entry chainloads Windows when it is found.
#   manual     The user partitions the disks (cfdisk) and picks the partitions:
#                --root DEV        partition for the system (formatted as ext4 unless --format-root no)
#                --esp DEV         EFI System Partition (UEFI only; reused unless --format-esp yes)
#              BIOS installs put GRUB in the MBR/gap of the disk that holds the root partition.
#              (Kyanite's init does not read /etc/fstab, so swap and extra mount points are not offered.)
#
#   --auto auto  pick the first suitable disk and do not ask anything (used by CI)
#
# Paths inside the chroot: /src/rootfs (system tree), /src/medium (ISO: boot/vmlinuz, boot/initramfs),
#                          /usr/local/share/kyanite/grub.cfg.in (GRUB template)
set -eu
(set -o pipefail) 2>/dev/null && set -o pipefail

trap 'rc=$?; [ "$rc" -eq 0 ] || echo "KYANITE-INSTALL: ERROR: exited with code $rc" >&2' EXIT

say() { echo "KYANITE-INSTALL: $*"; }
die() { echo "KYANITE-INSTALL: ERROR: $*" >&2; exit 1; }

AUTO=""; MODE=""; KEYMAP=""; ROOTARG=""; ESPARG=""; FMT_ROOT="yes"; FMT_ESP="no"
while [ $# -gt 0 ]; do
  case "$1" in
    --auto)        AUTO="${2:-auto}";    shift; [ $# -gt 0 ] && shift ;;
    --mode)        MODE="${2:-}";        shift; [ $# -gt 0 ] && shift ;;
    --keymap)      KEYMAP="${2:-}";      shift; [ $# -gt 0 ] && shift ;;
    --root)        ROOTARG="${2:-}";     shift; [ $# -gt 0 ] && shift ;;
    --esp)         ESPARG="${2:-}";      shift; [ $# -gt 0 ] && shift ;;
    --format-root) FMT_ROOT="${2:-yes}"; shift; [ $# -gt 0 ] && shift ;;
    --format-esp)  FMT_ESP="${2:-no}";   shift; [ $# -gt 0 ] && shift ;;
    *) shift ;;
  esac
done

CMD_KEYMAP=""
for p in $(cat /proc/cmdline); do
  case "$p" in
    kyanite.keymap=*) CMD_KEYMAP="${p#kyanite.keymap=}" ;;
  esac
done

UEFI=0
[ -d /sys/firmware/efi ] && UEFI=1

modprobe -ab vfat ext4 nls_cp437 nls_iso8859_1 2>/dev/null || true
if [ "$UEFI" = 1 ]; then
  mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null || true
fi

# ---- helpers: partition tables --------------------------------------------------------------------
# NOTE: every parser captures the tool output first and only then filters it. A consumer that exits early
# (grep -q, awk exit) in a pipeline would kill the producer with SIGPIPE, and with `pipefail` that makes
# the whole check fail. Parsers below always read their whole input.
parent_of() {
  n=${1#/dev/}
  if [ -e "/sys/class/block/$n/partition" ]; then
    basename "$(dirname "$(readlink -f "/sys/class/block/$n")")"
  else
    echo "$n"
  fi
}

gpt_info() { sgdisk -p "$1" 2>/dev/null || true; }
is_gpt()   { case "$(gpt_info "$1")" in *"GPT: present"*) return 0 ;; esac; return 1; }
esp_number() {
  printf '%s\n' "$(gpt_info "$1")" | awk '$1 ~ /^[0-9]+$/ && toupper($6) == "EF00" && !f {print $1; f=1}'
}
part_numbers() {
  printf '%s\n' "$(gpt_info "$1")" | awk '$1 ~ /^[0-9]+$/ {print $1}'
}
lbs_of() { cat "/sys/block/${1#/dev/}/queue/logical_block_size" 2>/dev/null || echo 512; }

# Largest free block of a GPT disk, in MiB
free_mib() {
  d=$1; lbs=$(lbs_of "$d")
  first=$(sgdisk -F "$d" 2>/dev/null | tail -n 1) || first=""
  last=$(sgdisk -E "$d" 2>/dev/null | tail -n 1) || last=""
  case "$first$last" in ''|*[!0-9]*) echo 0; return ;; esac
  if [ "$last" -ge "$first" ]; then echo $(( (last - first + 1) * lbs / 1048576 )); else echo 0; fi
}

mbr_info()  { sfdisk -d "$1" 2>/dev/null || true; }
is_msdos()  { case "$(mbr_info "$1")" in *"label: dos"*) return 0 ;; esac; return 1; }
mbr_entries() { printf '%s\n' "$(mbr_info "$1")" | awk '/^\/dev\// {print $1}'; }
mbr_first_start() {
  printf '%s\n' "$(mbr_info "$1")" | sed -n 's/.*start= *\([0-9][0-9]*\),.*/\1/p' | sort -n | sed -n 1p
}
# Largest free block of an MBR disk, in MiB (sfdisk -F prints: Start End Sectors Size)
free_mib_mbr() {
  d=$1; lbs=$(lbs_of "$d")
  sectors=$(printf '%s\n' "$(sfdisk -F "$d" 2>/dev/null || true)" \
    | awk '$1 ~ /^[0-9]+$/ && NF >= 4 { if ($3 + 0 > m) m = $3 + 0 } END { print m + 0 }') || sectors=0
  echo $(( sectors * lbs / 1048576 ))
}

blk_attr() {   # blk_attr DEVICE UUID|TYPE|LABEL
  blkid "$1" 2>/dev/null | sed -n "s/.* $2=\"\([^\"]*\)\".*/\1/p" || true
}

wait_for_node() {
  i=0
  while [ ! -b "$1" ] && [ "$i" -lt 15 ]; do
    sleep 1; i=$((i + 1)); mdev -s 2>/dev/null || true
  done
  [ -b "$1" ] || die "device node $1 did not appear"
}

# alongside_check DISK -> sets ALONG_KIND (uefi|bios) and ALONG_WHY (human readable). Returns 0 when possible.
ALONG_KIND=""; ALONG_WHY=""
alongside_check() {
  d=$1; ALONG_KIND=""; ALONG_WHY=""
  if [ "$UEFI" = 1 ]; then
    if ! is_gpt "$d"; then ALONG_WHY="UEFI boot but the disk has no GPT"; return 1; fi
    esp=$(esp_number "$d") || esp=""
    if [ -z "$esp" ]; then ALONG_WHY="UEFI boot but the disk has no EFI System Partition"; return 1; fi
    free=$(free_mib "$d")
    if [ "$free" -lt 2048 ]; then ALONG_WHY="only $free MiB free (need 2048)"; return 1; fi
    ALONG_KIND=uefi; ALONG_WHY="UEFI/GPT, EFI System Partition #$esp, $free MiB free"; return 0
  fi
  if is_gpt "$d"; then ALONG_WHY="BIOS boot but the disk is GPT: boot the live system in UEFI mode to install alongside it"; return 1; fi
  if ! is_msdos "$d"; then ALONG_WHY="BIOS boot but the disk has no recognised partition table"; return 1; fi
  cnt=$(mbr_entries "$d" | wc -l) || cnt=0
  if [ "$cnt" -lt 1 ]; then ALONG_WHY="the MBR disk has no partitions (use the wipe mode)"; return 1; fi
  if [ "$cnt" -ge 4 ]; then ALONG_WHY="all 4 primary partition slots are in use"; return 1; fi
  fs=$(mbr_first_start "$d") || fs=""
  if [ "${fs:-0}" -lt 2048 ]; then ALONG_WHY="first partition starts at sector ${fs:-?}: no room for GRUB after the MBR"; return 1; fi
  free=$(free_mib_mbr "$d")
  if [ "$free" -lt 2048 ]; then ALONG_WHY="only $free MiB free (need 2048)"; return 1; fi
  ALONG_KIND=bios; ALONG_WHY="BIOS/MBR, $cnt partitions, $free MiB free"; return 0
}

# ---- which disk is the installation medium (never offer it) -----------------------------------
MEDIA_DEV=$(awk '$2=="/src/medium"{print $1; exit}' /proc/mounts || true)
MEDIA_PARENT=""
[ -n "$MEDIA_DEV" ] && MEDIA_PARENT=$(parent_of "$MEDIA_DEV")

# ---- candidate disks (>= 1 GiB, no loop/ram/cdrom, not the medium) ------------------------------
CANDIDATES=""
for d in /sys/block/*; do
  n=${d##*/}
  case "$n" in loop*|ram*|sr*|fd*|zram*|dm-*|md*|nbd*) continue ;; esac
  [ "$n" = "$MEDIA_PARENT" ] && continue
  [ "$(cat "$d/size")" -ge 2097152 ] || continue
  CANDIDATES="$CANDIDATES $n"
done
[ -n "$CANDIDATES" ] || die "no candidate disks found (>= 1 GiB)"

# ---- common pieces ------------------------------------------------------------------------------------
KINIT=/sbin/init
[ -x /src/rootfs/usr/lib/kyanite/hw/boot-init ] && KINIT=/usr/lib/kyanite/hw/boot-init

# GRUB config from the shared template. $1 = "" (kernel in the ESP) or "/boot" (kernel in the root partition)
write_grub_cfg() {
  sed -e "s#@ROOT_UUID@#$UUID#g" -e "s#@KINIT@#$KINIT#g" -e "s#@KPREFIX@#$1#g" \
    /usr/local/share/kyanite/grub.cfg.in
}

# Entry that chainloads Windows. It is evaluated by GRUB at boot, so it also works if Windows is
# installed (or moved) later: the entry only shows up when the Windows boot files are found.
windows_block() {   # $1 = uefi | bios
  case "$1" in
    uefi) cat <<'EOF'

insmod part_gpt
insmod fat
insmod chain
if search --no-floppy --file --set=winroot /EFI/Microsoft/Boot/bootmgfw.efi; then
  menuentry "Windows" --id windows {
    set root=$winroot
    chainloader /EFI/Microsoft/Boot/bootmgfw.efi
  }
fi
EOF
    ;;
    bios) cat <<'EOF'

insmod part_msdos
insmod part_gpt
insmod ntfs
insmod chain
if search --no-floppy --file --set=winroot /bootmgr; then
  menuentry "Windows" --id windows {
    set root=$winroot
    chainloader +1
  }
fi
EOF
    ;;
  esac
}

# GRUB config for layouts where the kernel lives in the root partition (/boot)
grub_cfg_root() {   # $1 = uefi | bios
  write_grub_cfg "/boot" | sed 's/^set timeout=3$/set timeout=5/'
  windows_block "$1"
}

copy_system() {
  say "copying the system (this can take a while)"
  tar -C /src/rootfs -cf - . | tar -C /target -xpf -
  if [ -n "$KEYMAP" ] && [ "$KEYMAP" != us ]; then
    mkdir -p /target/etc/kyanite
    echo "$KEYMAP" > /target/etc/kyanite/keymap
  fi
}

ask_keymap() {
  if [ -z "$KEYMAP" ]; then
    if [ -z "$AUTO" ]; then
      cur=${CMD_KEYMAP:-us}
      printf "Keyboard layout [%s] (e.g. us, es, de, fr, gb, it): " "$cur"
      read -r k
      KEYMAP=${k:-$cur}
    else
      KEYMAP=$CMD_KEYMAP
    fi
  fi
  if [ -n "$KEYMAP" ] && [ "$KEYMAP" != us ]; then
    if [ ! -f "/src/rootfs/usr/share/bkeymaps/${KEYMAP%%-*}/$KEYMAP.bmap.gz" ]; then
      say "warning: keymap '$KEYMAP' not found, using us"
      KEYMAP=us
    fi
  fi
}

confirm() {   # confirm "text"   (interactive only)
  [ -z "$AUTO" ] || return 0
  printf "%s Type YES to continue: " "$1"
  read -r a
  [ "$a" = "YES" ] || die "cancelled"
}

# Kernel, initramfs and GRUB in a layout where /boot lives in the root partition, UEFI flavour.
# $1 = EFI System Partition device (already formatted)
install_uefi_rootboot() {
  esp=$1
  espn=$(cat "/sys/class/block/${esp#/dev/}/partition") || die "$esp is not a partition"
  espdisk=/dev/$(parent_of "$esp")
  # iocharset=iso8859-1 keeps FAT lookups case-insensitive, so we never create "EFI/BOOT" next to Windows' "EFI/Boot"
  mount -t vfat -o iocharset=iso8859-1,shortname=mixed "$esp" /mnt/esp
  set -- $(df -k /mnt/esp | tail -n 1)
  [ "$4" -ge 8192 ] || die "the EFI System Partition has less than 8 MiB free"

  say "installing GRUB (UEFI, kernel in the root partition)"
  mkdir -p /target/boot/grub
  cp /src/medium/boot/vmlinuz /target/boot/vmlinuz
  cp /src/medium/boot/initramfs /target/boot/initramfs
  grub_cfg_root uefi > /target/boot/grub/grub.cfg
  grub-editenv /target/boot/grub/grubenv create
  grub-install --target=x86_64-efi --efi-directory=/mnt/esp --boot-directory=/target/boot \
    --bootloader-id=Kyanite --modules="part_gpt part_msdos fat ext2 chain" --no-nvram
  [ -f /mnt/esp/EFI/Kyanite/grubx64.efi ] || die "grub-install did not create EFI/Kyanite/grubx64.efi"

  # Fallback path, only when it does not exist: never overwrite what another OS installed there
  if [ ! -e /mnt/esp/EFI/BOOT/BOOTX64.EFI ]; then
    mkdir -p /mnt/esp/EFI/BOOT
    cp /mnt/esp/EFI/Kyanite/grubx64.efi /mnt/esp/EFI/BOOT/BOOTX64.EFI
    say "installed the fallback boot path EFI/BOOT/BOOTX64.EFI"
  fi

  # UEFI boot entry (best effort: the fallback path already makes the disk bootable)
  if command -v efibootmgr >/dev/null 2>&1; then
    if efibootmgr --create --disk "$espdisk" --part "$espn" --loader '\EFI\Kyanite\grubx64.efi' \
        --label "Kyanite" >/dev/null 2>&1; then
      say "UEFI boot entry registered"
    else
      say "warning: could not register a UEFI boot entry (the fallback path is installed)"
    fi
  fi
}

# Kernel, initramfs and GRUB for BIOS: /boot in the root partition, GRUB in the MBR/gap of $1
install_bios_rootboot() {
  say "installing GRUB (BIOS) on $1"
  mkdir -p /target/boot/grub
  cp /src/medium/boot/vmlinuz /target/boot/vmlinuz
  cp /src/medium/boot/initramfs /target/boot/initramfs
  grub_cfg_root bios > /target/boot/grub/grub.cfg
  grub-editenv /target/boot/grub/grubenv create
  grub-install --target=i386-pc --boot-directory=/target/boot \
    --modules="part_msdos part_gpt ext2 ntfs chain" --no-floppy "$1"
}

finish() {
  sync
  umount /mnt/esp 2>/dev/null || true
  umount /target
  sync
  say "installation finished (mode: $MODE)"
  [ -n "$AUTO" ] && echo "KYANITE-INSTALL-DONE"
  exit 0
}

mkdir -p /target /mnt/esp

# ================================== mode: wipe ================================================
run_wipe() {
  DISK=$1
  n=${DISK#/dev/}
  case "$n" in *[0-9]) P="${DISK}p" ;; *) P="$DISK" ;; esac
  confirm "ALL DATA on $DISK will be ERASED."
  UUID=$(cat /proc/sys/kernel/random/uuid)
  ask_keymap

  say "partitioning $DISK"
  dd if=/dev/zero of="$DISK" bs=1M count=2 conv=fsync 2>/dev/null
  sgdisk -o "$DISK" >/dev/null
  sgdisk -n 1:2048:4095 -t 1:ef02 -c 1:biosboot -A 1:set:2 "$DISK" >/dev/null
  sgdisk -n 2:0:+256M -t 2:ef00 -c 2:KYBOOT "$DISK" >/dev/null
  sgdisk -n 3:0:0 -t 3:8300 -c 3:kyroot "$DISK" >/dev/null
  wait_for_node "${P}3"

  say "formatting"
  mkfs.vfat -F 32 -n KYBOOT "${P}2" >/dev/null
  mkfs.ext4 -q -F -L kyroot -U "$UUID" "${P}3"
  mount -t ext4 "${P}3" /target
  mount -t vfat "${P}2" /mnt/esp

  copy_system

  say "installing GRUB (BIOS and UEFI)"
  mkdir -p /mnt/esp/boot/grub
  cp /src/medium/boot/vmlinuz /mnt/esp/vmlinuz
  cp /src/medium/boot/initramfs /mnt/esp/initramfs
  write_grub_cfg "" > /mnt/esp/boot/grub/grub.cfg
  grub-editenv /mnt/esp/boot/grub/grubenv create
  grub-install --target=i386-pc --boot-directory=/mnt/esp/boot \
    --modules="part_gpt fat" --no-floppy "$DISK"
  grub-install --target=x86_64-efi --efi-directory=/mnt/esp \
    --boot-directory=/mnt/esp/boot --modules="part_gpt fat" --removable --no-nvram
  [ -f /mnt/esp/EFI/BOOT/BOOTX64.EFI ] || die "grub-install did not create BOOTX64.EFI"
  finish
}

# ================================ mode: alongside ================================================
run_alongside() {
  DISK=$1
  n=${DISK#/dev/}
  case "$n" in *[0-9]) P="${DISK}p" ;; *) P="$DISK" ;; esac
  alongside_check "$DISK" || die "alongside mode is not possible on $DISK: $ALONG_WHY"
  say "alongside check: $ALONG_WHY"
  confirm "A new partition will be created in the free space of $DISK. Existing partitions are not modified."
  UUID=$(cat /proc/sys/kernel/random/uuid)
  ask_keymap

  if [ "$ALONG_KIND" = uefi ]; then
    ESPN=$(esp_number "$DISK")
    BEFORE=$(part_numbers "$DISK")
    say "creating the root partition in the free space (GPT)"
    sgdisk -n 0:0:0 -t 0:8300 -c 0:kyroot "$DISK" >/dev/null
    NEWN=""
    for p in $(part_numbers "$DISK"); do
      printf '%s\n' "$BEFORE" | grep -x "$p" >/dev/null || NEWN=$p
    done
    [ -n "$NEWN" ] || die "could not find the new partition"
    ROOTDEV=${P}${NEWN}
  else
    BEFORE=$(mbr_entries "$DISK")
    say "creating the root partition in the free space (MBR)"
    echo ',,83' | sfdisk -q --append "$DISK"
    ROOTDEV=""
    for e in $(mbr_entries "$DISK"); do
      printf '%s\n' "$BEFORE" | grep -x "$e" >/dev/null || ROOTDEV=$e
    done
    [ -n "$ROOTDEV" ] || die "could not find the new partition"
  fi
  wait_for_node "$ROOTDEV"
  say "new root partition: $ROOTDEV"

  mkfs.ext4 -q -F -L kyroot -U "$UUID" "$ROOTDEV"
  mount -t ext4 "$ROOTDEV" /target
  copy_system

  if [ "$ALONG_KIND" = uefi ]; then
    install_uefi_rootboot "${P}${ESPN}"
  else
    install_bios_rootboot "$DISK"
  fi
  finish
}

# ================================== mode: manual ==================================================
list_partitions() {
  for p in /sys/class/block/*/partition; do
    [ -e "$p" ] || continue
    name=$(basename "$(dirname "$p")")
    case "$name" in loop*|ram*|sr*) continue ;; esac
    [ "$(parent_of "/dev/$name")" = "$MEDIA_PARENT" ] && continue
    size=$(( $(cat "/sys/class/block/$name/size") / 2048 ))
    type=$(blk_attr "/dev/$name" TYPE)
    label=$(blk_attr "/dev/$name" LABEL)
    echo "  $name   ${size} MiB   ${type:--}   ${label:-}"
  done
}

manual_interactive() {
  echo "Manual partitioning"
  echo "  You need one Linux partition for the system (it will be formatted as ext4)."
  if [ "$UEFI" = 1 ]; then
    echo "  UEFI: you also need an EFI System Partition (FAT32). An existing one can be reused."
  else
    echo "  BIOS: on a GPT disk create a 1 MiB 'BIOS boot' partition; on an MBR disk keep 1 MiB free before the first partition."
  fi
  while :; do
    printf "Disk to partition with cfdisk (e.g. sda), or Enter to continue: "
    read -r d
    [ -n "$d" ] || break
    if [ -b "/dev/$d" ] && [ "$d" != "$MEDIA_PARENT" ]; then
      cfdisk "/dev/$d" || true
    else
      echo "/dev/$d is not a usable disk"
    fi
  done
  echo "Partitions:"
  list_partitions
  printf "Partition for the system (root), e.g. sda2: "
  read -r r
  ROOTARG=/dev/$r
  printf "Format it as ext4? ALL its data will be erased [YES/no]: "
  read -r a
  [ "$a" = "no" ] && FMT_ROOT=no
  if [ "$UEFI" = 1 ]; then
    def=""
    for c in $CANDIDATES; do
      e=$(esp_number "/dev/$c") || e=""
      if [ -n "$e" ]; then
        case "$c" in *[0-9]) def="${c}p${e}" ;; *) def="${c}${e}" ;; esac
        break
      fi
    done
    printf "EFI System Partition [%s]: " "$def"
    read -r e
    ESPARG=/dev/${e:-$def}
    printf "Format the EFI System Partition? Only if it is new: other systems' boot files would be lost [no/YES]: "
    read -r a
    [ "$a" = "YES" ] && FMT_ESP=yes
  fi
}

run_manual() {
  if [ -z "$ROOTARG" ]; then
    [ -z "$AUTO" ] || die "manual mode needs --root DEV when run unattended"
    manual_interactive
  fi
  ROOTDEV=$ROOTARG
  ESPDEV=$ESPARG
  [ -b "$ROOTDEV" ] || die "$ROOTDEV is not a block device"
  ROOTDISK=/dev/$(parent_of "$ROOTDEV")
  [ "${ROOTDISK#/dev/}" != "$MEDIA_PARENT" ] || die "$ROOTDEV is on the installation medium"

  if [ "$UEFI" = 1 ]; then
    if [ -z "$ESPDEV" ]; then
      e=$(esp_number "$ROOTDISK") || e=""
      [ -n "$e" ] || die "UEFI install: no EFI System Partition given (--esp DEV) and none found on $ROOTDISK"
      case "${ROOTDISK#/dev/}" in *[0-9]) ESPDEV="${ROOTDISK}p${e}" ;; *) ESPDEV="${ROOTDISK}${e}" ;; esac
    fi
    [ -b "$ESPDEV" ] || die "$ESPDEV is not a block device"
    [ "$ESPDEV" != "$ROOTDEV" ] || die "the root and EFI partitions must be different"
  else
    if is_gpt "$ROOTDISK"; then
      case "$(gpt_info "$ROOTDISK")" in
        *EF02*) ;;
        *) die "BIOS boot on a GPT disk needs a 1 MiB 'BIOS boot' partition (type EF02) on $ROOTDISK" ;;
      esac
    elif is_msdos "$ROOTDISK"; then
      fs=$(mbr_first_start "$ROOTDISK") || fs=""
      [ "${fs:-0}" -ge 2048 ] || die "the first partition of $ROOTDISK starts at sector ${fs:-?}: no room for GRUB after the MBR"
    else
      die "$ROOTDISK has no recognised partition table"
    fi
  fi

  say "mode: manual, root: $ROOTDEV (format: $FMT_ROOT)${ESPDEV:+, EFI System Partition: $ESPDEV (format: $FMT_ESP)}"
  if [ "$FMT_ROOT" = yes ]; then
    confirm "ALL DATA on $ROOTDEV will be ERASED."
  else
    [ "$(blk_attr "$ROOTDEV" TYPE)" = ext4 ] || die "--format-root no needs an existing ext4 filesystem on $ROOTDEV"
    confirm "The system will be copied over the existing files of $ROOTDEV."
  fi
  if [ "$UEFI" = 1 ] && [ "$FMT_ESP" = yes ]; then
    confirm "The EFI System Partition $ESPDEV will be ERASED."
  fi
  ask_keymap

  if [ "$FMT_ROOT" = yes ]; then
    UUID=$(cat /proc/sys/kernel/random/uuid)
    mkfs.ext4 -q -F -L kyroot -U "$UUID" "$ROOTDEV"
  else
    UUID=$(blk_attr "$ROOTDEV" UUID)
  fi
  [ -n "$UUID" ] || die "could not determine the UUID of $ROOTDEV"
  mount -t ext4 "$ROOTDEV" /target
  copy_system

  if [ "$UEFI" = 1 ]; then
    if [ "$FMT_ESP" = yes ]; then mkfs.vfat -F 32 -n KYBOOT "$ESPDEV" >/dev/null; fi
    install_uefi_rootboot "$ESPDEV"
  else
    install_bios_rootboot "$ROOTDISK"
  fi
  finish
}

# ====================================== dispatch ====================================================
if [ -z "$MODE" ]; then
  if [ -n "$AUTO" ]; then
    MODE=wipe
  else
    ALONG_DISKS=""
    for c in $CANDIDATES; do
      alongside_check "/dev/$c" && ALONG_DISKS="$ALONG_DISKS $c"
    done
    echo "Install mode:"
    echo "  1) Automatic - erase a whole disk and install Kyanite"
    [ -n "$ALONG_DISKS" ] && echo "  2) Automatic - install alongside the existing system (keeps Windows and your data)"
    echo "  3) Manual    - partition the disks yourself (cfdisk) and choose the partitions"
    printf "Choice [1]: "
    read -r c
    case "$c" in 2) MODE=alongside ;; 3) MODE=manual ;; *) MODE=wipe ;; esac
  fi
fi
case "$MODE" in wipe|alongside|manual) ;; *) die "unknown mode: $MODE" ;; esac

if [ "$MODE" = manual ]; then
  run_manual
fi

# wipe / alongside: choose the disk
REASONS=""
if [ -n "$AUTO" ] && [ "$AUTO" != "auto" ]; then
  n=${AUTO#/dev/}
elif [ "$AUTO" = "auto" ]; then
  n=""
  for c in $CANDIDATES; do
    if [ "$MODE" = alongside ]; then
      if ! alongside_check "/dev/$c"; then
        REASONS="$REASONS /dev/$c: $ALONG_WHY;"
        continue
      fi
    fi
    n=$c
    break
  done
  [ -n "$n" ] || die "no suitable disk found for mode $MODE.$REASONS"
else
  echo "Available disks:"
  for c in $CANDIDATES; do
    size=$(( $(cat "/sys/block/$c/size") / 2048 ))
    model=$(cat "/sys/block/$c/device/model" 2>/dev/null | tr -s ' ') || model=""
    tag=""
    if alongside_check "/dev/$c"; then tag="  [alongside possible: $ALONG_WHY]"; fi
    echo "  $c   ${size} MiB   $model$tag"
  done
  printf "Target disk (e.g. sda): "
  read -r n
fi
[ -b "/dev/$n" ] || die "/dev/$n is not a disk"
[ "$n" != "$MEDIA_PARENT" ] || die "/dev/$n is the installation medium"

say "mode: $MODE, disk: /dev/$n"
if [ "$MODE" = wipe ]; then
  run_wipe "/dev/$n"
else
  run_alongside "/dev/$n"
fi
