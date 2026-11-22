#!/usr/bin/env bash
# Utilidades para probar imagenes en QEMU con capturas de pantalla. Se usa con `source`.
# Todo lo que genera va a shots/<caso>/ (serial.log, frame-*.png) y shots/results.tsv.
# shellcheck shell=bash

SHOTS_ROOT=${SHOTS_ROOT:-$PWD/shots}
LIMIT=${LIMIT:-150}         # segundos maximos por caso
INTERVAL=${INTERVAL:-3}     # segundos entre capturas
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
  if [ -w /dev/kvm ]; then echo "KVM disponible"; else echo "KVM no disponible: se usa emulacion (mas lento)"; fi
}

# fw_args bios|uefi  -> argumentos de QEMU para el firmware
fw_args() {
  if [ "$1" = uefi ]; then echo "-bios $OVMF"; fi
  return 0
}

# set_serial_entry IMAGEN -> deja la entrada "consola serie" como predeterminada en su grubenv
set_serial_entry() {
  local img=$1 loop i
  loop=$(sudo losetup --find --show --partscan "$img")
  sudo udevadm settle || true
  for i in 1 2 3 4 5 6 7 8 9 10; do [ -b "${loop}p2" ] && break; sleep 1; done
  mkdir -p "$PWD/mnt/esp"
  sudo mount "${loop}p2" "$PWD/mnt/esp"
  sudo grub-editenv "$PWD/mnt/esp/boot/grub/grubenv" set saved_entry=serial
  sudo umount "$PWD/mnt/esp"
  sudo losetup -d "$loop"
}

# matches LOG REGEX: ignora las lineas de nuestros marcadores salvo que EXPECT_RAW=1
matches() {
  if [ "${EXPECT_RAW:-0}" = 1 ]; then
    grep -qaiE "$2" "$1"
  else
    grep -av 'KYANITE-\(LIVE\|INSTALL\)' "$1" | grep -qaiE "$2"
  fi
}

detect_stages() {
  local log=$1 s=()
  grep -qa "Linux version" "$log" && s+=(kernel)
  grep -qaE "EXT4-fs \(.*\): mounted" "$log" && s+=(root-ext4)
  grep -qa "KYANITE-LIVE: switch_root" "$log" && s+=(live-init)
  grep -qa "KYANITE-SMOKE: inicio" "$log" && s+=(smoke-script)
  grep -qa "KYANITE-INSTALL-DONE" "$log" && s+=(instalador)
  local IFS=,
  echo "${s[*]:-ninguna}"
}

record() { printf '%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" >> "$SHOTS_ROOT/results.tsv"; }
record_skip() { record "$1" skip "$2" "-"; }

# run_vm NOMBRE REGEX_ESPERADA ESPERAR_APAGADO(0|1) [argumentos de QEMU...]
# Devuelve 0 si la salida serie coincide con la regex. Variables opcionales: LIMIT, VM_MEM, VM_CPU (def. max), VM_ACCEL (def. kvm:tcg), EXPECT_RAW.
run_vm() {
  local name=$1 expect=$2 waitexit=$3; shift 3
  local dir="$SHOTS_ROOT/$name" log="$SHOTS_ROOT/$name/serial.log"
  local ok=0 reason="tiempo agotado" qp sp i stages
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
    if matches "$log" "$expect"; then ok=1; reason="salida esperada encontrada"; break; fi
    if grep -qaE 'Kernel panic|KYANITE-(LIVE|INSTALL): ERROR|KYANITE-SMOKE vim rc=[1-9]' "$log"; then reason="error durante el arranque"; break; fi
    if ! kill -0 "$qp" 2>/dev/null; then reason="QEMU termino sin la salida esperada"; break; fi
  done
  if [ "$ok" = 1 ] && [ "$waitexit" = 1 ]; then
    for i in $(seq 1 20); do kill -0 "$qp" 2>/dev/null || break; sleep 2; done
  fi
  kill "$qp" 2>/dev/null || true; wait "$qp" 2>/dev/null || true
  kill "$sp" 2>/dev/null || true; wait "$sp" 2>/dev/null || true

  stages=$(detect_stages "$log")
  if [ "$ok" = 1 ]; then
    echo "OK   $name ($reason; etapas: $stages)"
    record "$name" ok "$reason" "$stages"
  else
    echo "FAIL $name ($reason; etapas: $stages)"
    tail -n 30 "$log" | tr -cd '\11\12\40-\176'
    record "$name" fail "$reason" "$stages"
  fi
  echo "::endgroup::"
  [ "$ok" = 1 ]
}

# make_smoke_image ORIGEN DESTINO
# Copia de la imagen de disco con /kyanite-smoke.sh en el rootfs y una entrada de GRUB que lo usa
# como init (init=/kyanite-smoke.sh). Comprueba que los binarios arrancan en la CPU de la VM, sin
# pasar por kyanite-init. Solo toca la copia; la imagen final no cambia.
make_smoke_image() {
  local src=$1 dst=$2 loop i uuidarg
  cp --sparse=always "$src" "$dst"
  loop=$(sudo losetup --find --show --partscan "$dst")
  sudo udevadm settle || true
  for i in 1 2 3 4 5 6 7 8 9 10; do [ -b "${loop}p3" ] && break; sleep 1; done
  mkdir -p "$PWD/mnt/esp" "$PWD/mnt/root"
  sudo mount "${loop}p3" "$PWD/mnt/root"
  sudo mount "${loop}p2" "$PWD/mnt/esp"

  sudo tee "$PWD/mnt/root/kyanite-smoke.sh" >/dev/null <<'EOF'
#!/bin/brush
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
echo "KYANITE-SMOKE: inicio"
if ! command -v vim >/dev/null 2>&1; then
  echo "KYANITE-SMOKE vim rc=127 (vim no esta en el rootfs)"
else
  vim --version > /tmp/kyanite-vim.txt 2>&1
  rc=$?
  read -r first < /tmp/kyanite-vim.txt
  echo "KYANITE-SMOKE vim rc=$rc ($first)"
fi
read -r _
EOF
  sudo chmod 755 "$PWD/mnt/root/kyanite-smoke.sh"

  uuidarg=$(grep -o 'root=UUID=[^ ]*' "$PWD/mnt/esp/boot/grub/grub.cfg" | head -n1)
  sudo tee -a "$PWD/mnt/esp/boot/grub/grub.cfg" >/dev/null <<EOF
menuentry "Kyanite (smoke)" --id smoke {
  linux /vmlinuz $uuidarg rootfstype=ext4 rw rootflags=rw init=/kyanite-smoke.sh console=tty0 console=ttyS0,115200
  initrd /initramfs
}
EOF
  sudo grub-editenv "$PWD/mnt/esp/boot/grub/grubenv" set saved_entry=smoke
  sudo umount "$PWD/mnt/esp" "$PWD/mnt/root"
  sudo losetup -d "$loop"
}
