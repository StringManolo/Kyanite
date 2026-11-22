# Limitaciones conocidas y notas de compatibilidad

Esta página reúne detalles de implementación actuales que pueden afectar a la compatibilidad o al comportamiento esperado. Está pensada como referencia técnica.

## Runtime y plataforma

- Kyanite se distribuye como un sistema de archivos raíz y actualmente no incluye su propio kernel.
- El runtime actual usa PRoot y comparte el kernel del anfitrión.
- El rootfs se ha probado a través de [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android).
- `proot-distro`, QEMU y udocker todavía necesitan más pruebas.
- En Android 8 y posteriores, es posible que las aplicaciones no puedan leer algunos archivos de `/proc`. Puede que los lanzadores tengan que enlazar archivos sustitutos.

## Gestor de paquetes

- `kpkg` no ejecuta los scripts de mantenedor de Debian (`preinst`, `postinst`, `prerm`, `postrm`). Los paquetes que necesiten esas acciones requieren un hook de Kyanite o una configuración manual.
- `Recommends` y `Suggests` no se instalan automáticamente.
- Los componentes preinstalados de la base se actualizan mediante versiones de Kyanite, no mediante las actualizaciones normales de paquetes.
- La compilación obtiene actualmente algunos componentes de runtime del entorno de compilación de Ubuntu, mientras que `kpkg` instala paquetes de Debian stable. Sus versiones pueden diferir.

## Certificados

La implementación nativa de `update-ca-certificates` genera `/etc/ssl/certs/ca-certificates.crt`. No crea los enlaces `<hash>.0` que usan las aplicaciones que dependen de `CApath`.

## Comportamiento de la shell y de init

`/bin/sh` apunta al ejecutable de Kyanite init. Esto forma parte del diseño de arranque actual. Por tanto, un script de paquete que invoque `/bin/sh` puede interactuar con init en lugar de con una shell convencional si se ejecuta fuera de la ruta de arranque prevista. Los scripts de MOTD se ejecutan explícitamente con `/bin/brush`.

La compatibilidad de la shell y el comportamiento de los paquetes que presuponen un `/bin/sh` convencional son áreas pendientes de más pruebas.

## Seguridad y aislamiento

PRoot no es una máquina virtual ni una barrera de seguridad de kernel independiente. El entorno de Kyanite comparte el kernel del anfitrión y depende del lanzador del anfitrión y de su configuración.

Consulta [SECURITY.md](../../SECURITY.md) (en inglés) para ver el modelo de amenazas, las protecciones y las instrucciones para informar de problemas de seguridad.
