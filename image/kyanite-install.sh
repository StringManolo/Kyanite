#!/bin/sh
# Kyanite installer. Runs inside the Alpine installer layer (chroot), launched by installer-wrapper.sh.
#
# Usage: kyanite-install.sh [--auto auto|/dev/sdX] [--mode wipe|alongside] [--keymap LAYOUT]
#
#   wipe       Erase the whole disk. GPT: bios_grub + EFI System Partition + ext4 root.
#              Bootable in BIOS and UEFI. Kernel and initramfs live in the ESP.      (default)
#   alongside  UEFI only. Keep the existing system (for example Windows): create ONE new ext4 root partition
#              in the largest free space, REUSE the existing EFI System Partition (only EFI/Kyanite/ and,
#              if missing, EFI/BOOT/BOOTX64.EFI are added), keep kernel/initramfs in the root partition,
#              and add a GRUB entry that chainloads Windows. Existing partitions are never modified.
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

AUTO=""
MODE=""
KEYMAP=""
while [ $# -gt 0 ]; do
  case "$1" in
    --auto)   AUTO="${2:-auto}"; shift; [ $# -gt 0 ] && shift ;;
    --mode)   MODE="${2:-}";     shift; [ $# -gt 0 ] && shift ;;
    --keymap) KEYMAP="${2:-}";   shift; [ $# -gt 0 ] && shift ;;
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

# ---- helpers ------------------------------------------------------------------------------------
parent_of() {
  n=${1#/dev/}
  if [ -e "/sys/class/block/$n/partition" ]; then
    basename "$(dirname "$(readlink -f "/sys/class/block/$n")")"
  else
    echo "$n"
  fi
}

is_gpt()        { sgdisk -p "$1" 2>/dev/null | grep -q 'GPT: present'; }
esp_number()    { sgdisk -p "$1" 2>/dev/null | awk '$1 ~ /^[0-9]+$/ && toupper($6) == "EF00" {print $1; exit}'; }
part_numbers()  { sgdisk -p "$1" 2>/dev/null | awk '$1 ~ /^[0-9]+$/ {print $1}'; }

# Largest free block of the disk, in MiB
free_mib() {
  d=$1
  n=${d#/dev/}
  lbs=$(cat "/sys/block/$n/queue/logical_block_size" 2>/dev/null || echo 512)
  first=$(sgdisk -F "$d" 2>/dev/null | tail -n 1)
  last=$(sgdisk -E "$d" 2>/dev/null | tail -n 1)
  case "$first$last" in ''|*[!0-9]*) echo 0; return ;; esac
  if [ "$last" -ge "$first" ]; then
    echo $(( (last - first + 1) * lbs / 1048576 ))
  else
    echo 0
  fi
}

# alongside mode needs: UEFI boot, GPT disk, an existing EFI System Partition and >= 2 GiB free
alongside_ok() {
  [ "$UEFI" = 1 ] || return 1
  is_gpt "$1" || return 1
  [ -n "$(esp_number "$1")" ] || return 1
  [ "$(free_mib "$1")" -ge 2048 ]
}

wait_for_node() {
  i=0
  while [ ! -b "$1" ] && [ "$i" -lt 15 ]; do
    sleep 1; i=$((i + 1)); mdev -s 2>/dev/null || true
  done
  [ -b "$1" ] || die "device node $1 did not appear"
}

# ---- which disk is the installation medium (never offer it) ----------------------------------
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

# ---- choose the disk -----------------------------------------------------------------------------
if [ -n "$AUTO" ] && [ "$AUTO" != "auto" ]; then
  n=${AUTO#/dev/}
elif [ "$AUTO" = "auto" ]; then
  n=""
  for c in $CANDIDATES; do
    if [ "$MODE" = alongside ]; then
      alongside_ok "/dev/$c" || continue
    fi
    n=$c
    break
  done
  [ -n "$n" ] || die "no suitable disk found${MODE:+ for mode $MODE}"
else
  echo "Available disks:"
  for c in $CANDIDATES; do
    size=$(( $(cat "/sys/block/$c/size") / 2048 ))
    model=$(cat "/sys/block/$c/device/model" 2>/dev/null | tr -s ' ' || true)
    tag=""
    alongside_ok "/dev/$c" && tag="  [can be installed alongside an existing system]"
    echo "  $c   ${size} MiB   $model$tag"
  done
  printf "Target disk (e.g. sda): "
  read -r n
fi
[ -b "/dev/$n" ] || die "/dev/$n is not a disk"
[ "$n" != "$MEDIA_PARENT" ] || die "/dev/$n is the installation medium"
DISK=/dev/$n

# ---- choose the mode ---------------------------------------------------------------------------------
if [ -z "$MODE" ]; then
  MODE=wipe
  if [ -z "$AUTO" ] && alongside_ok "$DISK"; then
    echo "Install mode:"
    echo "  1) Erase the whole disk and install Kyanite"
    echo "  2) Install alongside the existing system (keeps Windows and your data)"
    printf "Choice [1]: "
    read -r c
    [ "$c" = "2" ] && MODE=alongside
  fi
fi
case "$MODE" in wipe|alongside) ;; *) die "unknown mode: $MODE" ;; esac
if [ "$MODE" = alongside ]; then
  alongside_ok "$DISK" || die "alongside mode needs: UEFI boot, a GPT disk, an existing EFI System Partition and at least 2 GiB of free space"
fi

# ---- keyboard layout ---------------------------------------------------------------------------------------
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

# ---- confirmation (interactive only) -----------------------------------------------------------------------
case "$n" in *[0-9]) P="${DISK}p" ;; *) P="$DISK" ;; esac
if [ "$MODE" = wipe ]; then
  say "mode: wipe, disk: $DISK"
  if [ -z "$AUTO" ]; then
    printf "ALL DATA on %s will be ERASED. Type YES to continue: " "$DISK"
    read -r a
    [ "$a" = "YES" ] || die "cancelled"
  fi
