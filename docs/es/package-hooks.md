# Hooks de paquetes

Kyanite usa hooks declarativos para configurar los paquetes, en lugar de ejecutar los scripts de mantenedor de los paquetes de Debian.

Los hooks describen un conjunto limitado de operaciones que `kpkg` puede realizar dentro del sistema de archivos raíz de destino. También pueden añadir o ignorar dependencias de paquetes.

## Ubicación de los hooks

Los hooks incluidos en el sistema se guardan en:

```text
/usr/share/kpkg/hooks/<paquete>.hook
```

Las modificaciones locales de los hooks se guardan en:

```text
/etc/kpkg/hooks.d/
```

## Acciones admitidas

Actualmente el motor de hooks ofrece un conjunto cerrado de acciones:

- `mkdir`
- `symlink`
- `link-default`
- `copy-default`
- `remove`
- `chmod`
- `builtin`

Los hooks también pueden añadir o ignorar dependencias.

Estas acciones son declarativas; un hook no es un script de shell arbitrario.

## Ejemplo: certificados CA

El paquete `ca-certificates` usa funcionalidad nativa de Kyanite para reconstruir:

```text
/etc/ssl/certs/ca-certificates.crt
```

El hook de `curl` añade una dependencia de `ca-certificates`, de modo que el paquete de certificados esté disponible cuando se instala curl.

## Volver a ejecutar los hooks

Ejecuta los hooks de configuración de los paquetes instalados con:

```sh
kpkg configure
```

Para configurar paquetes concretos:

```sh
kpkg configure curl ca-certificates
```

## Desarrollar un hook

Al añadir un hook, identifica primero la configuración que realizan los scripts de mantenedor de Debian del paquete. Decide qué partes se necesitan en Kyanite y exprésalas con las acciones de hook admitidas o con un `builtin` nativo existente.

Un ciclo de pruebas útil incluye:

1. Instalar el paquete en un rootfs limpio.
2. Comprobar los archivos y la configuración resultantes.
3. Ejecutar `kpkg configure` de nuevo.
4. Eliminar el paquete y comprobar cómo se limpia.
5. Probar las dependencias que introduce el hook.

Consulta los ejemplos de hooks y la implementación en el código fuente para conocer la gramática exacta. Esta guía enumera las familias de acciones, pero no define la sintaxis del analizador.
