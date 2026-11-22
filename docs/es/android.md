# Kyanite en Android

Esta página es un resumen para principiantes de la app de Kyanite para Android. El [repositorio KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid) es la fuente oficial y contiene todos los detalles actualizados (en inglés).

## ¿Qué es?

Una app de Android que ejecuta Kyanite con una terminal integrada. Usa PRoot, así que no se necesitan permisos especiales. El sistema Kyanite viene incluido dentro de la app.

## Instalar

1. [Descarga el último APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk).
2. Abre el archivo e instálalo. Android puede pedirte que permitas instalar desde este origen.
3. Las actualizaciones siempre necesitan tu confirmación. Deben estar firmadas con la misma clave que la versión instalada.

## Primer inicio

No se instala ni se inicia nada hasta que tú lo eliges. El primer inicio abre la pantalla de ajustes (el botón ⚙ está siempre disponible).

## Formas de ejecutar Kyanite

| Modo | Qué ocurre |
| --- | --- |
| **Normal** | Tus cambios se conservan entre sesiones. |
| **Efímero** | Todo se borra al salir. Usa almacenamiento temporal en disco, así que pueden quedar restos en el almacenamiento flash. |
| **VM** | Experimental. Se ejecuta en una máquina emulada (QEMU): mucho más lento, pero no escribe nada en el disco. |

También puedes elegir entre **System** (tu Kyanite instalado) y **LiveCD** (el sistema que viene con la app, como un live CD).

## Ventana y teclado

- **Pantalla completa** o **ventana flotante** (necesita el permiso «Mostrar sobre otras apps»). La ventana flotante se puede mover, redimensionar, girar, fijar, hacer transparente y minimizar.
- Una **barra de teclas extra** ofrece ESC, TAB, CTRL, ALT, flechas y más. Toca una tecla modificadora una vez para usarla una sola vez; mantenla pulsada para bloquearla.
- Elige esquemas de color y tamaño de letra en los ajustes. Pellizca para ampliar el texto.

## Opciones de privacidad y seguridad

- Bloqueo con PIN.
- Cifrado de los sistemas almacenados.
- Opciones para bloquear capturas de pantalla, ocultar notificaciones y borrar el portapapeles.
- Un interruptor para cortar el acceso de la app a la red.
- Borrado programado de los datos.

## Conviene saber

- PRoot **no** es una barrera de seguridad. Para un aislamiento más fuerte, usa el modo VM.
- Android 12 y posteriores pueden cerrar procesos en segundo plano. Si tus sesiones terminan en segundo plano, busca el ajuste del «phantom process killer».
- Una vez dentro, usa `kpkg` para instalar software. Consulta la [guía de kpkg](kpkg.md).
