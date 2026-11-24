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
