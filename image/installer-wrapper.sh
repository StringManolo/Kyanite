#!/bin/sh
# Prepares the chroot of the installer layer (Alpine/musl) and runs kyanite-install.sh inside it.
# Runs with busybox ash, from the initramfs (autoinstall) or from the live system (interactive).
# Variables: K_INSTALLER, K_ROOTFS, K_MEDIUM (defaults: the live paths under /live/*).

K_INSTALLER=${K_INSTALLER:-/live/installer}
K_ROOTFS=${K_ROOTFS:-/live/rootfs}
K_MEDIUM=${K_MEDIUM:-/live/medium}
B=$K_INSTALLER/bin/busybox.static

if [ ! -x "$B" ]; then
  echo "KYANITE-INSTALL: ERROR: installer layer not found ($B)" >&2
  exit 1
fi

"$B" mount -o bind /dev "$K_INSTALLER/dev"
"$B" mount -t proc proc "$K_INSTALLER/proc"
"$B" mount -t sysfs sysfs "$K_INSTALLER/sys"
"$B" mount -t tmpfs tmpfs "$K_INSTALLER/tmp"
"$B" mount -t tmpfs tmpfs "$K_INSTALLER/run"
"$B" mount -o bind "$K_ROOTFS" "$K_INSTALLER/src/rootfs"
"$B" mount -o bind "$K_MEDIUM" "$K_INSTALLER/src/medium"
[ -d /lib/modules ] && "$B" mount -o bind /lib/modules "$K_INSTALLER/lib/modules"

"$B" chroot "$K_INSTALLER" /bin/sh /usr/local/bin/kyanite-install.sh "$@"
rc=$?

for m in lib/modules src/medium src/rootfs run tmp sys proc dev; do
  "$B" umount "$K_INSTALLER/$m" 2>/dev/null
done
exit $rc
