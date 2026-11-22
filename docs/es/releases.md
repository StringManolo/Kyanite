# Versiones

Kyanite publica archivos del sistema de archivos raíz para cada arquitectura admitida a través de GitHub Releases.

## Etiquetas de versión

Las etiquetas de versión usan el formato `vX.Y.Z`. Por ejemplo:

```sh
git tag -a v0.1.0 -m "Kyanite 0.1.0"
git push origin v0.1.0
```

Al enviar una etiqueta con este formato se activa el workflow de publicación. Otros formatos de etiqueta no los usa el mecanismo automático de actualización de la base.

## Archivos de cada versión

Cada versión contiene los siguientes archivos para cada arquitectura:

- `kyanite-rootfs-<arq>.tar.xz`
- `kyanite-rootfs-<arq>.tar.xz.sha256`
- `kyanite-rootfs-<arq>.tar.xz.sha256.sig`

El archivo `.sig` contiene una firma Ed25519 del archivo `.sha256`.

## Clave de firma

La clave pública de las versiones se guarda en:

```text
rootfs-overlay/etc/kpkg/kyanite-release.pub
```

La clave privada correspondiente se guarda como secreto del repositorio de GitHub:

```text
KYANITE_SIGNING_KEY
```

No subas ni publiques nunca la clave privada de firma.

Para la verificación por parte del usuario, consulta [Verificar versiones](../../SECURITY.md#verifying-releases) (en inglés).

## Changelog

Cuando una versión requiere una operación que no se puede representar como una actualización normal de archivos (por ejemplo, reemplazar un paquete preinstalado), añade una sección para la nueva versión en:

```text
rootfs-overlay/usr/share/kyanite/CHANGELOG
```

El changelog describe las operaciones que el actualizador puede realizar en estas transiciones.

## Compilaciones de desarrollo

Las compilaciones de commits que no corresponden a una etiqueta de versión se etiquetan con `git describe`, por ejemplo:

```text
v0.1.0-3-g1a2b3c4
```

Las compilaciones de desarrollo no son elegibles para actualizaciones automáticas de la base. El actualizador solo considera las etiquetas de versión `vX.Y.Z`.

## Aplicación de Android

La aplicación de Android tiene un repositorio y un proceso de publicación independientes:

- [Versiones de KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid/releases)
- [Descargar el último APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
