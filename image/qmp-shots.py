#!/usr/bin/env python3
"""Captura la pantalla de una VM de QEMU cada N segundos via QMP (no hace falta GUI).

Uso: qmp-shots.py SOCKET_QMP DIRECTORIO_SALIDA INTERVALO_SEGUNDOS
Guarda frame-NNNNs.png (NNNN = segundos desde que arranco el script).
Descarta los fotogramas identicos al anterior. Termina solo cuando QEMU se cierra.
"""
import hashlib
import json
import os
import socket
import sys
import time

sock_path, outdir, interval = sys.argv[1], sys.argv[2], float(sys.argv[3])
os.makedirs(outdir, exist_ok=True)
t0 = time.time()

sock = None
while sock is None and time.time() - t0 < 30:
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        s.connect(sock_path)
        sock = s
    except OSError:
        s.close()
        time.sleep(0.5)
if sock is None:
    sys.exit(1)

f = sock.makefile("rw", buffering=1)


def cmd(name, **args):
    f.write(json.dumps({"execute": name, "arguments": args}) + "\n")
    f.flush()
    while True:
        line = f.readline()
        if not line:
            raise EOFError
        msg = json.loads(line)
        if "return" in msg or "error" in msg:
            return msg


try:
    f.readline()  # saludo de QMP
    cmd("qmp_capabilities")
    last = None
    while True:
        path = os.path.join(os.path.abspath(outdir), "frame-%04ds.png" % int(time.time() - t0))
        r = cmd("screendump", filename=path, format="png")
        if "return" in r and os.path.exists(path):
            with open(path, "rb") as fh:
                h = hashlib.md5(fh.read()).hexdigest()
            if h == last:
                os.remove(path)
            else:
                last = h
        time.sleep(interval)
except (EOFError, OSError, ValueError):
    pass
