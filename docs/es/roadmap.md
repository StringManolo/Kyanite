# Hoja de ruta e investigaciones

Esta página resume áreas de posible trabajo futuro. No es un plan de publicación cerrado: las prioridades pueden cambiar a medida que el desarrollo y las pruebas revelen qué es más útil.

## Experiencia de usuario

- Seguir mejorando la configuración predeterminada de Brush y la experiencia de inicio de sesión.
- Mejorar el registro (logging) de init para facilitar el diagnóstico de problemas de arranque y de ejecución.
- Revisar el MOTD predeterminado y la configuración de la shell.
- Valorar añadir utilidades de uso común que actualmente no están en la base.
- Estudiar una papelera de reciclaje y opciones de copia de seguridad en la nube.

## Compatibilidad de paquetes

- Ampliar las pruebas con software de uso común e identificar los paquetes que necesitan hooks adicionales.
- Seguir probando herramientas de desarrollo y aplicaciones de línea de comandos.
- Investigar problemas de compatibilidad cuando las aplicaciones detectan o presuponen un entorno Linux diferente.
- Revisar cómo interactúan otras shells con el flujo de init y de inicio de sesión de Kyanite.

## Soporte de plataformas y estabilidad

- Seguir investigando el arranque de QEMU y el manejo de la terminal.
- Probar el rootfs en más entornos, incluidos `proot-distro` y udocker.
- Mejorar la experiencia de instalación con `proot-distro`.
- Seguir probando el entorno x86_64 y otras configuraciones de anfitrión.
- Explorar trabajo relacionado con el kernel y enfoques alternativos de arranque o despliegue.

## Seguridad y mantenimiento

- Revisar los archivos incluidos en el sistema de archivos raíz.
- Seguir revisando el comportamiento de seguridad del gestor de paquetes y la selección de versiones de los repositorios.
- Investigar un reemplazo en Rust para el componente GNU `env` que queda.
- Revisar el equilibrio entre binarios musl estáticos y componentes enlazados dinámicamente.

## Tamaño y dependencias

- Revisar el tamaño y el uso de recursos de componentes incluidos como Brush y uutils coreutils.
- Estudiar si merecerían la pena alternativas más pequeñas u optimizaciones específicas.

## Otras ideas

Algunas ideas son exploratorias y puede que no lleguen a formar parte de Kyanite:

- Soporte de X11
- Flujos de trabajo adicionales con el kernel y con dispositivos
- Una distribución independiente centrada en la seguridad, construida con herramientas relacionadas con Kyanite

El README del proyecto describe las capacidades actuales orientadas al usuario. Esta página recoge áreas en consideración, no promesas ni compromisos de publicación.
