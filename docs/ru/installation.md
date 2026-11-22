# Установка

Kyanite распространяется в виде архива корневой файловой системы Linux. Её также можно использовать через специальное приложение для Android.

## Android

Самый простой способ начать на устройстве с Android — установить приложение Kyanite.

- [Скачать последний APK Kyanite](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
- [Исходный код и документация KyaniteAndroid](https://github.com/StringManolo/KyaniteAndroid)

Приложение для Android предлагает собственный процесс настройки и запуска. См. также [руководство по Android](android.md).

## Загрузка корневой файловой системы Linux

Откройте [страницу релизов Kyanite](https://github.com/StringManolo/Kyanite/releases) и скачайте архив для вашей архитектуры:

| Архитектура | Архив |
| --- | --- |
| ARM64 | `kyanite-rootfs-aarch64.tar.xz` |
| AMD64 | `kyanite-rootfs-x86_64.tar.xz` |

Скачайте также соответствующие файлы `.sha256` и `.sha256.sig`.

## Проверка загрузки

Проверьте контрольную сумму SHA256 в каталоге со скачанными файлами:

```sh
sha256sum -c kyanite-rootfs-aarch64.tar.xz.sha256
```

Используйте имя файла контрольной суммы, соответствующее вашему архиву.

Чтобы проверить ещё и подпись Ed25519, следуйте разделу [Проверка релизов](../../SECURITY.md#verifying-releases) (на английском).

## Распаковка корневой файловой системы

Создайте каталог и распакуйте в него архив:

```sh
mkdir kyanite
tar -xpf kyanite-rootfs-aarch64.tar.xz -C kyanite
```

Замените имя архива на то, которое соответствует вашей архитектуре.

## Запуск с помощью PRoot

Базовая команда запуска выглядит так:

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

В зависимости от окружения хоста загрузчику могут понадобиться дополнительные привязки (`-b`). На Android некоторые файлы в `/proc` могут требовать особой обработки.

## Поведение при запуске

`/bin/sh` — символическая ссылка на `/sbin/init`. Когда загрузчик запускает `/bin/sh`, Kyanite init запускает Brush как интерактивную login-оболочку.

Подробнее об оболочке и окружении входа — в [руководстве по использованию](usage.md).
