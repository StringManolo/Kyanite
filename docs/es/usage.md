# Uso

Esta guía cubre el entorno de shell interactiva incluido en Kyanite y las principales formas de personalizarlo.

## Iniciar una sesión

`/bin/sh` de Kyanite apunta a `/sbin/init`. Init configura la terminal e inicia Brush como shell interactiva de inicio de sesión.

Cuando la shell termina, init termina y la sesión finaliza.

## Archivos de configuración de la shell

| Ruta | Propósito |
| --- | --- |
| `/etc/profile` | Perfil de inicio de sesión; muestra el MOTD y carga `~/.bashrc` |
| `/root/.bashrc` | Prompt, ajustes del historial, alias y funciones auxiliares |
| `/etc/update-motd.d/` | Scripts que generan el mensaje del día |
| `/etc/motd` | Texto estático opcional que se muestra después de los scripts del MOTD |
| `/etc/kyanite/logo.txt` | Logo ASCII que muestra el script del MOTD del logo |
| `/etc/os-release` | Nombre de la distribución y versión de la compilación |

Los archivos predeterminados se mantienen en el directorio [`rootfs-overlay/`](../../rootfs-overlay/) del repositorio.

## Personalizar el mensaje del día

Los scripts del MOTD se ejecutan por orden de nombre de archivo desde `/etc/update-motd.d/`.

Para desactivar un script conservando el archivo:

```sh
chmod -x /etc/update-motd.d/00-logo
```

Para añadir un mensaje estático:

```sh
printf '%s\n' 'Welcome to Kyanite' > /etc/motd
```

Para silenciar el MOTD en tu inicio de sesión:

```sh
touch ~/.hushlogin
```

Los colores se desactivan cuando `NO_COLOR` está definida o `TERM` es `dumb`.

Los scripts del MOTD se ejecutan con `/bin/brush`, por lo que no necesitan una línea shebang `#!/bin/sh`.

## Personalizar el logo

Sustituye `/etc/kyanite/logo.txt` por tu propio texto ya coloreado.

Por ejemplo, si `chafa` está instalado:

```sh
chafa --size=36x --colors=256 logo.png > /etc/kyanite/logo.txt
```

## Alias y comandos auxiliares

El `.bashrc` predeterminado incluye un prompt, la configuración del historial, alias y funciones auxiliares como `extract` y `gacp`.

Algunos de los comandos que usan las funciones auxiliares se instalan por separado. Por ejemplo:

```sh
kpkg add tar git curl unzip
```

Consulta la [guía de kpkg](kpkg.md) para instalar y gestionar paquetes.

## Entorno

Normalmente, un lanzador define variables de entorno como `HOME`, `USER`, `LOGNAME`, `TERM`, `LANG` y `PATH`. Los valores exactos pueden variar según la terminal y la integración con el anfitrión.
