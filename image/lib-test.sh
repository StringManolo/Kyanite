#!/usr/bin/env bash
# QEMU test helpers with screenshots. Use with `source`.
# Everything is written to shots/<case>/ (serial.log, frame-*.png) and shots/results.tsv.
# shellcheck shell=bash

SHOTS_ROOT=${SHOTS_ROOT:-$PWD/shots}
LIMIT=${LIMIT:-150}         # maximum seconds per case
INTERVAL=${INTERVAL:-3}     # seconds between screenshots
mkdir -p "$SHOTS_ROOT"

OVMF=""
for f in /usr/share/ovmf/OVMF.fd /usr/share/OVMF/OVMF.fd; do
  if [ -f "$f" ]; then OVMF=$f; break; fi
done

enable_kvm() {
  echo 'KERNEL=="kvm", GROUP="kvm", MODE="0666", OPTIONS+="static_node=kvm"' \
    | sudo tee /etc/udev/rules.d/99-kvm4all.rules >/dev/null
  sudo udevadm control --reload-rules
  sudo udevadm trigger --name-match=kvm || true
  if [ -w /dev/kvm ]; then echo "KVM available"; else echo "KVM not available: falling back to emulation (slower)"; fi
}

# fw_args bios|uefi -> QEMU arguments for the firmware
fw_args() {
  if [ "$1" = uefi ]; then echo "-bios $OVMF"; fi
  return 0
}

# wait_node DEVICE -> waits for a block device node to appear
wait_node() {
  local i
  for i in 1 2 3 4 5 6 7 8 9 10; do [ -b "$1" ] && return 0; sleep 1; done
  return 1
}

# set_serial_entry IMAGE [PARTITION_NUMBER] [GRUBENV_PATH]
# Makes the "serial console" entry the default in the image's grubenv.
# Disk image / wipe install: grubenv is in partition 2 (ESP), path boot/grub/grubenv.
# Alongside install: grubenv is in the new root partition (3), path boot/grub/grubenv.
set_serial_entry() {
  local img=$1 part=${2:-2} env=${3:-boot/grub/grubenv} loop
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p${part}"
  mkdir -p "$PWD/mnt/esp"
  sudo mount "${loop}p${part}" "$PWD/mnt/esp"
  sudo grub-editenv "$PWD/mnt/esp/$env" set saved_entry=serial
  sudo umount "$PWD/mnt/esp"
  sudo losetup -d "$loop"
}

# matches LOG REGEX
# Only counts output that comes AFTER the hand-over to Kyanite, so firmware/GRUB/kernel text cannot make a
# boot look successful:
#   - disk: after "Mounting root: ok." (Alpine initramfs)
#   - live: after "KYANITE-LIVE: switch_root"
# OVMF copies the GRUB menu ("Kyanite", ...) to the serial console WITHOUT newlines, so it ends up on the
# same line as "Linux version": the text has to be cut by position, not by line.
# With EXPECT_RAW=1 (e.g. the installer, which never reaches Kyanite) it is enough that it appears after "Linux version".
# Our own marker lines (KYANITE-LIVE/INSTALL/BOOT) are ignored unless EXPECT_RAW=1.
# (grep without -q on purpose: with pipefail a grep -q that exits early causes SIGPIPE and false negatives.)
matches() {
  if [ "${EXPECT_RAW:-0}" = 1 ]; then
    LC_ALL=C awk 'f{print; next} /Linux version/{sub(/^.*Linux version/, "Linux version"); f=1; print}' "$1" \
      | grep -aiE "$2" >/dev/null
  else
    LC_ALL=C awk 'f{print; next} match($0, /Mounting root: ok\.|KYANITE-LIVE: switch_root/){f=1; print substr($0, RSTART + RLENGTH)}' "$1" \
      | grep -av 'KYANITE-\(LIVE\|INSTALL\|BOOT\)' | grep -aiE "$2" >/dev/null
  fi
}

detect_stages() {
  local log=$1 s=()
  grep -qa "Linux version" "$log" && s+=(kernel)
  grep -qaE "EXT4-fs \(.*\): mounted" "$log" && s+=(root-ext4)
  grep -qa "KYANITE-LIVE: switch_root" "$log" && s+=(live-init)
  grep -qa "KYANITE-BOOT: coldplug done" "$log" && s+=(hw-setup)
  grep -qa "KYANITE-SMOKE: start" "$log" && s+=(smoke-script)
  grep -qa "KYANITE-INSTALL-DONE" "$log" && s+=(installer)
  local IFS=,
  echo "${s[*]:-none}"
}

