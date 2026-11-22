# Glosario

Explicaciones breves de las palabras técnicas que aparecen en la documentación de Kyanite.

| Término | Significado |
| --- | --- |
| **Linux** | Una familia de sistemas operativos libres. Kyanite es un sistema Linux pequeño. |
| **Kernel (núcleo)** | El corazón de un sistema operativo. Se comunica con el hardware. Kyanite todavía no incluye su propio kernel; usa el del dispositivo donde se ejecuta (el *anfitrión*). |
| **Sistema de archivos raíz (rootfs)** | Todos los archivos que forman un sistema Linux: programas, configuración y carpetas. Kyanite se distribuye como un rootfs. |
| **Terminal** | La ventana donde escribes comandos. |
| **Shell** | El programa dentro de la terminal que lee tus comandos y los ejecuta. Kyanite usa Brush. |
| **Comando** | Una instrucción que escribes en la shell, como `ls` para listar archivos. |
| **Paquete** | Un programa (o una biblioteca) empaquetado para instalarlo fácilmente. |
| **Gestor de paquetes** | Una herramienta que instala, actualiza y elimina paquetes. En Kyanite es `kpkg`. |
| **Repositorio** | Una colección de paquetes en línea. Kyanite usa los repositorios de Debian. |
| **Debian** | Una distribución de Linux muy conocida, con una colección enorme de paquetes. |
| **Init** | El primer programa que arranca el sistema. Inicia la shell y mantiene todo en marcha. |
| **PRoot** | Una herramienta que ejecuta un sistema Linux dentro de una carpeta, sin permisos especiales. Kyanite funciona actualmente así. |
| **Rust** | Un lenguaje de programación diseñado para evitar muchos errores relacionados con la memoria, una fuente habitual de problemas de seguridad. |
| **musl y glibc** | Dos bibliotecas de C que los programas necesitan para funcionar. Las herramientas propias de Kyanite usan musl; los paquetes de Debian usan glibc. Ambas están incluidas. |
| **Arquitectura** | El tipo de procesador. `aarch64` (ARM64) es habitual en móviles; `x86_64` (AMD64) en la mayoría de los PC. |
| **Suma de comprobación (SHA256)** | Una huella corta de un archivo. Si el archivo cambia, la huella cambia, así que sirve para detectar descargas dañadas. |
| **Firma (Ed25519)** | Una forma de demostrar que un archivo fue publicado por su autor real. |
| **APK** | El formato de archivo de las apps de Android. |
| **MOTD** | «Mensaje del día»: el texto que se muestra al iniciar sesión. |
| **Hook** | Una pequeña regla de Kyanite que configura un paquete después de instalarlo. Consulta [Hooks de paquetes](package-hooks.md). |
| **Base** | La parte preinstalada de Kyanite, que se actualiza mediante versiones de Kyanite y no como paquetes normales. |

Volver al [índice de documentación](../../README.es.md#documentación).
