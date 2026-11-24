#!/usr/bin/env bash
# Crea la documentación de las imágenes x86_64 (en/es/ru/zh-CN) y añade su fila a la tabla "Documentación" de los 4 README.
# Ejecutar en la raíz del repo Kyanite. Es seguro repetirlo: no sobrescribe nada (FORCE=1 reescribe los .md de docs).
set -eu
[ -d docs/es ] && [ -f README.md ] || { echo "ERROR: ejecútalo en la raíz del repo Kyanite" >&2; exit 1; }
made=0
echo "== docs ==" 
if [ -e "docs/x86_64-images.md" ] && [ "${FORCE:-}" != 1 ]; then echo "  [YA EXISTE] docs/x86_64-images.md"; else
cat > "docs/x86_64-images.md" <<'KYX86EOF'
🇺🇸 English | [🇪🇸 Español](es/x86_64-images.md) | [🇷🇺 Русский](ru/x86_64-images.md) | [🇨🇳 简体中文](zh-CN/x86_64-images.md)

# x86_64 images

Kyanite also ships ready-to-use disk images for x86_64: PCs, virtual machines and Windows/Linux hosts. They are published in the GitHub release [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

Looking for the root filesystem archives (`aarch64` / `x86_64`) instead? See the [installation guide](installation.md).

> ⚠️ The installer included in the live ISO has **not been tested on real hardware yet** and can permanently delete all your files. If you do not know what you are doing, do not run it. Back up your data first.

## Which file do I need?

| I want to... | File |
|---|---|
| Try Kyanite in QEMU | `kyanite-x86_64.img` |
| Use VirtualBox | `kyanite-x86_64.vdi` |
| Use VMware | `kyanite-x86_64.vmdk` |
| Write a USB stick with Rufus, **no** firmware included | `kyanite-x86_64.img.zip` |
| Write a USB stick for a real PC, firmware **included** | `kyanite-x86_64-hw.img.zip` |
| Get the typical Linux live CD (with an installer script) | `kyanite-x86_64-hw-live.iso` |

## QEMU (BIOS)

<img width="720" height="391" alt="Kyanite GRUB menu in QEMU" src="https://github.com/user-attachments/assets/032289ef-c50f-4e2f-b491-79b66ea7fecb" />

This is just my setup; you can use any other.

1. Install Termux and Termux:X11.
2. Run in Termux:

```bash
termux-x11 :0 -listen tcp -ac
```

3. Run in [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) with an Alpine rootfs (this assumes `qemu-system-x86_64` is already installed):

```bash
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. Open the Termux:X11 app and you will see the BIOS.

<img width="720" height="395" alt="Kyanite booting in QEMU" src="https://github.com/user-attachments/assets/5f5197f0-e16f-4dba-b45b-0e99b841d2ba" />

## VirtualBox

Use `kyanite-x86_64.vdi`.

## VMware

Use `kyanite-x86_64.vmdk`.

## Rufus (USB stick)

Extract the zip first and write the `.img` in **DD Image** mode. The image lives on its own virtual drive.

- `kyanite-x86_64.img.zip`: **no** firmware included.
- `kyanite-x86_64-hw.img.zip`: firmware **included**.

The images are distributed inside a zip because they are about 90% smaller compressed: they contain a virtual disk with more than 500 MB of zeros as extra storage. Once extracted they can take up to 1.6 GB.

## Real hardware (the `hw` images)

The *hardware* image set is meant for real PCs and includes `linux-firmware`.

| File | What it is |
|---|---|
| `kyanite-x86_64-hw-live.iso` | Live system with an installer. Write it to a USB stick with Rufus in "DD Image" mode (or balenaEtcher / `dd`). Boot the PC from it in UEFI or BIOS mode. |
| `kyanite-x86_64-hw.img` | The same system already installed on a disk image. Write it to a USB stick with Rufus in "DD Image" mode, or use it as a virtual disk. |

### Before you boot

- **Secure Boot must be disabled** (the bootloader and the kernel are not signed).
- If the NVMe/SATA disk is not visible, switch the SATA mode from RAID/Intel RST to AHCI in the firmware setup.
- Keyboard layout: the GRUB menu has a "keyboard layout" submenu (es, de, fr, it, gb, pt, br-abnt2).

### Installing from the live system

Run `/usr/local/sbin/kyanite-install`. It offers two options:

1. **Erase the whole disk** and install Kyanite (BIOS and UEFI).
2. **Install alongside the existing system** (UEFI only, GPT disk, with an EFI System Partition and 2 GiB of free space). This keeps Windows and your data:
   - Shrink the Windows partition from Windows first (Disk Management).
   - If the disk is encrypted with BitLocker, suspend it first.
   - Kyanite gets its own partition and reuses the existing EFI System Partition. The GRUB menu gets a "Windows" entry. You can also choose Windows or Kyanite from the firmware boot menu.

### Not included

- Secure Boot support.
- Automatic network configuration (DHCP).
- Dual boot with BIOS/MBR Windows.
KYX86EOF
echo "  [CREADO]    docs/x86_64-images.md"; made=$((made+1)); fi
if [ -e "docs/es/x86_64-images.md" ] && [ "${FORCE:-}" != 1 ]; then echo "  [YA EXISTE] docs/es/x86_64-images.md"; else
cat > "docs/es/x86_64-images.md" <<'KYX86EOF'
🇪🇸 Español | [🇺🇸 English](../x86_64-images.md) | [🇷🇺 Русский](../ru/x86_64-images.md) | [🇨🇳 简体中文](../zh-CN/x86_64-images.md)

# Imágenes x86_64

Kyanite también incluye imágenes de disco listas para usar en x86_64: PC, máquinas virtuales y equipos con Windows o Linux. Están publicadas en la release de GitHub [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

¿Buscas los archivos del sistema de archivos raíz (`aarch64` / `x86_64`)? Mira la [guía de instalación](installation.md).

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

## QEMU (BIOS)

<img width="720" height="391" alt="Kyanite GRUB menu in QEMU" src="https://github.com/user-attachments/assets/032289ef-c50f-4e2f-b491-79b66ea7fecb" />

Esta es solo mi configuración; puedes usar cualquier otra.

1. Instala Termux y Termux:X11.
2. Ejecuta en Termux:

```bash
termux-x11 :0 -listen tcp -ac
```

3. Ejecuta en [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) con un rootfs de Alpine (se supone que `qemu-system-x86_64` ya está instalado):

```bash
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. Abre la app Termux:X11 y verás la BIOS.

<img width="720" height="395" alt="Kyanite booting in QEMU" src="https://github.com/user-attachments/assets/5f5197f0-e16f-4dba-b45b-0e99b841d2ba" />

## VirtualBox

Usa `kyanite-x86_64.vdi`.

## VMware

Usa `kyanite-x86_64.vmdk`.

## Rufus (memoria USB)

Extrae primero el zip y graba el `.img` en modo **DD Image**. La imagen vive en su propia unidad virtual.

- `kyanite-x86_64.img.zip`: **sin** firmware incluido.
- `kyanite-x86_64-hw.img.zip`: **con** firmware incluido.

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
KYX86EOF
echo "  [CREADO]    docs/es/x86_64-images.md"; made=$((made+1)); fi
if [ -e "docs/ru/x86_64-images.md" ] && [ "${FORCE:-}" != 1 ]; then echo "  [YA EXISTE] docs/ru/x86_64-images.md"; else
cat > "docs/ru/x86_64-images.md" <<'KYX86EOF'
🇷🇺 Русский | [🇺🇸 English](../x86_64-images.md) | [🇪🇸 Español](../es/x86_64-images.md) | [🇨🇳 简体中文](../zh-CN/x86_64-images.md)

# Образы x86_64

Kyanite также поставляется с готовыми образами дисков для x86_64: для ПК, виртуальных машин и компьютеров с Windows или Linux. Они опубликованы в релизе GitHub [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

Нужны архивы корневой файловой системы (`aarch64` / `x86_64`)? См. [руководство по установке](installation.md).

> ⚠️ Установщик в live-ISO **пока не проверен на реальном оборудовании** и может безвозвратно удалить все ваши файлы. Если вы не уверены в своих действиях, не запускайте его. Сначала сделайте резервную копию данных.

## Какой файл мне нужен?

| Я хочу... | Файл |
|---|---|
| Попробовать Kyanite в QEMU | `kyanite-x86_64.img` |
| Использовать VirtualBox | `kyanite-x86_64.vdi` |
| Использовать VMware | `kyanite-x86_64.vmdk` |
| Записать USB-накопитель через Rufus, **без** прошивок | `kyanite-x86_64.img.zip` |
| Записать USB-накопитель для реального ПК, **с** прошивками | `kyanite-x86_64-hw.img.zip` |
| Обычный live-CD Linux (со скриптом установки) | `kyanite-x86_64-hw-live.iso` |

## QEMU (BIOS)

<img width="720" height="391" alt="Kyanite GRUB menu in QEMU" src="https://github.com/user-attachments/assets/032289ef-c50f-4e2f-b491-79b66ea7fecb" />

Это просто моя конфигурация; можно использовать любую другую.

1. Установите Termux и Termux:X11.
2. Выполните в Termux:

```bash
termux-x11 :0 -listen tcp -ac
```

3. Выполните в [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) с rootfs Alpine (предполагается, что `qemu-system-x86_64` уже установлен):

```bash
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. Откройте приложение Termux:X11, и вы увидите BIOS.

<img width="720" height="395" alt="Kyanite booting in QEMU" src="https://github.com/user-attachments/assets/5f5197f0-e16f-4dba-b45b-0e99b841d2ba" />

## VirtualBox

Используйте `kyanite-x86_64.vdi`.

## VMware

Используйте `kyanite-x86_64.vmdk`.

## Rufus (USB-накопитель)

Сначала распакуйте zip и запишите `.img` в режиме **DD Image**. Образ находится на собственном виртуальном диске.

- `kyanite-x86_64.img.zip`: прошивки **не** включены.
- `kyanite-x86_64-hw.img.zip`: прошивки **включены**.

Образы распространяются в zip-архиве, потому что в сжатом виде они меньше примерно на 90 %: внутри виртуальный диск с более чем 500 МБ нулей в качестве дополнительного пространства. После распаковки они могут занимать до 1,6 ГБ.

## Реальное оборудование (образы `hw`)

Набор образов *hardware* предназначен для реальных ПК и включает `linux-firmware`.

| Файл | Что это |
|---|---|
| `kyanite-x86_64-hw-live.iso` | Live-система с установщиком. Запишите её на USB-накопитель через Rufus в режиме «DD Image» (или через balenaEtcher / `dd`). Загрузите ПК с неё в режиме UEFI или BIOS. |
| `kyanite-x86_64-hw.img` | Та же система, уже установленная в образ диска. Запишите её на USB-накопитель через Rufus в режиме «DD Image» или используйте как виртуальный диск. |

### Перед загрузкой

- **Secure Boot должен быть отключён** (загрузчик и ядро не подписаны).
- Если диск NVMe/SATA не виден, переключите режим SATA с RAID/Intel RST на AHCI в настройках прошивки.
- Раскладка клавиатуры: в меню GRUB есть подменю «keyboard layout» (es, de, fr, it, gb, pt, br-abnt2).

### Установка из live-системы

Запустите `/usr/local/sbin/kyanite-install`. Доступны два варианта:

1. **Стереть весь диск** и установить Kyanite (BIOS и UEFI).
2. **Установить рядом с существующей системой** (только UEFI, диск GPT, наличие системного раздела EFI и 2 ГиБ свободного места). Windows и ваши данные сохраняются:
   - Сначала уменьшите раздел Windows средствами самой Windows (Управление дисками).
   - Если диск зашифрован BitLocker, сначала приостановите защиту.
   - Kyanite получает собственный раздел и использует существующий системный раздел EFI. В меню GRUB появляется пункт «Windows». Windows или Kyanite можно также выбрать в меню загрузки прошивки.

### Не включено

- Поддержка Secure Boot.
- Автоматическая настройка сети (DHCP).
- Двойная загрузка с Windows в режиме BIOS/MBR.
KYX86EOF
echo "  [CREADO]    docs/ru/x86_64-images.md"; made=$((made+1)); fi
if [ -e "docs/zh-CN/x86_64-images.md" ] && [ "${FORCE:-}" != 1 ]; then echo "  [YA EXISTE] docs/zh-CN/x86_64-images.md"; else
cat > "docs/zh-CN/x86_64-images.md" <<'KYX86EOF'
🇨🇳 简体中文 | [🇺🇸 English](../x86_64-images.md) | [🇪🇸 Español](../es/x86_64-images.md) | [🇷🇺 Русский](../ru/x86_64-images.md)

# x86_64 镜像

Kyanite 还提供可直接使用的 x86_64 磁盘镜像，适用于 PC、虚拟机以及 Windows/Linux 主机。镜像发布在 GitHub 的这个 release 中：[x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest)。

想找的是根文件系统压缩包（`aarch64` / `x86_64`）？请看[安装指南](installation.md)。

> ⚠️ live ISO 中自带的安装程序**尚未在真实硬件上测试**，可能会永久删除你的所有文件。如果你不清楚自己在做什么，请不要运行它。请先备份你的数据。

## 我需要哪个文件？

| 我想要…… | 文件 |
|---|---|
| 在 QEMU 中试用 Kyanite | `kyanite-x86_64.img` |
| 使用 VirtualBox | `kyanite-x86_64.vdi` |
| 使用 VMware | `kyanite-x86_64.vmdk` |
| 用 Rufus 写入 U 盘，**不含**固件 | `kyanite-x86_64.img.zip` |
| 为真实 PC 写入 U 盘，**含**固件 | `kyanite-x86_64-hw.img.zip` |
| 常见的 Linux live CD（带安装脚本） | `kyanite-x86_64-hw-live.iso` |

## QEMU（BIOS）

<img width="720" height="391" alt="Kyanite GRUB menu in QEMU" src="https://github.com/user-attachments/assets/032289ef-c50f-4e2f-b491-79b66ea7fecb" />

这只是我自己的配置，你也可以使用其他方式。

1. 安装 Termux 和 Termux:X11。
2. 在 Termux 中运行：

```bash
termux-x11 :0 -listen tcp -ac
```

3. 在 [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) 中，使用 Alpine rootfs 运行（假设已安装 `qemu-system-x86_64`）：

```bash
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. 打开 Termux:X11 应用，即可看到 BIOS。

<img width="720" height="395" alt="Kyanite booting in QEMU" src="https://github.com/user-attachments/assets/5f5197f0-e16f-4dba-b45b-0e99b841d2ba" />

## VirtualBox

使用 `kyanite-x86_64.vdi`。

## VMware

使用 `kyanite-x86_64.vmdk`。

## Rufus（U 盘）

请先解压 zip，再以 **DD 镜像** 模式写入 `.img`。该镜像位于它自己的虚拟磁盘上。

- `kyanite-x86_64.img.zip`：**不含**固件。
- `kyanite-x86_64-hw.img.zip`：**含**固件。

镜像放在 zip 中分发，是因为压缩后体积约小 90%：它们包含一个虚拟磁盘，其中有超过 500 MB 的零作为额外存储空间。解压后最多可能占用 1.6 GB。

## 真实硬件（`hw` 镜像）

*hardware* 镜像套装面向真实 PC，并包含 `linux-firmware`。

| 文件 | 说明 |
|---|---|
| `kyanite-x86_64-hw-live.iso` | 带安装程序的 live 系统。用 Rufus 的“DD 镜像”模式（或 balenaEtcher / `dd`）写入 U 盘，然后用 UEFI 或 BIOS 模式从它启动 PC。 |
| `kyanite-x86_64-hw.img` | 同一系统，已安装在磁盘镜像中。用 Rufus 的“DD 镜像”模式写入 U 盘，或将其用作虚拟磁盘。 |

### 启动之前

- **必须关闭 Secure Boot**（引导程序和内核均未签名）。
- 如果看不到 NVMe/SATA 磁盘，请在固件设置中把 SATA 模式从 RAID/Intel RST 改为 AHCI。
- 键盘布局：GRUB 菜单中有“keyboard layout”子菜单（es、de、fr、it、gb、pt、br-abnt2）。

### 从 live 系统安装

运行 `/usr/local/sbin/kyanite-install`。它提供两种方式：

1. **清除整个磁盘**并安装 Kyanite（支持 BIOS 和 UEFI）。
2. **与现有系统并存安装**（仅限 UEFI、GPT 磁盘，需要有 EFI 系统分区和 2 GiB 空闲空间）。这样可以保留 Windows 和你的数据：
   - 请先在 Windows 中压缩 Windows 分区（磁盘管理）。
   - 如果磁盘使用 BitLocker 加密，请先暂停保护。
   - Kyanite 会获得自己的分区，并复用现有的 EFI 系统分区。GRUB 菜单会增加一个“Windows”条目。你也可以在固件的启动菜单中选择 Windows 或 Kyanite。

### 不包含

- Secure Boot 支持。
- 自动网络配置（DHCP）。
- 与 BIOS/MBR 模式 Windows 的双系统启动。
KYX86EOF
echo "  [CREADO]    docs/zh-CN/x86_64-images.md"; made=$((made+1)); fi
echo "== README (tabla Documentación) =="
python3 - <<'KYX86PY'
import re, sys
ROWS = {'README.md': '| [x86_64 images](docs/x86_64-images.md) | Disk images for PCs and virtual machines (QEMU, VirtualBox, VMware) |', 'README.es.md': '| [Imágenes x86_64](docs/es/x86_64-images.md) | Imágenes de disco para PC y máquinas virtuales (QEMU, VirtualBox, VMware) |', 'README.ru.md': '| [Образы x86_64](docs/ru/x86_64-images.md) | Образы дисков для ПК и виртуальных машин (QEMU, VirtualBox, VMware) |', 'README.zh-CN.md': '| [x86_64 镜像](docs/zh-CN/x86_64-images.md) | 适用于 PC 和虚拟机（QEMU、VirtualBox、VMware）的磁盘镜像 |'}
for name, row in ROWS.items():
    try:
        raw = open(name, encoding="utf-8", newline="").read()
    except FileNotFoundError:
        print("  [NO ESTÁ]   %s (no lo toco)" % name); continue
    if "x86_64-images.md" in raw:
        print("  [YA TIENE]  %s" % name); continue
    m = re.search(r"^\| \[[^\]]*\]\(docs/(?:es/|ru/|zh-CN/)?installation\.md\) \|[^\r\n]*(\r?\n)", raw, re.M)
    if not m:
        print("  [SIN FILA]  %s: no encuentro la fila de 'Installation' en la tabla; añade a mano: %s" % (name, row)); continue
    out = raw[:m.end()] + row + m.group(1) + raw[m.end():]
    open(name, "w", encoding="utf-8", newline="").write(out)
    print("  [FILA AÑADIDA] %s" % name)
KYX86PY
echo; echo "Hecho: $made documento(s) creado(s). Revisa con: git diff --stat && git status --short"