record() { printf '%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" >> "$SHOTS_ROOT/results.tsv"; }
record_skip() { record "$1" skip "$2" "-"; }

# run_vm NAME EXPECTED_REGEX WAIT_FOR_POWEROFF(0|1) [QEMU arguments...]
# Returns 0 if the serial output matches the regex.
# Optional variables: LIMIT, VM_MEM, VM_CPU (default max), VM_ACCEL (default kvm:tcg),
#                     VM_SOFT (1 = report a warning instead of a failure), EXPECT_RAW, NEED_FW (1 = fw=0 is a failure)
run_vm() {
  local name=$1 expect=$2 waitexit=$3; shift 3
  local dir="$SHOTS_ROOT/$name" log="$SHOTS_ROOT/$name/serial.log"
  local ok=0 reason="timed out" qp sp i stages
  local failre='Kernel panic|KYANITE-(LIVE|INSTALL): ERROR|KYANITE-SMOKE summary .*(vim=[1-9]|net=none|kbd=none)'
  [ "${NEED_FW:-0}" = 1 ] && failre="$failre|KYANITE-SMOKE summary .*fw=0 "
  rm -rf "$dir"; mkdir -p "$dir"; : > "$log"
  echo "::group::$name"

  qemu-system-x86_64 -machine "q35,accel=${VM_ACCEL:-kvm:tcg}" -cpu "${VM_CPU:-max}" -m "${VM_MEM:-1024}" -smp 2 \
    -display none -serial "file:$log" -qmp "unix:$dir/qmp.sock,server=on,wait=off" \
    -net none -no-reboot "$@" &
  qp=$!
  python3 image/qmp-shots.py "$dir/qmp.sock" "$dir" "$INTERVAL" &
  sp=$!

  for ((i = 0; i < LIMIT / 2; i++)); do
    sleep 2
    if matches "$log" "$expect"; then ok=1; reason="expected output found"; break; fi
    if grep -qaE "$failre" "$log"; then reason="error during boot"; break; fi
    if ! kill -0 "$qp" 2>/dev/null; then reason="QEMU exited without the expected output"; break; fi
  done
  if [ "$ok" = 1 ] && [ "$waitexit" = 1 ]; then
    for i in $(seq 1 20); do kill -0 "$qp" 2>/dev/null || break; sleep 2; done
  fi
  kill "$qp" 2>/dev/null || true; wait "$qp" 2>/dev/null || true
  kill "$sp" 2>/dev/null || true; wait "$sp" 2>/dev/null || true

  stages=$(detect_stages "$log")
  if [ "$ok" = 1 ]; then
    echo "OK   $name ($reason; stages: $stages)"
    record "$name" ok "$reason" "$stages"
  else
    local status=fail
    [ "${VM_SOFT:-0}" = 1 ] && status=warn   # informational: does not fail the verdict
    echo "${status^^} $name ($reason; stages: $stages)"
    tail -n 30 "$log" | tr -cd '\11\12\40-\176'
    record "$name" "$status" "$reason" "$stages"
  fi
  echo "::endgroup::"
  [ "$ok" = 1 ]
}

# make_smoke_image SOURCE DEST
# Copy of the disk image with /kyanite-smoke.sh in the rootfs and a GRUB entry that uses it as init.
# It checks, without going through kyanite-init, that on this VM:
#   - vim runs (CPU instruction compatibility)         - a network driver gets loaded (coldplug)
#   - a USB keyboard driver gets loaded (coldplug)     - firmware files are present (hardware variant)
#   - the console keymap loads (informational)
# Only the copy is touched; the final image is unchanged.
make_smoke_image() {
  local src=$1 dst=$2 loop uuidarg
  cp --sparse=always "$src" "$dst"
  loop=$(sudo losetup --find --show --partscan "$dst")
  sudo udevadm settle || true
  wait_node "${loop}p3"
  mkdir -p "$PWD/mnt/esp" "$PWD/mnt/root"
  sudo mount "${loop}p3" "$PWD/mnt/root"
  sudo mount "${loop}p2" "$PWD/mnt/esp"

  sudo tee "$PWD/mnt/root/kyanite-smoke.sh" >/dev/null <<'EOF'
#!/bin/brush
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
echo "KYANITE-SMOKE: start"
/usr/lib/kyanite/hw/hw-setup

vim=127
first=
if command -v vim >/dev/null 2>&1; then
  vim --version > /tmp/kyanite-vim.txt 2>&1
  vim=$?
  read -r first < /tmp/kyanite-vim.txt
fi

net=none
for d in /sys/class/net/*; do
  n=${d##*/}
  [ "$n" = lo ] || [ ! -e "$d" ] || net=ok
done

kbd=none
while read -r line; do
  case "$line" in
    *"USB Keyboard"*) kbd=ok ;;
  esac
