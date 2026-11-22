#!/bin/sh
# Prepara el chroot de la capa del instalador (Alpine/musl) y lanza kyanite-install.sh dentro.
# Se ejecuta con busybox ash, desde el initramfs (autoinstall) o desde el live (interactivo).
# Variables: K_INSTALLER, K_ROOTFS, K_MEDIUM (por defecto, las rutas del live en /live/*).

K_INSTALLER=${K_INSTALLER:-/live/installer}
K_ROOTFS=${K_ROOTFS:-/live/rootfs}
K_MEDIUM=${K_MEDIUM:-/live/medium}
B=$K_INSTALLER/bin/busybox.static

if [ ! -x "$B" ]; then
  echo "KYANITE-INSTALL: ERROR: falta la capa del instalador ($B)" >&2
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
