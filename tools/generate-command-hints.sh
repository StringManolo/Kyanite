#!/usr/bin/env bash
# Generate etc/kyanite/command-hints.sh for the rootfs.
# Usage: generate-command-hints.sh <rootfs-dir> [brush-binary]
set -euo pipefail

rootfs=$1
brush=${2:-}

# 1. Command -> package map (same file the shell sources at runtime).
. rootfs-overlay/etc/kyanite/hint-map.sh

# 2. Every executable name shipped in the image.
#    Adjust these dirs to match the runtime $PATH of the image.
search_dirs=()
for dir in "$rootfs/bin" "$rootfs/usr/bin" "$rootfs/sbin" "$rootfs/usr/sbin"; do
  [[ -d "$dir" ]] && search_dirs+=("$dir")
done

names=$(
  find "${search_dirs[@]}" \
       -mindepth 1 -maxdepth 1 \( -type f -o -type l \) -printf '%f\n' | sort -u
)
names=$'\n'"$names"$'\n'

# 3. Brush builtins (runtime `type -t` would match these too).
builtins=$'\n'
if [[ -n $brush && -x $brush ]]; then
  builtins+=$("$brush" -c 'compgen -b' 2>/dev/null || true)$'\n'
fi

have() { [[ $names == *$'\n'"$1"$'\n'* || $builtins == *$'\n'"$1"$'\n'* ]]; }

emit_stub() {
  sed -e "s/__CMD__/$1/g" -e "s/__PKG__/$2/g" <<'EOF'
__CMD__() {
    local _kx_pkg='__PKG__' _kx_exe
    _kx_exe=$(type -P '__CMD__' 2>/dev/null)
    if [[ -n $_kx_exe ]]; then
        unset -f '__CMD__'
        "$_kx_exe" "$@"
        return $?
    fi
    printf '\033[31merror\033[0m: command not found: %s\n' '__CMD__' >&2
    printf 'hint: install it with: kpkg add %s\n' "$_kx_pkg" >&2
    return 127
}
EOF
}

out="$rootfs/etc/kyanite/command-hints.sh"
mkdir -p "$(dirname "$out")"
{
  echo '# Auto-generated at image build time from etc/kyanite/hint-map.sh.'
  echo '# Do not edit; regenerate instead.'
  n=0
  for cmd in "${!_KX_MISSING[@]}"; do
    have "$cmd" && continue
    emit_stub "$cmd" "${_KX_MISSING[$cmd]}"
    n=$((n + 1))
  done
  echo "# $n hint function(s)"
} > "$out"
chmod 0644 "$out"
echo "command-hints.sh: $n stubs"