done < /proc/bus/input/devices

fw=0
for f in /lib/firmware/*; do
  [ -e "$f" ] && fw=$((fw + 1))
done

km=none
[ -r /run/kyanite/keymap ] && read -r km < /run/kyanite/keymap

echo "KYANITE-SMOKE summary vim=$vim net=$net kbd=$kbd fw=$fw keymap=$km ($first)"
read -r _
EOF
  sudo chmod 755 "$PWD/mnt/root/kyanite-smoke.sh"

  uuidarg=$(grep -o 'root=UUID=[^ ]*' "$PWD/mnt/esp/boot/grub/grub.cfg" | head -n1)
  sudo tee -a "$PWD/mnt/esp/boot/grub/grub.cfg" >/dev/null <<EOF

menuentry "Kyanite (smoke)" --id smoke {
  linux /vmlinuz $uuidarg rootfstype=ext4 rw rootflags=rw init=/kyanite-smoke.sh console=tty0 console=ttyS0,115200 kyanite.keymap=es
  initrd /initramfs
}
EOF
  sudo grub-editenv "$PWD/mnt/esp/boot/grub/grubenv" set saved_entry=smoke
  sudo umount "$PWD/mnt/esp" "$PWD/mnt/root"
  sudo losetup -d "$loop"
}

# make_fake_windows_disk IMAGE
# 8 GiB GPT disk that imitates a Windows PC: 100 MiB EFI System Partition (FAT32) with a fake
# EFI/Microsoft/Boot/bootmgfw.efi, a 1 GiB NTFS partition with a file, and the rest left free.
# Checksums of what must stay untouched are saved next to the image (IMAGE.sha.*, IMAGE.pt0).
make_fake_windows_disk() {
  local img=$1 loop
  rm -f "$img" "$img".sha.* "$img".pt0
  truncate -s 8G "$img"
  sgdisk -o "$img" >/dev/null
  sgdisk -n 1:2048:+100M -t 1:ef00 -c 1:"EFI system partition" "$img" >/dev/null
  sgdisk -n 2:0:+1G -t 2:0700 -c 2:"Basic data partition" "$img" >/dev/null
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p2"
  sudo mkfs.vfat -F 32 -n SYSTEM "${loop}p1" >/dev/null
  sudo mkfs.ntfs -Q -L Windows "${loop}p2" >/dev/null 2>&1

  mkdir -p "$PWD/mnt/esp" "$PWD/mnt/ntfs"
  sudo mount "${loop}p1" "$PWD/mnt/esp"
  sudo mkdir -p "$PWD/mnt/esp/EFI/Microsoft/Boot"
  head -c 65536 /dev/urandom | sudo tee "$PWD/mnt/esp/EFI/Microsoft/Boot/bootmgfw.efi" >/dev/null
  sudo sha256sum "$PWD/mnt/esp/EFI/Microsoft/Boot/bootmgfw.efi" | cut -d' ' -f1 > "$img.sha.bootmgfw"
  sudo umount "$PWD/mnt/esp"

  sudo mount -t ntfs3 "${loop}p2" "$PWD/mnt/ntfs" 2>/dev/null || sudo mount -t ntfs-3g "${loop}p2" "$PWD/mnt/ntfs"
  echo "fake windows data" | sudo tee "$PWD/mnt/ntfs/pagefile.sys" >/dev/null
  sudo umount "$PWD/mnt/ntfs"

  sudo dd if="${loop}p2" bs=1M status=none | sha256sum | cut -d' ' -f1 > "$img.sha.ntfs"
  sgdisk -p "$img" | awk '$1 ~ /^[0-9]+$/' > "$img.pt0"
  sudo losetup -d "$loop"
}

# verify_alongside IMAGE
# Checks, after an "alongside" install, that Windows' data is untouched and Kyanite is in place.
# Records one result line ("alongside-verify").
verify_alongside() {
  local img=$1 loop ok=1 details="" new
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p3"
  mkdir -p "$PWD/mnt/esp" "$PWD/mnt/root"

  new=$(sudo dd if="${loop}p2" bs=1M status=none | sha256sum | cut -d' ' -f1)
  if [ "$new" = "$(cat "$img.sha.ntfs")" ]; then details="$details ntfs-intact"; else details="$details NTFS-CHANGED"; ok=0; fi

  if [ "$(sgdisk -p "$img" | awk '$1 ~ /^[0-9]+$/' | head -n 2)" = "$(head -n 2 "$img.pt0")" ]; then
    details="$details windows-partitions-unchanged"
  else
    details="$details PARTITIONS-CHANGED"; ok=0
  fi

  sudo mount -o ro "${loop}p1" "$PWD/mnt/esp"
  new=$(sudo sha256sum "$PWD/mnt/esp/EFI/Microsoft/Boot/bootmgfw.efi" | cut -d' ' -f1)
  if [ "$new" = "$(cat "$img.sha.bootmgfw")" ]; then details="$details bootmgfw-intact"; else details="$details BOOTMGFW-CHANGED"; ok=0; fi
  if sudo test -f "$PWD/mnt/esp/EFI/Kyanite/grubx64.efi"; then details="$details esp-has-kyanite"; else details="$details NO-EFI/Kyanite"; ok=0; fi
  if sudo test -f "$PWD/mnt/esp/EFI/BOOT/BOOTX64.EFI"; then details="$details fallback-present"; else details="$details NO-FALLBACK"; ok=0; fi
  sudo umount "$PWD/mnt/esp"

  sudo mount -o ro "${loop}p3" "$PWD/mnt/root"
  if sudo test -f "$PWD/mnt/root/boot/vmlinuz" && sudo test -f "$PWD/mnt/root/boot/initramfs"; then
    details="$details kernel-in-root"; else details="$details NO-KERNEL-IN-ROOT"; ok=0
  fi
  if sudo grep -q 'menuentry "Windows"' "$PWD/mnt/root/boot/grub/grub.cfg"; then
    details="$details grub-has-windows-entry"; else details="$details NO-WINDOWS-ENTRY"; ok=0
  fi
  sudo umount "$PWD/mnt/root"
  sudo losetup -d "$loop"

  if [ "$ok" = 1 ]; then
    echo "OK   alongside-verify ($details)"
    record alongside-verify ok "Windows data untouched, Kyanite installed alongside" "$details"
  else
    echo "FAIL alongside-verify ($details)"
    record alongside-verify fail "alongside install checks failed" "$details"
  fi
  [ "$ok" = 1 ]
}

# ---------------------------------------------------------------------------------------------------
# Installer test fixtures. Each make_* builds a disk image in the state a user would start from, and
# each verify_* checks the result after an unattended install and records one line in results.tsv.
# ---------------------------------------------------------------------------------------------------

# Prints the partition entries of an image without the device name (so they can be compared)
mbr_entries_of() { sfdisk -d "$1" 2>/dev/null | grep ' : start=' | sed 's/^[^ ]* //'; }

sha_part()  { sudo dd if="$1" bs=1M status=none | sha256sum | cut -d' ' -f1; }

# make_fake_windows_mbr_disk IMAGE
# 8 GiB MBR disk that imitates a legacy-BIOS Windows PC: 100 MiB "System Reserved" NTFS partition (active,
# with a fake /bootmgr) and a 1 GiB NTFS partition; the rest is free.
make_fake_windows_mbr_disk() {
  local img=$1 loop
  rm -f "$img" "$img".sha.* "$img".pt0
  truncate -s 8G "$img"
  sfdisk -q "$img" <<'EOF'
label: dos
unit: sectors

start=2048, size=204800, type=7, bootable
start=206848, size=2097152, type=7
EOF
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p2"
  sudo mkfs.ntfs -Q -L "System Reserved" "${loop}p1" >/dev/null 2>&1
  sudo mkfs.ntfs -Q -L Windows "${loop}p2" >/dev/null 2>&1
  mkdir -p "$PWD/mnt/ntfs"
  sudo mount -t ntfs3 "${loop}p1" "$PWD/mnt/ntfs" 2>/dev/null || sudo mount -t ntfs-3g "${loop}p1" "$PWD/mnt/ntfs"
  head -c 65536 /dev/urandom | sudo tee "$PWD/mnt/ntfs/bootmgr" >/dev/null
  sudo umount "$PWD/mnt/ntfs"
  sudo mount -t ntfs3 "${loop}p2" "$PWD/mnt/ntfs" 2>/dev/null || sudo mount -t ntfs-3g "${loop}p2" "$PWD/mnt/ntfs"
  echo "fake windows data" | sudo tee "$PWD/mnt/ntfs/pagefile.sys" >/dev/null
  sudo umount "$PWD/mnt/ntfs"
  sha_part "${loop}p1" > "$img.sha.p1"
  sha_part "${loop}p2" > "$img.sha.p2"
  mbr_entries_of "$img" > "$img.pt0"
  sudo losetup -d "$loop"
}

# verify_alongside_bios IMAGE  (records "alongside-bios-verify")
verify_alongside_bios() {
  local img=$1 loop ok=1 details="" cur
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p3"
  mkdir -p "$PWD/mnt/root"

  if [ "$(sha_part "${loop}p1")" = "$(cat "$img.sha.p1")" ]; then details="$details system-reserved-intact"; else details="$details SYSTEM-RESERVED-CHANGED"; ok=0; fi
  if [ "$(sha_part "${loop}p2")" = "$(cat "$img.sha.p2")" ]; then details="$details windows-intact"; else details="$details WINDOWS-CHANGED"; ok=0; fi
  cur=$(mbr_entries_of "$img" | head -n 2)
  if [ "$cur" = "$(head -n 2 "$img.pt0")" ]; then details="$details entries-unchanged"; else details="$details ENTRIES-CHANGED"; ok=0; fi
  if mbr_entries_of "$img" | grep 'type=83' >/dev/null; then details="$details linux-partition"; else details="$details NO-LINUX-PARTITION"; ok=0; fi
  if sudo dd if="$img" bs=512 count=1 status=none | grep -a GRUB >/dev/null; then details="$details grub-in-mbr"; else details="$details NO-GRUB-IN-MBR"; ok=0; fi

  sudo mount -o ro "${loop}p3" "$PWD/mnt/root"
  if sudo test -f "$PWD/mnt/root/boot/vmlinuz" && sudo test -f "$PWD/mnt/root/boot/initramfs"; then
    details="$details kernel-in-root"; else details="$details NO-KERNEL-IN-ROOT"; ok=0
  fi
  if sudo grep 'menuentry "Windows"' "$PWD/mnt/root/boot/grub/grub.cfg" >/dev/null \
     && sudo grep '/bootmgr' "$PWD/mnt/root/boot/grub/grub.cfg" >/dev/null; then
    details="$details windows-entry"; else details="$details NO-WINDOWS-ENTRY"; ok=0
  fi
  sudo umount "$PWD/mnt/root"
  sudo losetup -d "$loop"

  if [ "$ok" = 1 ]; then
    echo "OK   alongside-bios-verify ($details)"
    record alongside-bios-verify ok "Windows (BIOS/MBR) data untouched, Kyanite installed alongside" "$details"
  else
    echo "FAIL alongside-bios-verify ($details)"
    record alongside-bios-verify fail "BIOS alongside install checks failed" "$details"
  fi
  [ "$ok" = 1 ]
}

# make_manual_uefi_disk IMAGE
# GPT disk with an existing EFI System Partition (holding a marker file that must survive) and an empty
# 3 GiB partition for the system. Used with: --mode manual --root /dev/sda2 --esp /dev/sda1
make_manual_uefi_disk() {
  local img=$1 loop
  rm -f "$img" "$img".sha.*
  truncate -s 8G "$img"
  sgdisk -o "$img" >/dev/null
  sgdisk -n 1:2048:+100M -t 1:ef00 -c 1:"EFI system partition" "$img" >/dev/null
  sgdisk -n 2:0:+3G -t 2:8300 -c 2:"root" "$img" >/dev/null
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p2"
  sudo mkfs.vfat -F 32 -n SYSTEM "${loop}p1" >/dev/null
  mkdir -p "$PWD/mnt/esp"
  sudo mount "${loop}p1" "$PWD/mnt/esp"
  sudo mkdir -p "$PWD/mnt/esp/EFI/Marker"
  echo "keep me" | sudo tee "$PWD/mnt/esp/EFI/Marker/keep.txt" >/dev/null
  sudo sha256sum "$PWD/mnt/esp/EFI/Marker/keep.txt" | cut -d' ' -f1 > "$img.sha.marker"
  sudo umount "$PWD/mnt/esp"
  sudo losetup -d "$loop"
}

# verify_manual_uefi IMAGE  (records "manual-uefi-verify")
verify_manual_uefi() {
  local img=$1 loop ok=1 details="" new
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p2"
  mkdir -p "$PWD/mnt/esp" "$PWD/mnt/root"

  sudo mount -o ro "${loop}p1" "$PWD/mnt/esp"
  new=$(sudo sha256sum "$PWD/mnt/esp/EFI/Marker/keep.txt" | cut -d' ' -f1)
  if [ "$new" = "$(cat "$img.sha.marker")" ]; then details="$details esp-marker-intact"; else details="$details ESP-MARKER-CHANGED"; ok=0; fi
  if sudo test -f "$PWD/mnt/esp/EFI/Kyanite/grubx64.efi"; then details="$details esp-has-kyanite"; else details="$details NO-EFI/Kyanite"; ok=0; fi
  sudo umount "$PWD/mnt/esp"

  sudo mount -o ro "${loop}p2" "$PWD/mnt/root"
  if sudo test -f "$PWD/mnt/root/boot/vmlinuz" && sudo test -f "$PWD/mnt/root/boot/grub/grub.cfg"; then
    details="$details kernel-and-grubcfg-in-root"; else details="$details NO-KERNEL-IN-ROOT"; ok=0
  fi
  sudo umount "$PWD/mnt/root"
  sudo losetup -d "$loop"

  if [ "$ok" = 1 ]; then
    echo "OK   manual-uefi-verify ($details)"
    record manual-uefi-verify ok "manual install: chosen partitions used, existing ESP content kept" "$details"
  else
    echo "FAIL manual-uefi-verify ($details)"
    record manual-uefi-verify fail "manual UEFI install checks failed" "$details"
  fi
  [ "$ok" = 1 ]
}

# make_manual_bios_disk IMAGE
# MBR disk with a single empty 3 GiB Linux partition (and free space). Used with: --mode manual --root /dev/sda1
make_manual_bios_disk() {
  local img=$1
  rm -f "$img" "$img".sha.*
  truncate -s 8G "$img"
  sfdisk -q "$img" <<'EOF'
label: dos
unit: sectors

start=2048, size=6291456, type=83
EOF
}

# verify_manual_bios IMAGE  (records "manual-bios-verify")
verify_manual_bios() {
  local img=$1 loop ok=1 details=""
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  wait_node "${loop}p1"
  mkdir -p "$PWD/mnt/root"
  if sudo dd if="$img" bs=512 count=1 status=none | grep -a GRUB >/dev/null; then details="$details grub-in-mbr"; else details="$details NO-GRUB-IN-MBR"; ok=0; fi
  sudo mount -o ro "${loop}p1" "$PWD/mnt/root"
  if sudo test -f "$PWD/mnt/root/boot/vmlinuz" && sudo test -f "$PWD/mnt/root/boot/grub/grub.cfg"; then
    details="$details kernel-and-grubcfg-in-root"; else details="$details NO-KERNEL-IN-ROOT"; ok=0
  fi
  sudo umount "$PWD/mnt/root"
  sudo losetup -d "$loop"

  if [ "$ok" = 1 ]; then
    echo "OK   manual-bios-verify ($details)"
    record manual-bios-verify ok "manual install on a BIOS/MBR disk: GRUB in the MBR, system on the chosen partition" "$details"
  else
    echo "FAIL manual-bios-verify ($details)"
    record manual-bios-verify fail "manual BIOS install checks failed" "$details"
  fi
  [ "$ok" = 1 ]
}
