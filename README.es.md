🇪🇸 Español | [🇺🇸 English](README.md) | [🇷🇺 Русский](README.ru.md) | [🇨🇳 简体中文](README.zh-CN.md)

# Kyanite

Kyanite es un sistema Linux pequeño, fácil de llevar y fácil de usar. Sus herramientas principales están escritas en Rust, un lenguaje de programación que ayuda a evitar muchos errores comunes de memoria. Puedes instalar miles de programas adicionales de Debian con un solo comando.

![Pantalla de inicio de sesión de Kyanite](docs/screenshot.png)

[![Build Kyanite Rootfs](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml/badge.svg)](https://github.com/StringManolo/Kyanite/actions/workflows/build-rootfs.yml)

## En resumen

- **Pequeño:** un sistema base compacto, rápido de descargar y de arrancar.
- **Más seguro por diseño:** el sistema de inicio, el gestor de paquetes, la shell y los comandos básicos están escritos en Rust.
- **Mucho software:** `kpkg` instala programas desde los repositorios de Debian.
- **Familiar:** comandos habituales y una shell interactiva, listos para usar.
- **Funciona como una app:** no reemplaza tu sistema operativo. En Android es una app normal; en Linux funciona dentro de una carpeta.

¿Eres nuevo en Linux? El [glosario](docs/es/glossary.md) explica en lenguaje sencillo las palabras técnicas que aparecen aquí.

## Obtener Kyanite

### En Android (lo más fácil)

1. Descarga Kyanite para Android (.apk):
   - [**64 bits** (la mayoría de móviles)](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
   - [32 bits (móviles antiguos)](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/kyanite_v7.apk)
2. Abre el archivo descargado e instálalo (Android puede pedirte que permitas instalar apps desde este origen).
3. Abre la app y elige cómo iniciar Kyanite en la primera pantalla.

¿No sabes cuál descargar? La [web de Kyanite](https://stringmanolo.github.io/KyaniteAndroid/) te sugiere la adecuada, y si una no se instala, prueba la otra. Más detalles en la [guía de Android](docs/es/android.md). Código fuente: [KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid).

### En Windows

Ejecuta Kyanite en VirtualBox, VMware o QEMU, o arranca un PC real desde una memoria USB con Rufus. Consulta la [guía para Windows](docs/es/installation.md#windows).

### En Linux

Descarga un archivo del sistema de archivos raíz desde la [página de Releases](https://github.com/StringManolo/Kyanite/releases). Hay tres arquitecturas disponibles:

- `aarch64` (ARM64)
- `x86_64` (AMD64)
- `armv7` (ARM de 32 bits)

Cada versión incluye el archivo, su suma de comprobación SHA256 y una firma, para que puedas comprobar que la descarga es auténtica. Sigue la [guía para Linux](docs/es/installation.md#linux) paso a paso. También puedes usar [Docker](docs/es/installation.md#docker).

### En Termux

Ejecuta Kyanite en tu móvil desde Termux, con proot-distro o QEMU. Consulta la [guía para Termux](docs/es/installation.md#termux).

## Tus primeros pasos

Cuando Kyanite esté en marcha, puedes probar algunos comandos básicos:

```sh
pwd        # muestra la carpeta en la que estás
ls         # lista los archivos de esta carpeta
cd /etc    # te mueve a otra carpeta
```

Para instalar software, usa `kpkg`:

```sh
kpkg update
kpkg add curl
```

La [guía de uso](docs/es/usage.md) y la [guía de kpkg](docs/es/kpkg.md) tienen más ejemplos.

## ¿Qué incluye?

| Componente | Qué hace |
| --- | --- |
| `kyanite-init` | Arranca el sistema y abre la shell |
| [Brush](https://github.com/reubeno/brush) | La shell: el programa donde escribes los comandos |
| [uutils coreutils](https://github.com/uutils/coreutils) | Comandos de uso diario como `ls` y `cp`, escritos en Rust |
| `kpkg` | El gestor de paquetes: instala y elimina programas de Debian |
| Runtime glibc | Permite ejecutar programas de Debian |
| `vi` | Un editor de texto sencillo |

¿Quieres saber cómo encajan las piezas? Lee la [guía de arquitectura](docs/es/architecture.md).

## Documentación

| Guía | Qué cubre |
| --- | --- |
| [Glosario](docs/es/glossary.md) | Palabras técnicas explicadas en lenguaje sencillo |
| [Guía de Android](docs/es/android.md) | Usar Kyanite en tu móvil |
| [Instalación](docs/es/installation.md) | Instalación paso a paso en Android, Windows, Linux, Termux y Docker |
| [Imágenes x86_64](docs/es/x86_64-images.md) | Imágenes de disco para PC y máquinas virtuales (QEMU, VirtualBox, VMware) |
| [Uso](docs/es/usage.md) | Entorno de shell y personalización |
| [Gestor de paquetes (`kpkg`)](docs/es/kpkg.md) | Comandos, opciones y gestión de paquetes |
| [Hooks de paquetes](docs/es/package-hooks.md) | Cómo configura Kyanite los paquetes |
| [Arquitectura](docs/es/architecture.md) | Componentes y diseño del sistema |
| [Desarrollo](docs/es/development.md) | Compilar y trabajar con el código fuente |
| [Versiones](docs/es/releases.md) | Versionado y proceso de publicación |
| [Limitaciones conocidas](docs/es/limitations.md) | Notas de compatibilidad y limitaciones técnicas actuales |
| [Hoja de ruta](docs/es/roadmap.md) | Áreas en estudio y posibles mejoras futuras |

## Seguridad

La seguridad es un objetivo importante del diseño. [SECURITY.md](SECURITY.md) (en inglés) describe el modelo de amenazas, las protecciones ya implementadas, las carencias conocidas y cómo informar de una vulnerabilidad.

## Contribuir

Kyanite se desarrolla de forma abierta. La [guía de desarrollo](docs/es/development.md) explica cómo compilar el proyecto y por dónde empezar.
