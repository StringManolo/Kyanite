# kpkg

`kpkg` es el gestor de paquetes de Kyanite. Ofrece una interfaz de línea de comandos al estilo de apk, pero instala paquetes desde los repositorios de Debian. También ofrece un modo compatible con apt.

## Inicio rápido

```sh
kpkg update
kpkg search curl
kpkg add curl
kpkg info curl
```

## Comandos

| Comando | Descripción |
| --- | --- |
| `kpkg update` | Descarga y verifica los índices de paquetes; comprueba si hay una actualización de la base de Kyanite |
| `kpkg search <término>` | Busca en los nombres y descripciones de los paquetes |
| `kpkg add <paquete>...` | Instala paquetes y sus dependencias |
| `kpkg del <paquete>...` | Elimina paquetes |
| `kpkg purge <paquete>...` | Elimina paquetes, incluidos los archivos de configuración modificados |
| `kpkg autoremove` | Elimina las dependencias que ya no se necesitan |
| `kpkg upgrade` | Aplica una actualización pendiente de la base de Kyanite y actualiza los paquetes instalados |
| `kpkg fix <paquete>...` | Reinstala paquetes para repararlos |
| `kpkg info [paquete]` | Muestra información de un paquete |
| `kpkg list [patrón]` | Lista los paquetes disponibles |
| `kpkg policy <paquete>...` | Muestra las versiones instalada y candidata |
| `kpkg configure [paquete]...` | Vuelve a ejecutar los hooks de configuración de los paquetes |
| `kpkg audit` | Verifica los archivos instalados con las sumas SHA256 registradas |
| `kpkg cache clean` | Elimina los paquetes descargados de la caché |
| `kpkg cache size` | Muestra el uso de la caché |
| `kpkg base` | Muestra la versión de la base de Kyanite y la información de actualizaciones pendientes |

`kpkg info` admite consultas adicionales sobre archivos y dependencias:

- `-L`: lista los archivos
- `-W`: muestra a qué paquete pertenece una ruta
- `-R`: muestra las dependencias
- `-r`: muestra las dependencias inversas

Para conocer la sintaxis exacta disponible en tu compilación, ejecuta:

```sh
kpkg --help
```

## Modo compatible con APT

Kyanite ofrece `apt`, `apt-get` y `apt-cache` como enlaces simbólicos a `kpkg`.

Cuando se invoca con uno de estos nombres, o con `KPKG_SYNTAX=apt`, el programa muestra la ayuda al estilo de apt y pide confirmación antes de hacer cambios. Usa `-y` para omitir la confirmación.

```sh
apt update
apt install curl
apt remove curl
apt autoremove
apt list --installed
apt show curl
```

Un alias de shell como `alias apt=kpkg` no cambia el nombre del ejecutable. Para activar el modo apt con un alias, usa:

```sh
alias apt='KPKG_SYNTAX=apt kpkg'
```

## Opciones

| Opción | Descripción |
| --- | --- |
| `-s`, `--simulate` | Muestra las acciones previstas sin cambiar nada |
| `-f`, `--force` | Sobrescribe archivos sin propietario o ajenos e ignora las dependencias inversas |
| `-y`, `--yes` | Omite las confirmaciones en modo apt |
| `--root <dir>` | Opera sobre otra raíz; por defecto `/` |
| `--arch <arq>` | Establece la arquitectura de Debian |
| `--mirror <url>` | Establece el espejo del repositorio de Debian |
| `--security-mirror <url>` | Establece el espejo del repositorio de seguridad de Debian |
| `--suite <nombre>` | Establece la suite de Debian; por defecto `stable` |
| `--keyring <archivo>` | Usa un llavero OpenPGP concreto |
| `--no-security` | Desactiva el repositorio `<suite>-security` |

## Repositorios y selección de paquetes

El índice de paquetes se construye a partir de Debian `stable`, `stable-updates` y `stable-security`. Para cada paquete, `kpkg` selecciona la versión más alta disponible.

Los metadatos del repositorio se verifican con OpenPGP. Las descargas de paquetes se comprueban (tamaño y SHA256) con el índice firmado antes de escribirse en su ubicación final.

La resolución de dependencias admite:

- `Pre-Depends`
- `Depends`, incluidas las alternativas (`a | b`) y las restricciones de versión
- Paquetes virtuales mediante `Provides`
- `Conflicts` y `Breaks`

La comparación de versiones sigue el algoritmo de dpkg.

## Configuración de paquetes

Kyanite usa hooks declarativos para configurar los paquetes. Lee la [guía de hooks de paquetes](package-hooks.md) para conocer las ubicaciones de los hooks, las acciones admitidas y notas de desarrollo.

## Estado de los paquetes y caché

El estado del gestor de paquetes se guarda en `/var/lib/kpkg`. Los archivos `.deb` descargados se guardan en caché en `/var/cache/kpkg/pkgs`.

Los paquetes incluidos en el sistema de archivos raíz base figuran en `/etc/kpkg/assume-installed`, para que `kpkg` no los instale de nuevo.

<a id="updating-the-kyanite-base"></a>

## Actualizar la base de Kyanite

`kpkg update` comprueba cuál es la última versión elegible de Kyanite en GitHub. `kpkg upgrade` aplica una actualización pendiente de la base y además actualiza los paquetes instalados.

Solo se consideran las etiquetas de versión con el formato `vX.Y.Z`. Las compilaciones de desarrollo identificadas por un hash de commit o por la salida de `git describe` no son elegibles para actualizaciones automáticas de la base.

El actualizador verifica la firma de la versión y la suma SHA256, descomprime en un directorio temporal y aplica los archivos mediante un manifiesto. Los archivos modificados localmente se conservan; la versión nueva se guarda junto a ellos con el sufijo `.kyanite-new`.

El `CHANGELOG` de la versión documenta las operaciones necesarias para los cambios que no se pueden representar como actualizaciones normales de archivos.
