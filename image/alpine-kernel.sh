#!/bin/sh
# Corre DENTRO de un contenedor Alpine (lo lanza el workflow).
# Variables: FLAVOR (lts|virt). Montajes: /out (salida) y /image (scripts del repo, solo lectura).
set -eu

apk add --no-cache "linux-$FLAVOR" mkinitfs >/dev/null
KVER=$(ls /lib/modules | head -n1)

# Feature propia con lo que necesita el live (squashfs, overlay, iso9660, vfat, cdrom...)
cat > /etc/mkinitfs/features.d/kylive.modules <<'EOF'
kernel/fs/squashfs/*
kernel/fs/overlayfs/*
kernel/fs/isofs/*
kernel/fs/fat/*
kernel/fs/nls/nls_cp437.*
kernel/fs/nls/nls_iso8859-1.*
kernel/fs/nls/nls_utf8.*
kernel/drivers/block/loop.*
kernel/drivers/cdrom/*
kernel/drivers/scsi/sr_mod.*
kernel/drivers/scsi/sd_mod.*
kernel/drivers/usb/storage/*
EOF

echo 'features="base ata ide scsi usb virtio ext4 nvme"' > /tmp/disk.conf
echo 'features="base ata ide scsi usb virtio ext4 nvme kylive"' > /tmp/live.conf

# initramfs del sistema instalado (init estandar de Alpine: monta root=UUID=...)
mkinitfs -b / -c /tmp/disk.conf -o /out/initramfs "$KVER"

# initramfs del live (nuestro /init: busca el medio, squashfs + overlay en RAM)
cp /image/live-init /tmp/live-init
chmod 755 /tmp/live-init
mkinitfs -b / -c /tmp/live.conf -i /tmp/live-init -o /out/initramfs-live "$KVER"

cp "/boot/vmlinuz-$FLAVOR" /out/vmlinuz
mkdir -p /out/modules
cp -a "/lib/modules/$KVER" /out/modules/
echo "$KVER" > /out/kver
chmod -R a+rX /out
ls -lh /out