else
  ESPN=$(esp_number "$DISK")
  say "mode: alongside, disk: $DISK, reusing EFI System Partition ${P}${ESPN}"
  if [ -z "$AUTO" ]; then
    echo "A new partition will be created in the free space of $DISK ($(free_mib "$DISK") MiB)."
    echo "Existing partitions are not modified; the EFI System Partition ${P}${ESPN} is reused."
    printf "Type YES to continue: "
    read -r a
    [ "$a" = "YES" ] || die "cancelled"
  fi
fi

UUID=$(cat /proc/sys/kernel/random/uuid)
KINIT=/sbin/init
[ -x /src/rootfs/usr/lib/kyanite/hw/boot-init ] && KINIT=/usr/lib/kyanite/hw/boot-init

# GRUB config from the shared template. $1 = "" (kernel in the ESP) or "/boot" (kernel in the root partition)
write_grub_cfg() {
  sed -e "s#@ROOT_UUID@#$UUID#g" -e "s#@KINIT@#$KINIT#g" -e "s#@KPREFIX@#$1#g" \
    /usr/local/share/kyanite/grub.cfg.in
}

copy_system() {
  say "copying the system (this can take a while)"
  tar -C /src/rootfs -cf - . | tar -C /target -xpf -
  if [ -n "$KEYMAP" ] && [ "$KEYMAP" != us ]; then
    mkdir -p /target/etc/kyanite
    echo "$KEYMAP" > /target/etc/kyanite/keymap
  fi
}

mkdir -p /target /mnt/esp

# ================================ wipe mode ==========================================================
if [ "$MODE" = wipe ]; then
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

# =============================== alongside mode ======================================================
else
  BEFORE=$(part_numbers "$DISK")
  say "creating the root partition in the free space"
  sgdisk -n 0:0:0 -t 0:8300 -c 0:kyroot "$DISK" >/dev/null
  NEWN=""
  for p in $(part_numbers "$DISK"); do
    echo "$BEFORE" | grep -qx "$p" || NEWN=$p
  done
  [ -n "$NEWN" ] || die "could not find the new partition"
  ROOTDEV=${P}${NEWN}
  ESPDEV=${P}${ESPN}
  wait_for_node "$ROOTDEV"
  say "new root partition: $ROOTDEV"

  mkfs.ext4 -q -F -L kyroot -U "$UUID" "$ROOTDEV"
  mount -t ext4 "$ROOTDEV" /target
  # iocharset=iso8859-1 keeps FAT lookups case-insensitive, so we never create "EFI/BOOT" next to Windows' "EFI/Boot"
  mount -t vfat -o iocharset=iso8859-1,shortname=mixed "$ESPDEV" /mnt/esp

  set -- $(df -k /mnt/esp | tail -n 1)
  [ "$4" -ge 8192 ] || die "the EFI System Partition has less than 8 MiB free"

  copy_system

  say "installing GRUB (UEFI, alongside the existing system)"
  mkdir -p /target/boot/grub
  cp /src/medium/boot/vmlinuz /target/boot/vmlinuz
  cp /src/medium/boot/initramfs /target/boot/initramfs
  write_grub_cfg "/boot" | sed 's/^set timeout=3$/set timeout=5/' > /target/boot/grub/grub.cfg
  if find /mnt/esp/EFI -maxdepth 3 -iname bootmgfw.efi 2>/dev/null | grep -q .; then
    say "Windows Boot Manager found, adding a GRUB entry for it"
    cat >> /target/boot/grub/grub.cfg <<'EOF'

menuentry "Windows" --id windows {
  insmod part_gpt
  insmod fat
  insmod chain
  search --no-floppy --file --set=root /EFI/Microsoft/Boot/bootmgfw.efi
  chainloader /EFI/Microsoft/Boot/bootmgfw.efi
}
EOF
  fi
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

  # UEFI boot entry (best effort: the fallback path above already makes the disk bootable)
  if [ "$UEFI" = 1 ] && command -v efibootmgr >/dev/null 2>&1; then
    if efibootmgr --create --disk "$DISK" --part "$ESPN" --loader '\EFI\Kyanite\grubx64.efi' \
        --label "Kyanite" >/dev/null 2>&1; then
      say "UEFI boot entry registered"
    else
      say "warning: could not register a UEFI boot entry (the fallback path is installed)"
    fi
  fi
fi

sync
umount /mnt/esp
umount /target
sync

say "installation finished on $DISK (mode: $MODE)"
[ -n "$AUTO" ] && echo "KYANITE-INSTALL-DONE"
exit 0
