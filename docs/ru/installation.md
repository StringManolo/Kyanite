# Установка

Kyanite работает на Android, Windows, Linux, в Termux и в Docker. Выберите вашу систему:

| У вас... | Перейти |
| --- | --- |
| Телефон на Android (проще всего) | [Android](#android) |
| Компьютер с Windows | [Windows](#windows) |
| Компьютер с Linux | [Linux](#linux) |
| Termux на Android | [Termux](#termux) |
| Docker | [Docker](#docker) |

## Какие есть файлы?

Kyanite публикуется на [странице релизов Kyanite](https://github.com/StringManolo/Kyanite/releases). Приложение для Android публикуется отдельно, в [релизах KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid/releases/latest).

| Что | Файлы | Для чего |
| --- | --- | --- |
| Архив корневой файловой системы | `kyanite-rootfs-aarch64.tar.xz` (ARM64), `kyanite-rootfs-x86_64.tar.xz` (AMD64), `kyanite-rootfs-armv7.tar.xz` (32-битный ARM) | Linux, Termux и Docker. Kyanite работает внутри папки, без виртуальной машины. |
| Образы дисков x86_64 | `.img`, `.vdi`, `.vmdk`, `.iso` | Виртуальные машины и USB-накопители. См. [Образы x86_64](x86_64-images.md). |
| Приложение для Android | `.apk` для 64-битных и 32-битных устройств | Android |

## Android

Проще всего начать на устройстве с Android — установить приложение Kyanite. Скачайте версию для вашего телефона:

- [Kyanite для Android, 64-битная](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk): большинство телефонов.
- [Kyanite для Android, 32-битная](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/kyanite_v7.apk): старые телефоны.

Не уверены, какую выбрать? Сначала попробуйте 64-битную. Если Android не даёт её установить, возьмите 32-битную. У некоторых телефонов 64-битный процессор, но 32-битная система Android, и им нужна 32-битная версия. [Сайт Kyanite](https://stringmanolo.github.io/KyaniteAndroid/ru/) тоже подскажет подходящую. В 32-битной версии нет режима виртуальной машины.

- [Исходный код и документация KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid)

У приложения для Android свой процесс настройки и запуска. См. также [руководство по Android](android.md).

## Windows

В Windows Kyanite работает как виртуальная машина или запускается с USB-накопителя на настоящем компьютере. Скачайте образы дисков x86_64 в релизе [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest). Что за файл что означает — см. [Образы x86_64](x86_64-images.md).

### VirtualBox

1. Скачайте `kyanite-x86_64.vdi` и установите [VirtualBox](https://www.virtualbox.org/).
2. Нажмите **Создать**. Назовите машину `Kyanite`, выберите тип **Linux** и версию **Other Linux (64-bit)**. Если мастер предлагает автоматическую установку, пропустите её.
3. Выделите не менее 1024 МБ памяти.
4. На шаге с жёстким диском выберите **Использовать существующий виртуальный жёсткий диск**, нажмите значок папки, затем **Добавить** и выберите `kyanite-x86_64.vdi`.
5. Завершите работу мастера, выберите машину и нажмите **Запустить**.

### VMware

1. Скачайте `kyanite-x86_64.vmdk` и установите VMware Workstation или VMware Player.
2. Выберите **File → New Virtual Machine** и затем **Custom (advanced)**.
3. Выберите **I will install the operating system later**. В качестве гостевой системы выберите **Linux** и **Other Linux 64-bit** (точное название меняется в разных версиях VMware).
4. Выделите не менее 1 ГБ памяти.
5. Когда мастер спросит про диск, выберите **Use an existing virtual disk**, укажите `kyanite-x86_64.vmdk` и сохраните существующий формат, если об этом спросят.
6. Завершите настройку и нажмите **Power on this virtual machine**.

### QEMU

1. Установите QEMU для Windows с [официальной страницы загрузки](https://www.qemu.org/download/#windows).
2. Скачайте `kyanite-x86_64.img` (или `kyanite-x86_64.img.zip` и распакуйте его).
3. Откройте PowerShell в папке с образом и выполните:

```powershell
& "C:\Program Files\qemu\qemu-system-x86_64.exe" -m 1G -drive file=kyanite-x86_64.img,format=raw
```

Без ускорения работает медленно. По желанию включите компонент Windows *Windows Hypervisor Platform* и добавьте в команду `-accel whpx`.

### Rufus (настоящий компьютер с USB-накопителя)

1. Скачайте `kyanite-x86_64-hw-live.iso` (live-система с установщиком) или `kyanite-x86_64-hw.img.zip` (сначала распакуйте) и скачайте [Rufus](https://rufus.ie/).
2. Вставьте USB-накопитель. **Всё, что на нём есть, будет стёрто.**
3. Откройте Rufus. В поле **Устройство** выберите свой USB-накопитель, нажмите **ВЫБРАТЬ** и укажите файл (для `.img` в диалоге выберите «Все файлы»).
4. Нажмите **СТАРТ**. Когда Rufus спросит, выберите **режим образа DD** и подтвердите.
5. Перезагрузите компьютер, откройте меню загрузки и запустите систему с USB-накопителя.

Перед запуском на настоящем компьютере прочитайте [Образы x86_64](x86_64-images.md): Secure Boot должен быть отключён, а установщик пока не проверен на реальном оборудовании.

## Linux

### PRoot (без root)

PRoot запускает Kyanite от имени обычного пользователя, внутри папки. Он работает в большинстве систем Linux (хост должен разрешать `ptrace`).

1. Установите `proot` через менеджер пакетов, например `sudo apt install proot` в Debian или Ubuntu.
2. Откройте [страницу релизов Kyanite](https://github.com/StringManolo/Kyanite/releases) и скачайте архив для вашей архитектуры:

| Архитектура | Архив |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |
| 32-битный ARM | `kyanite-rootfs-armv7.tar.xz` |

Скачайте также соответствующие файлы `.sha256` и `.sha256.sig`.

3. Проверьте контрольную сумму SHA256 в каталоге, где лежат скачанные файлы:

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Используйте имя файла контрольной суммы, которое соответствует скачанному архиву. Чтобы дополнительно проверить подпись Ed25519, следуйте разделу [Проверка релизов](../../SECURITY.md#verifying-releases) (на английском).

4. Создайте каталог и распакуйте в него архив (замените имя архива на то, которое соответствует вашей архитектуре):

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

5. Запустите Kyanite. Базовая команда запуска выглядит так:

```sh
proot --kill-on-exit --link2symlink -0 -r kyanite \
  -b /dev -b /proc -b /sys \
  -w /root \
  /usr/bin/env -i \
    HOME=/root USER=root LOGNAME=root \
    TERM=xterm-256color COLORTERM=truecolor LANG=C.UTF-8 \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    /bin/sh -l
```

В зависимости от среды хоста лаунчеру могут понадобиться дополнительные привязки (`-b`).

### QEMU

QEMU запускает образ диска x86_64 как виртуальную машину.

```sh
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw
```

Добавьте `-enable-kvm`, если это поддерживают ваш процессор и ядро: так намного быстрее. Образ находится в релизе [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest).

### USB-накопитель (настоящий компьютер)

Скачайте `kyanite-x86_64-hw-live.iso` (или `kyanite-x86_64-hw.img.zip` и распакуйте его) в релизе [x86_64 images for PC/Windows/Linux, QEMU, VirtualBox, VMware, etc.](https://github.com/StringManolo/Kyanite/releases/tag/qemuTest) и запишите на USB-накопитель через balenaEtcher или `dd`:

```sh
lsblk                                     # найдите свой USB-накопитель, например /dev/sdX
sudo dd if=kyanite-x86_64-hw-live.iso of=/dev/sdX bs=4M status=progress conv=fsync
```

Замените `/dev/sdX` на свой USB-накопитель. **Всё на нём будет стёрто, а при ошибке в выборе устройства можно уничтожить свои данные.** Перед запуском прочитайте [Образы x86_64](x86_64-images.md).

## Termux

Нужен [Termux](https://termux.dev/), установленный из F-Droid или со страницы релизов Termux на GitHub.

### proot-distro (рекомендуется)

Требуется PRoot-Distro версии 5 или новее. Быстрее всего — скрипт установки: он скачивает подходящий для телефона архив, проверяет его контрольную сумму SHA256, устанавливает Kyanite и создаёт команду `kyanite`.

```sh
pkg update && pkg upgrade
curl -fsSL https://raw.githubusercontent.com/StringManolo/Kyanite/main/install-termux.sh -o install-termux.sh
sh install-termux.sh
```

Затем введите `kyanite`, чтобы открыть оболочку, или `kyanite КОМАНДА`, чтобы выполнить одну команду.

То же самое по шагам:

```sh
pkg install proot-distro curl
ARCH=aarch64        # на 32-битном телефоне используйте armv7
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz
curl -LO https://github.com/StringManolo/Kyanite/releases/latest/download/kyanite-rootfs-$ARCH.tar.xz.sha256
sha256sum -c kyanite-rootfs-$ARCH.tar.xz.sha256
proot-distro install ./kyanite-rootfs-$ARCH.tar.xz --name kyanite
proot-distro login kyanite
```

Чтобы создать команду `kyanite` вручную:

```sh
cat > "$PREFIX/bin/kyanite" <<LAUNCHER
#!$PREFIX/bin/sh
if [ "\$#" -eq 0 ]; then
  exec proot-distro login kyanite
else
  exec proot-distro login kyanite -- "\$@"
fi
LAUNCHER
chmod 755 "$PREFIX/bin/kyanite"
```

Чтобы удалить: `proot-distro remove kyanite && rm "$PREFIX/bin/kyanite"`.

### QEMU

Это экспериментальный и медленный способ: QEMU эмулирует целый компьютер, чтобы запустить образ x86_64.

```sh
pkg install qemu-system-x86-64-headless unzip curl
curl -LO https://github.com/StringManolo/Kyanite/releases/download/qemuTest/kyanite-x86_64.img.zip
unzip kyanite-x86_64.img.zip
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -vnc 127.0.0.1:0
```

Откройте на телефоне приложение-клиент VNC и подключитесь к `127.0.0.1:5900`.

Если нужно графическое окно вместо VNC, вот собственная конфигурация автора:

1. Установите Termux и Termux:X11.
2. Выполните в Termux: `termux-x11 :0 -listen tcp -ac`
3. В [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) с rootfs Alpine выполните:

```sh
apk add qemu-ui-gtk
unset SDL_VIDEODRIVER
export DISPLAY=127.0.0.1:0
qemu-system-x86_64 -m 1G -drive file=kyanite-x86_64.img,format=raw -display gtk -vga std
```

4. Откройте приложение Termux:X11.

## Docker

Это экспериментальный способ. Используйте архив, который соответствует вашему процессору (`aarch64` на ARM-компьютерах, например на Mac с Apple Silicon). Сначала скачайте его и проверьте контрольную сумму, как в шагах PRoot в разделе [Linux](#linux).

```sh
docker import kyanite-rootfs-x86_64.tar.xz kyanite
docker run --rm -it kyanite /bin/sh -l
```

## Поведение при запуске

`/bin/sh` — это символическая ссылка на `/sbin/init`. Когда лаунчер запускает `/bin/sh`, Kyanite init запускает Brush как интерактивную login-оболочку.

Подробнее об оболочке и окружении входа — в [руководстве по использованию](usage.md).
