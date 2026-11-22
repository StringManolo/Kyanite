# Desarrollo

Esta guía presenta la estructura del código fuente de Kyanite y el flujo de compilación. El workflow de GitHub Actions es la fuente de referencia para los comandos exactos de compilación y las versiones de las dependencias.

## Flujo de compilación

La compilación se define en:

```text
.github/workflows/build-rootfs.yml
```

Sus objetivos son:

- `aarch64-unknown-linux-musl`
- `x86_64-unknown-linux-musl`

A grandes rasgos, el workflow:

1. Calcula la versión con `git describe --tags --always`.
2. Compila Kyanite init y `kpkg` con Cargo.
3. Compila Brush y uutils coreutils para los objetivos musl.
4. Ensambla el sistema de archivos raíz.
5. Añade las bibliotecas de runtime y los paquetes de soporte.
6. Aplica `rootfs-overlay/`.
7. Genera el manifiesto de la base.
8. Crea el archivo comprimido y la suma SHA256.
9. Firma la suma de comprobación en las compilaciones de versión.

## Requisitos de compilación

Los requisitos locales documentados actualmente son:

- Una toolchain de Rust
- Los objetivos musl de Rust necesarios
- `musl-tools`
- `gcc-aarch64-linux-gnu` para la compilación cruzada a aarch64

Consulta el workflow por si se necesitan herramientas o paquetes adicionales para la compilación actual.

## Superposición del sistema de archivos raíz

El directorio `rootfs-overlay/` contiene los archivos que se copian al sistema de archivos raíz generado, incluidos los de configuración y de entorno de la shell.

Al modificar la superposición, comprueba las rutas de los archivos, los permisos y cómo interactúan los cambios con el manifiesto de la base y el mecanismo de actualización.

## Probar los cambios

Usa un sistema de archivos raíz desechable para probar los cambios.

Si trabajas en el gestor de paquetes, prueba las rutas de instalación, actualización, eliminación, reparación, configuración y auditoría correspondientes. Si trabajas en las actualizaciones de la base, prueba tanto los archivos sin modificar como los archivos cambiados por el usuario.

## Etiquetas de versión

Las compilaciones usan `git describe --tags --always`. Un commit posterior a una etiqueta de versión puede etiquetarse con un valor como `v0.1.0-3-g1a2b3c4`. Cuando no hay ninguna etiqueta disponible, la versión puede ser el hash del commit.

Solo las etiquetas de versión con el formato `vX.Y.Z` son elegibles para actualizaciones automáticas de la base.

## Documentación relacionada

- [Arquitectura](architecture.md)
- [Gestor de paquetes](kpkg.md)
- [Hooks de paquetes](package-hooks.md)
- [Proceso de publicación](releases.md)
- [Hoja de ruta e investigaciones](roadmap.md)
