#!/bin/sh
# Instalador de Kyanite. Corre dentro de la capa Alpine (chroot), lanzado por installer-wrapper.sh.
# Uso: kyanite-install.sh [--auto auto|/dev/sdX]
#   sin --auto : interactivo (elige disco y confirma)
#   --auto auto: elige el primer disco que no sea el medio de instalacion (para el CI)
# Rutas en el chroot: /src/rootfs (arbol del sistema), /src/medium (ISO: boot/vmlinuz, boot/initramfs)
set -eu
(set -o pipefail) 2>/dev/null && set -o pipefail

trap 'rc=$?; [ "$rc" -eq 0 ] || echo "KYANITE-INSTALL: ERROR: salida con codigo $rc" >&2' EXIT

say() { echo "KYANITE-INSTALL: $*"; }
die() { echo "KYANITE-INSTALL: ERROR: $*" >&2; exit 1; }

AUTO=""
while [ $# -gt 0 ]; do
  case "$1" in
    --auto) AUTO="${2:-auto}"; shift; [ $# -gt 0 ] && shift ;;
    *) shift ;;
  esac
done

modprobe -ab vfat ext4 nls_cp437 nls_iso8859_1 2>/dev/null || true

# --- que disco es el medio de instalacion (para no borrarlo) ----------------
parent_of() {
  n=${1#/dev/}
  if [ -e "/sys/class/block/$n/partition" ]; then
    basename "$(dirname "$(readlink -f "/sys/class/block/$n")")"
  else
    echo "$n"
  fi
}
MEDIA_DEV=$(awk '$2=="/src/medium"{print $1; exit}' /proc/mounts || true)
MEDIA_PARENT=""
[ -n "$MEDIA_DEV" ] && MEDIA_PARENT=$(parent_of "$MEDIA_DEV")

# --- discos candidatos (>= 1 GiB, sin loop/ram/cdrom ni el medio) -------------
CANDIDATES=""
for d in /sys/block/*; do
  n=${d##*/}
  case "$n" in loop*|ram*|sr*|fd*|zram*|dm-*|md*|nbd*) continue ;; esac
  [ "$n" = "$MEDIA_PARENT" ] && continue
  [ "$(cat "$d/size")" -ge 2097152 ] || continue
  CANDIDATES="$CANDIDATES $n"
done
[ -n "$CANDIDATES" ] || die "no hay discos candidatos (>= 1 GiB)"

# --- elegir disco ----------------------------------------------------------------
if [ -n "$AUTO" ] && [ "$AUTO" != "auto" ]; then
  n=${AUTO#/dev/}
elif [ "$AUTO" = "auto" ]; then
  set -- $CANDIDATES
  n=$1
else
  echo "Discos disponibles:"
  for c in $CANDIDATES; do
    size=$(( $(cat "/sys/block/$c/size") / 2048 ))
    model=$(cat "/sys/block/$c/device/model" 2>/dev/null | tr -s ' ' || true)
    echo "  $c   ${size} MiB   $model"
  done
  printf "Disco de destino (p.ej. sda): "
  read -r n
fi
[ -b "/dev/$n" ] || die "/dev/$n no es un disco"
[ "$n" != "$MEDIA_PARENT" ] || die "/dev/$n es el medio de instalacion"
DISK=/dev/$n

if [ -z "$AUTO" ]; then
  printf "Se BORRARA todo el contenido de %s. Escribe SI para continuar: " "$DISK"
  read -r a
  [ "$a" = "SI" ] || die "cancelado"
fi

case "$n" in *[0-9]) P="${DISK}p" ;; *) P="$DISK" ;; esac

# --- particionar (mismo esquema que la imagen de disco) -------------------------
say "particionando $DISK"
dd if=/dev/zero of="$DISK" bs=1M count=2 conv=fsync 2>/dev/null
sgdisk -o "$DISK" >/dev/null
sgdisk -n 1:2048:4095 -t 1:ef02 -c 1:biosboot -A 1:set:2 "$DISK" >/dev/null
sgdisk -n 2:0:+256M -t 2:ef00 -c 2:KYBOOT "$DISK" >/dev/null
sgdisk -n 3:0:0 -t 3:8300 -c 3:kyroot "$DISK" >/dev/null

i=0
while [ ! -b "${P}3" ] && [ "$i" -lt 15 ]; do
  sleep 1; i=$((i + 1)); mdev -s 2>/dev/null || true
done
[ -b "${P}3" ] || die "no aparecen las particiones de $DISK"

UUID=$(cat /proc/sys/kernel/random/uuid)
say "formateando"
mkfs.vfat -F 32 -n KYBOOT "${P}2" >/dev/null
mkfs.ext4 -q -F -L kyroot -U "$UUID" "${P}3"

mkdir -p /target /mnt/esp
mount -t ext4 "${P}3" /target
mount -t vfat "${P}2" /mnt/esp

# --- copiar el sistema ---------------------------------------------------------------
say "copiando el sistema (puede tardar)"
tar -C /src/rootfs -cf - . | tar -C /target -xpf -

# --- arranque -------------------------------------------------------------------------
say "instalando GRUB (BIOS y UEFI)"
mkdir -p /mnt/esp/boot/grub
cp /src/medium/boot/vmlinuz /mnt/esp/vmlinuz
cp /src/medium/boot/initramfs /mnt/esp/initramfs

# OJO: mismo grub.cfg que genera el workflow para la imagen de disco; mantenerlos iguales
write_grub_cfg() {
  cat > /mnt/esp/boot/grub/grub.cfg <<EOF
set timeout=3
load_env
if [ -z "\${saved_entry}" ]; then set saved_entry=normal; fi
set default="\${saved_entry}"
menuentry "Kyanite" --id normal {
  linux /vmlinuz root=UUID=$UUID rootfstype=ext4 rw rootflags=rw init=/sbin/init console=tty0
  initrd /initramfs
}
menuentry "Kyanite (consola serie)" --id serial {
  linux /vmlinuz root=UUID=$UUID rootfstype=ext4 rw rootflags=rw init=/sbin/init console=tty0 console=ttyS0,115200
  initrd /initramfs
}
EOF
}
write_grub_cfg
grub-editenv /mnt/esp/boot/grub/grubenv create

grub-install --target=i386-pc --boot-directory=/mnt/esp/boot \
  --modules="part_gpt fat" --no-floppy "$DISK"
grub-install --target=x86_64-efi --efi-directory=/mnt/esp \
  --boot-directory=/mnt/esp/boot --modules="part_gpt fat" --removable --no-nvram
[ -f /mnt/esp/EFI/BOOT/BOOTX64.EFI ] || die "grub-install no dejo BOOTX64.EFI"

sync
umount /mnt/esp
umount /target
sync

say "instalacion completada en $DISK"
[ -n "$AUTO" ] && echo "KYANITE-INSTALL-DONE"
exit 0
