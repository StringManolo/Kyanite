# Instalación

Kyanite se distribuye como un archivo comprimido del sistema de archivos raíz de Linux. También se puede usar mediante la aplicación dedicada para Android.

## Android

La forma más sencilla de empezar en un dispositivo Android es instalar la aplicación Kyanite.

- [Descargar el último APK de Kyanite](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
- [Código fuente y documentación de KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid)

La aplicación de Android ofrece su propia experiencia de configuración e inicio. Consulta también la [guía de Android](android.md).

## Descargar un sistema de archivos raíz de Linux

Abre la [página de Releases de Kyanite](https://github.com/StringManolo/Kyanite/releases) y descarga el archivo correspondiente a tu arquitectura:

| Arquitectura | Archivo |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |

Descarga también los archivos `.sha256` y `.sha256.sig` correspondientes.

## Verificar la descarga

Comprueba la suma SHA256 desde el directorio que contiene los archivos descargados:

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Usa el nombre del archivo de suma que corresponda al archivo que descargaste.

Para verificar también la firma Ed25519, sigue [Verificar versiones](../../SECURITY.md#verifying-releases) (en inglés).

## Extraer el sistema de archivos raíz

Crea un directorio y extrae el archivo en él:

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

Sustituye el nombre del archivo por el que corresponda a tu arquitectura.

## Iniciar con PRoot

Un comando de inicio básico tiene este aspecto:

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

Según el entorno del anfitrión, un lanzador puede necesitar enlaces adicionales (`-b`). En Android, algunos archivos de `/proc` pueden requerir un tratamiento especial.

## Comportamiento al iniciar

`/bin/sh` es un enlace simbólico a `/sbin/init`. Cuando el lanzador inicia `/bin/sh`, Kyanite init inicia Brush como shell interactiva de inicio de sesión.

Para más información sobre la shell y el entorno de inicio de sesión, consulta la [guía de uso](usage.md).
