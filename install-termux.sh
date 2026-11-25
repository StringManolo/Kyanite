#!/usr/bin/env sh
# Kyanite for Termux: installs the latest Kyanite rootfs as a PRoot-Distro container and creates the `kyanite` command.
# Needs PRoot-Distro 5 or newer. Safe to run again (it keeps an existing install and just rewrites the launcher).
set -eu

NAME=kyanite
BASE=https://github.com/StringManolo/Kyanite/releases/latest/download
: "${PREFIX:?run this inside Termux}"

# Archive name for this phone (dpkg uses Termux's own architecture names: aarch64, arm, x86_64)
case "$(dpkg --print-architecture)" in
  aarch64) ARCH=aarch64 ;;
  arm)     ARCH=armv7 ;;
  x86_64)  ARCH=x86_64 ;;
  *) echo "Unsupported architecture: $(dpkg --print-architecture)" >&2; exit 1 ;;
esac
FILE="kyanite-rootfs-$ARCH.tar.xz"

pkg install -y proot-distro curl

if [ -d "$PREFIX/var/lib/proot-distro/containers/$NAME" ]; then
  echo "Kyanite is already installed (remove it first with: proot-distro remove $NAME)."
else
  TMP=$(mktemp -d)
  trap 'rm -rf "$TMP"' EXIT
  echo "Downloading $FILE ..."
  curl -fL --progress-bar -o "$TMP/$FILE" "$BASE/$FILE"
  curl -fsSL -o "$TMP/$FILE.sha256" "$BASE/$FILE.sha256"
  echo "Checking the SHA256 checksum ..."
  (cd "$TMP" && sha256sum -c "$FILE.sha256")
  proot-distro install "$TMP/$FILE" --name "$NAME"
fi

# `kyanite`          -> opens a shell
# `kyanite COMMAND`  -> runs COMMAND and exits
cat > "$PREFIX/bin/kyanite" <<LAUNCHER
#!$PREFIX/bin/sh
if [ "\$#" -eq 0 ]; then
  exec proot-distro login $NAME --isolated --fix-low-ports
else
  exec proot-distro login $NAME --isolated --fix-low-ports -- "\$@"
fi
LAUNCHER
chmod 755 "$PREFIX/bin/kyanite"

echo
echo "Done. Type  kyanite  to start it."
echo "To remove it:  proot-distro remove $NAME && rm \"$PREFIX/bin/kyanite\""
