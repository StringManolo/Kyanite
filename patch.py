#!/usr/bin/env python3
from pathlib import Path
import re
import sys

path = Path(".github/workflows/build-rootfs.yml")

if not path.is_file():
    sys.exit(f"ERROR: No encuentro {path}")

original = path.read_text()

if "bash -x tools/generate-command-hints.sh" in original:
    sys.exit("El parche ya está aplicado.")

pattern = re.compile(
    r'(?m)^([ \t]*)bash tools/generate-command-hints\.sh[ \t]*\\\n'
    r'[ \t]*("\$ARCH_DIR")[ \t]+("\$PWD/brush-host/bin/brush")[ \t]*$'
)

match = pattern.search(original)

if not match:
    sys.exit(
        "ERROR: No encuentro la llamada esperada al generador.\n"
        "No se ha modificado ningún archivo."
    )

indent = match.group(1)

replacement = "\n".join([
    f'{indent}echo "=== HINT GENERATOR DIAGNOSTICS ==="',
    f'{indent}echo "ARCH_DIR=$ARCH_DIR"',
    f'{indent}echo "PWD=$PWD"',
    f'{indent}echo "=== HOST BRUSH ==="',
    f'{indent}ls -l "$PWD/brush-host/bin/brush" || true',
    f'{indent}' + 'if [ -x "$PWD/brush-host/bin/brush" ]; then',
    f'{indent}  "$PWD/brush-host/bin/brush" -c \'compgen -b\' || true',
    f'{indent}else',
    f'{indent}  echo "ERROR: Host Brush missing or not executable"',
    f'{indent}fi',
    f'{indent}echo "=== ROOTFS DIRECTORIES ==="',
    f'{indent}ls -ld "$ARCH_DIR/bin" "$ARCH_DIR/usr/bin" "$ARCH_DIR/sbin" "$ARCH_DIR/usr/sbin" "$ARCH_DIR/etc/kyanite" || true',
    f'{indent}echo "=== GENERATOR TRACE ==="',
    f'{indent}if bash -x tools/generate-command-hints.sh "$ARCH_DIR" "$PWD/brush-host/bin/brush"; then',
    f'{indent}  echo "Command hints generated successfully."',
    f'{indent}else',
    f'{indent}  status=$?',
    f'{indent}  echo "::error::Command hint generator failed with exit code $status"',
    f'{indent}  exit "$status"',
    f'{indent}fi',
])

backup = path.with_suffix(path.suffix + ".bak")
backup.write_text(original)

patched = original[:match.start()] + replacement + original[match.end():]
path.write_text(patched)

print("Patch applied successfully.")
print(f"Backup: {backup}")
print(f"Modified: {path}")
