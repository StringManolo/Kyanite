#!/bin/sh
# Runs INSIDE an Alpine container (launched by the workflow).
# Env:    FLAVOR (lts|virt), VARIANT (vm|hardware)
# Mounts: /out (output directory), /image (repo scripts, read-only)
set -eu

apk add --no-cache "linux-$FLAVOR" mkinitfs busybox-static kbd-bkeymaps >/dev/null
KVER=$(ls /lib/modules | head -n1)

# Custom initramfs feature with what the live ISO needs (squashfs, overlay, iso9660, vfat, cdrom...)
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

# initramfs of the installed system (stock Alpine init: mounts root=UUID=...)
mkinitfs -b / -c /tmp/disk.conf -o /out/initramfs "$KVER"

# initramfs of the live system (our own /init: finds the medium, squashfs + overlay in RAM)
cp /image/live-init /tmp/live-init
chmod 755 /tmp/live-init
mkinitfs -b / -c /tmp/live.conf -i /tmp/live-init -o /out/initramfs-live "$KVER"

cp "/boot/vmlinuz-$FLAVOR" /out/vmlinuz
mkdir -p /out/modules
cp -a "/lib/modules/$KVER" /out/modules/
echo "$KVER" > /out/kver

# Runtime helpers for the real-hardware boot path (driver coldplug + console keymaps)
mkdir -p /out/hw /out/bkeymaps
cp /bin/busybox.static /out/hw/busybox
cp -a /usr/share/bkeymaps/. /out/bkeymaps/

# Firmware: hardware variant only. Package names are tried one by one so a missing one does not break the build.
mkdir -p /out/firmware
if [ "${VARIANT:-vm}" = hardware ]; then
  FW_PKGS="linux-firmware-amdgpu linux-firmware-radeon linux-firmware-i915 linux-firmware-intel
    linux-firmware-iwlwifi linux-firmware-rtl_nic linux-firmware-rtlwifi linux-firmware-rtw88
    linux-firmware-rtw89 linux-firmware-rtl_bt linux-firmware-brcm linux-firmware-cypress
    linux-firmware-ath9k_htc linux-firmware-ath10k linux-firmware-ath11k linux-firmware-ath12k
    linux-firmware-mediatek linux-firmware-ralink linux-firmware-qca linux-firmware-bnx2
    linux-firmware-bnx2x linux-firmware-other sof-firmware"
  for pkg in $FW_PKGS; do
    if apk add --no-cache "$pkg" >/dev/null 2>&1; then
      echo "firmware: installed $pkg"
    else
      echo "firmware: $pkg not available (skipped)"
    fi
  done
  if [ -z "$(ls -A /lib/firmware 2>/dev/null)" ]; then
    echo "firmware: no individual package matched, trying the full linux-firmware package"
    apk add --no-cache linux-firmware >/dev/null
  fi
  [ -n "$(ls -A /lib/firmware 2>/dev/null)" ] || { echo "ERROR: no firmware could be installed" >&2; exit 1; }
  cp -a /lib/firmware/. /out/firmware/
  du -sh /out/firmware
fi

chmod -R a+rX /out
ls -lh /out
