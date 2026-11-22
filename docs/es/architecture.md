# Arquitectura

Kyanite es un sistema de archivos raíz de Linux mínimo, construido en torno a un pequeño conjunto de componentes básicos escritos en Rust y un entorno de paquetes compatible con Debian.

## Visión general del sistema

Los componentes de Rust preinstalados están enlazados estáticamente con musl:

- Kyanite init
- `kpkg`
- Brush
- uutils coreutils

El sistema de archivos raíz también incluye un runtime glibc para los paquetes de Debian enlazados dinámicamente.

## Secuencia de arranque

El flujo básico de arranque es:

1. Un lanzador entra en el sistema de archivos raíz, normalmente con PRoot.
2. El lanzador invoca `/bin/sh`.
3. `/bin/sh` apunta a `/sbin/init`.
4. Kyanite init configura la terminal e inicia Brush como shell de inicio de sesión.
5. Init recoge los procesos huérfanos y termina cuando termina la shell.

Init se ejecuta como PID 1 dentro del entorno de Kyanite.

## Componentes principales

| Componente | Responsabilidad |
| --- | --- |
| `kyanite-init` | PID 1, arranque de la shell, control de la terminal y de trabajos, recogida de procesos huérfanos |
| Brush | Shell interactiva |
| uutils coreutils | Utilidades de línea de comandos habituales |
| `kpkg` | Índices de paquetes, resolución de dependencias, instalación de paquetes y actualizaciones de la base |
| Runtime glibc | Soporte en tiempo de ejecución para los paquetes de Debian |
| `vim-tiny` | Editor mínimo compatible con vi |

## musl y glibc

Kyanite usa musl para sus herramientas de Rust enlazadas estáticamente. Esto mantiene las utilidades básicas independientes del runtime glibc.

El rootfs también incluye glibc para que los paquetes de los repositorios de Debian puedan ejecutarse. Así se combina una base compacta con el acceso al ecosistema de paquetes de Debian.

## Instalación de paquetes

`kpkg` descarga paquetes de Debian, verifica los metadatos del repositorio y los hashes de los paquetes, resuelve las dependencias y extrae los paquetes en la raíz de destino.

La configuración de los paquetes se gestiona mediante hooks declarativos de Kyanite. Consulta [Hooks de paquetes](package-hooks.md).

## Actualizaciones de la base

Las actualizaciones de la base de Kyanite son independientes de las actualizaciones normales de paquetes de Debian. Las versiones elegibles de GitHub contienen los archivos y el manifiesto necesarios para actualizar la base conservando los archivos modificados localmente.

Consulta [Versiones](releases.md) y [actualizaciones de la base con kpkg](kpkg.md#updating-the-kyanite-base).

## Modelo de ejecución

Kyanite se distribuye actualmente como un sistema de archivos raíz y usa el kernel del anfitrión a través de su entorno de ejecución. Para consideraciones sobre plataformas y aislamiento, consulta [Limitaciones conocidas](limitations.md) y [SECURITY.md](../../SECURITY.md) (en inglés).
