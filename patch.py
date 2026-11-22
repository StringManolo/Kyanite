#!/usr/bin/env python3
"""Parche v2 para init/src/main.rs (aplicar DESPUES de patch_init.py).

En vez de decidir por "soy PID 1", decide por lo que de verdad falta:
  - lacks_controlling_tty(): stdin es un tty pero tcgetpgrp() da ENOTTY
    (este proceso no tiene terminal de control)  -> setsid + TIOCSCTTY
  - winsize_unknown(): TIOCGWINSZ da 0x0 o falla   -> autodetect_winsize()
En proot ambas dan false (el pty de la app ya trae terminal de control y tamano),
asi que proot sigue sin tocarse, y en la VM funciona aunque el init no sea PID 1.

Uso:  python3 patch_init_v2.py [ruta/a/main.rs]     (por defecto init/src/main.rs)
"""
import shutil
import sys
from pathlib import Path

path = Path(sys.argv[1] if len(sys.argv) > 1 else "init/src/main.rs")
src = path.read_text(encoding="utf-8")

if "fn lacks_controlling_tty" in src:
    sys.exit("El parche v2 ya esta aplicado. No se hace nada.")
if "fn is_vm_boot" not in src:
    sys.exit("Falta el parche v1 (patch_init.py). Aplicalo primero.")

REPLACEMENTS = [
    (
        '''/// true = arranque como PID 1 (VM): no hay terminal de control ni tamano de ventana.
/// En proot (PID != 1) el pty de la app ya trae ambas cosas y no se toca nada.
fn is_vm_boot() -> bool {
    std::process::id() == 1
}
''',
        '''/// true = stdin es un tty pero este proceso no tiene terminal de control (arranque VM).
/// tcgetpgrp() da ENOTTY cuando el tty no es el terminal de control del proceso que llama.
/// En proot el pty de la app ya es el terminal de control: da false y no se toca nada.
fn lacks_controlling_tty() -> bool {
    unsafe { libc::isatty(0) == 1 && libc::tcgetpgrp(0) < 0 }
}

/// true = el tty no conoce su tamano (consola serie en la VM: 0 filas x 0 columnas).
fn winsize_unknown() -> bool {
    unsafe {
        if libc::isatty(0) != 1 {
            return false;
        }
        let mut ws: libc::winsize = std::mem::zeroed();
        libc::ioctl(0, libc::TIOCGWINSZ as _, &mut ws) != 0 || ws.ws_row == 0 || ws.ws_col == 0
    }
}
''',
    ),
    (
        "            child_job_control(is_vm_boot());\n",
        "            child_job_control(lacks_controlling_tty());\n",
    ),
    (
        "    if is_vm_boot() {\n        autodetect_winsize();\n    }\n",
        "    if winsize_unknown() {\n        autodetect_winsize();\n    }\n",
    ),
    (
        "        if !is_vm_boot() {\n            reclaim_tty();\n        }\n",
        "        if !lacks_controlling_tty() {\n            reclaim_tty();\n        }\n",
    ),
]

for n, (old, new) in enumerate(REPLACEMENTS, start=1):
    count = src.count(old)
    if count != 1:
        sys.exit(f"Reemplazo {n}: se esperaba 1 coincidencia y hay {count}. No se escribe nada.")
    src = src.replace(old, new)

if "is_vm_boot" in src:
    sys.exit("Quedan usos de is_vm_boot sin reemplazar. No se escribe nada.")

shutil.copyfile(path, str(path) + ".bak2")
path.write_text(src, encoding="utf-8")
print(f"OK: {path} actualizado a v2 (copia en {path}.bak2)")
