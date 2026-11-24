# Imágenes x86_64

Kyanite también incluye imágenes de disco listas para usar en x86_64: PC, máquinas virtuales y equipos con Windows o Linux. Están publicadas en la release de GitHub [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

Para las instrucciones paso a paso (VirtualBox, VMware, QEMU, Rufus y memorias USB), consulta la [guía de instalación](installation.md). ¿Buscas los archivos del sistema de archivos raíz (`aarch64` / `x86_64`)? Mira la [guía de instalación](installation.md).

> ⚠️ El instalador incluido en la ISO live **todavía no se ha probado en hardware real** y puede borrar todos tus archivos de forma permanente. Si no sabes lo que haces, no lo ejecutes. Haz antes una copia de seguridad de tus datos.

## ¿Qué archivo necesito?

| Quiero... | Archivo |
|---|---|
| Probar Kyanite en QEMU | `kyanite-x86_64.img` |
| Usar VirtualBox | `kyanite-x86_64.vdi` |
| Usar VMware | `kyanite-x86_64.vmdk` |
| Grabar un USB con Rufus, **sin** firmware | `kyanite-x86_64.img.zip` |
| Grabar un USB para un PC real, **con** firmware | `kyanite-x86_64-hw.img.zip` |
| El típico CD live de Linux (con script de instalación) | `kyanite-x86_64-hw-live.iso` |

## Lo que verás

Así arranca Kyanite en QEMU: el menú de GRUB y después el sistema.

<img width="720" height="391" alt="Kyanite GRUB menu in QEMU" src="https://github.com/user-attachments/assets/032289ef-c50f-4e2f-b491-79b66ea7fecb" />

<img width="720" height="395" alt="Kyanite booting in QEMU" src="https://github.com/user-attachments/assets/5f5197f0-e16f-4dba-b45b-0e99b841d2ba" />

## ¿Por qué las imágenes de Rufus están en un zip?

Las imágenes se distribuyen dentro de un zip porque comprimidas pesan un 90 % menos: contienen un disco virtual con más de 500 MB de ceros como almacenamiento extra. Una vez extraídas pueden ocupar hasta 1,6 GB.

## Hardware real (las imágenes `hw`)

El conjunto de imágenes *hardware* está pensado para PC reales e incluye `linux-firmware`.

| Archivo | Qué es |
|---|---|
| `kyanite-x86_64-hw-live.iso` | Sistema live con instalador. Grábalo en un USB con Rufus en modo «DD Image» (o con balenaEtcher / `dd`). Arranca el PC desde él en modo UEFI o BIOS. |
| `kyanite-x86_64-hw.img` | El mismo sistema ya instalado en una imagen de disco. Grábalo en un USB con Rufus en modo «DD Image», o úsalo como disco virtual. |

### Antes de arrancar

- **Secure Boot debe estar desactivado** (el gestor de arranque y el kernel no están firmados).
- Si no ves el disco NVMe/SATA, cambia el modo SATA de RAID/Intel RST a AHCI en la configuración del firmware.
- Distribución de teclado: el menú de GRUB tiene un submenú «keyboard layout» (es, de, fr, it, gb, pt, br-abnt2).

### Instalar desde el sistema live

Ejecuta `/usr/local/sbin/kyanite-install`. Ofrece dos opciones:

1. **Borrar todo el disco** e instalar Kyanite (BIOS y UEFI).
2. **Instalar junto al sistema existente** (solo UEFI, disco GPT, con una partición de sistema EFI y 2 GiB libres). Conserva Windows y tus datos:
   - Reduce antes la partición de Windows desde Windows (Administración de discos).
   - Si el disco está cifrado con BitLocker, suspéndelo primero.
   - Kyanite obtiene su propia partición y reutiliza la partición de sistema EFI existente. El menú de GRUB añade una entrada «Windows». También puedes elegir Windows o Kyanite desde el menú de arranque del firmware.

### No incluido

- Compatibilidad con Secure Boot.
- Configuración automática de red (DHCP).
- Doble arranque con Windows en BIOS/MBR.
