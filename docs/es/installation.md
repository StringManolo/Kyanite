# Instalación

Kyanite funciona en Android, Windows, Linux, Termux y Docker. Elige el sistema que tienes:

| Tienes... | Ir a |
| --- | --- |
| Un móvil Android (lo más fácil) | [Android](#android) |
| Un PC con Windows | [Windows](#windows) |
| Un PC con Linux | [Linux](#linux) |
| Termux en Android | [Termux](#termux) |
| Docker | [Docker](#docker) |

## ¿Qué archivos hay?

Kyanite se publica en la [página de Releases de Kyanite](https://github.com/StringManolo/Kyanite/releases). La app de Android se publica aparte, en las [releases de KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid/releases/latest).

| Qué | Archivos | Para qué sirve |
| --- | --- | --- |
| Archivo del sistema de archivos raíz | `kyanite-rootfs-aarch64.tar.xz` (ARM64), `kyanite-rootfs-x86_64.tar.xz` (AMD64), `kyanite-rootfs-armv7.tar.xz` (ARM de 32 bits) | Linux, Termux y Docker. Kyanite funciona dentro de una carpeta, sin máquina virtual. |
| Imágenes de disco x86_64 | `.img`, `.vdi`, `.vmdk`, `.iso` | Máquinas virtuales y memorias USB. Consulta [Imágenes x86_64](x86_64-images.md). |
| App de Android | `.apk` de 64 y de 32 bits | Android |

## Android

La forma más sencilla de empezar en un dispositivo Android es instalar la aplicación Kyanite. Descarga la versión para tu móvil:

- [Kyanite para Android, 64 bits](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk): la mayoría de móviles.
- [Kyanite para Android, 32 bits](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/kyanite_v7.apk): móviles antiguos.

¿No sabes cuál? Prueba primero la de 64 bits. Si Android se niega a instalarla, usa la de 32 bits. Algunos móviles tienen un procesador de 64 bits pero un sistema Android de 32 bits, y necesitan la versión de 32 bits. La [web de Kyanite](https://stringmanolo.github.io/KyaniteAndroid/) también te sugiere una. La versión de 32 bits no incluye el modo de máquina virtual.

- [Código fuente y documentación de KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid)

La aplicación de Android ofrece su propia experiencia de configuración e inicio. Consulta también la [guía de Android](android.md).

## Windows

En Windows, Kyanite funciona como máquina virtual, o desde una memoria USB en un PC real. Descarga las imágenes de disco x86_64 desde la release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest). Para saber qué es cada archivo, consulta [Imágenes x86_64](x86_64-images.md).

### VirtualBox

1. Descarga `kyanite-x86_64.vdi` e instala [VirtualBox](https://www.virtualbox.org/).
2. Pulsa **Nueva**. Ponle el nombre `Kyanite`, elige el tipo **Linux** y la versión **Other Linux (64-bit)**. Si el asistente ofrece una instalación desatendida, sáltala.
3. Dale al menos 1024 MB de memoria.
4. En el paso del disco duro elige **Usar un archivo de disco duro virtual existente**, pulsa el icono de la carpeta, **Añadir** y selecciona `kyanite-x86_64.vdi`.
5. Termina el asistente, selecciona la máquina y pulsa **Iniciar**.

### VMware

1. Descarga `kyanite-x86_64.vmdk` e instala VMware Workstation o VMware Player.
2. Elige **Archivo → Nueva máquina virtual** y selecciona **Personalizada (avanzada)**.
3. Elige **Instalaré el sistema operativo más tarde**. Como sistema operativo invitado selecciona **Linux** y **Other Linux 64-bit** (el nombre exacto cambia entre versiones de VMware).
4. Dale al menos 1 GB de memoria.
5. Cuando pida el disco, elige **Usar un disco virtual existente**, selecciona `kyanite-x86_64.vmdk` y mantén el formato existente si te lo pregunta.
6. Termina y pulsa **Encender esta máquina virtual**.

### QEMU

1. Instala QEMU para Windows desde la [página oficial de descargas](https://www.qemu.org/download/#windows).
2. Descarga `kyanite-x86_64.img` (o `kyanite-x86_64.img.zip` y extráelo).
3. Abre PowerShell en la carpeta de la imagen y ejecuta:

```powershell
& "C:\Program Files\qemu\qemu-system-x86_64.exe" -m 1G -drive file=kyanite-x86_64.img,format=raw
```

Sin aceleración va lento. Opcionalmente, activa la característica de Windows *Plataforma del hipervisor de Windows* y añade `-accel whpx` al comando.

### Rufus (un PC real desde una memoria USB)

1. Descarga `kyanite-x86_64-hw-live.iso` (un sistema live con instalador) o `kyanite-x86_64-hw.img.zip` (extráelo antes), y descarga [Rufus](https://rufus.ie/).
2. Conecta una memoria USB. **Se borrará todo lo que contenga.**
3. Abre Rufus. En **Dispositivo** elige tu memoria USB, pulsa **SELECCIONAR** y elige el archivo (para un `.img`, elige «Todos los archivos» en el cuadro de diálogo).
4. Pulsa **EMPEZAR**. Cuando Rufus pregunte, elige el **modo Imagen DD** y confirma.
5. Reinicia el PC, abre su menú de arranque e inicia desde la memoria USB.

Antes de arrancar un PC real, lee [Imágenes x86_64](x86_64-images.md): Secure Boot debe estar desactivado y el instalador todavía no se ha probado en hardware real.

## Linux

### PRoot (sin root)

PRoot ejecuta Kyanite como usuario normal, dentro de una carpeta. Funciona en la mayoría de sistemas Linux (el anfitrión debe permitir `ptrace`).

1. Instala `proot` con el gestor de paquetes, por ejemplo `sudo apt install proot` en Debian o Ubuntu.
2. Abre la [página de Releases de Kyanite](https://github.com/StringManolo/Kyanite/releases) y descarga el archivo correspondiente a tu arquitectura:

| Arquitectura | Archivo |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |
| ARM de 32 bits | `kyanite-rootfs-armv7.tar.xz` |

Descarga también los archivos `.sha256` y `.sha256.sig` correspondientes.

3. Comprueba la suma SHA256 desde el directorio que contiene los archivos descargados:

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Usa el nombre del archivo de suma que corresponda al archivo que descargaste. Para verificar también la firma Ed25519, sigue [Verificar versiones](../../SECURITY.md#verifying-releases) (en inglés).

4. Crea un directorio y extrae el archivo en él (sustituye el nombre del archivo por el que corresponda a tu arquitectura):

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

5. Inicia Kyanite. Un comando de inicio básico tiene este aspecto:

```sh
proot --kill-on-exit --link2symlink -0 -r kyanite \
  -b /dev -b /proc -b /sys \
  -w /root \
  /usr/bin/env -i \
    HOME=/root USER=root LOGNAME=root \
    TERM=xterm-256color COLORTERM=truecolor LANG=C.UTF-8 \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    /bin/sh -l
```

Según el entorno del anfitrión, un lanzador puede necesitar enlaces adicionales (`-b`).

### QEMU

QEMU ejecuta la imagen de disco x86_64 como máquina virtual.

```sh
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw
```

Añade `-enable-kvm` si tu CPU y tu kernel lo permiten: es mucho más rápido. La imagen está en la release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

### Memoria USB (un PC real)

Descarga `kyanite-x86_64-hw-live.iso` (o `kyanite-x86_64-hw.img.zip` y extráelo) desde la release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest) y grábalo en una memoria USB con balenaEtcher o con `dd`:

```sh
lsblk                                     # busca tu memoria USB, por ejemplo /dev/sdX
sudo dd if=kyanite-x86_64-hw-live.iso of=/dev/sdX bs=4M status=progress conv=fsync
```

Sustituye `/dev/sdX` por tu memoria USB. **Se borra todo lo que contenga, y si te equivocas de dispositivo puedes destruir tus datos.** Después lee [Imágenes x86_64](x86_64-images.md) antes de arrancar.

## Termux

Necesitas [Termux](https://termux.dev/), instalado desde F-Droid o desde las releases de GitHub de Termux.

### proot-distro (recomendado)

Requiere PRoot-Distro 5 o posterior. Lo más rápido es el script de instalación: descarga el archivo adecuado para tu móvil, comprueba su suma SHA256, instala Kyanite y crea el comando `kyanite`.

```sh
pkg update && pkg upgrade
curl -fsSL https://raw.githubusercontent.com/StringManolo/Kyanite/main/install-termux.sh -o install-termux.sh
sh install-termux.sh
```

Después escribe `kyanite` para abrir una shell, o `kyanite COMANDO` para ejecutar un solo comando.

Lo mismo, paso a paso:

```sh
pkg install proot-distro curl
ARCH=aarch64        # usa armv7 en un móvil de 32 bits
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz.sha256
sha256sum -c kyanite-rootfs-$ARCH.tar.xz.sha256
proot-distro install ./kyanite-rootfs-$ARCH.tar.xz --name kyanite
proot-distro login kyanite
```

Para crear el comando `kyanite` a mano:

```sh
cat > "$PREFIX/bin/kyanite" <<LAUNCHER
#!$PREFIX/bin/sh
if [ "\$#" -eq 0 ]; then
  exec proot-distro login kyanite
else
  exec proot-distro login kyanite -- "\$@"
fi
LAUNCHER
chmod 755 "$PREFIX/bin/kyanite"
```

Para eliminarlo: `proot-distro remove kyanite && rm "$PREFIX/bin/kyanite"`.

### QEMU

Es experimental y lento: QEMU emula un PC entero para ejecutar la imagen x86_64.

```sh
pkg install qemu-system-x86-64-headless unzip curl
curl -LO https://github.com/StringManolo/Kyanite/releases/download/qemuTest/kyanite-x86_64.img.zip
unzip kyanite-x86_64.img.zip
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -vnc 127.0.0.1:0
```

Abre una app de visor VNC en el móvil y conéctate a `127.0.0.1:5900`.

Para tener una ventana gráfica en lugar de VNC, esta es la configuración del propio autor:

1. Instala Termux y Termux:X11.
2. Ejecuta en Termux: `termux-x11 :0 -listen tcp -ac`
3. En [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android), con un rootfs de Alpine, ejecuta:

```sh
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. Abre la app Termux:X11.

## Docker

Es experimental. Usa el archivo que corresponda a tu CPU (`aarch64` en equipos ARM, como los Mac con Apple Silicon). Descárgalo y comprueba antes su suma, como en los pasos de PRoot de la sección [Linux](#linux).

```sh
docker import kyanite-rootfs-x86_64.tar.xz kyanite
docker run --rm -it kyanite /bin/sh -l
```

## Comportamiento al iniciar

`/bin/sh` es un enlace simbólico a `/sbin/init`. Cuando el lanzador inicia `/bin/sh`, Kyanite init inicia Brush como shell interactiva de inicio de sesión.

Para más información sobre la shell y el entorno de inicio de sesión, consulta la [guía de uso](usage.md).
