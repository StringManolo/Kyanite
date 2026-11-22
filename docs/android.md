# Kyanite on Android

This page is a beginner-friendly summary of the Kyanite app for Android. The [KyaniteAndroid repository](https://github.com/StringManolo/KyaniteAndroid) is the authoritative source and has the full, up-to-date details.

## What is it?

An Android app that runs Kyanite with a built-in terminal. It uses PRoot, so no special permissions are needed. The Kyanite system is shipped inside the app.

## Install

1. [Download the latest APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk).
2. Open the file and install it. Android may ask you to allow installing from this source.
3. Updates always need your confirmation. They must be signed with the same key as the installed version.

## First launch

Nothing is installed or started until you choose. The first launch opens the settings screen (the ⚙ button is always available).

## Ways to run Kyanite

| Mode | What happens |
| --- | --- |
| **Normal** | Your changes are kept between sessions. |
| **Ephemeral** | Everything is erased when you exit. It uses temporary storage on disk, so remnants may remain on flash storage. |
| **VM** | Experimental. Runs in an emulated machine (QEMU): much slower, but writes nothing to disk. |

You can also choose between **System** (your installed Kyanite) and **LiveCD** (the system shipped with the app, like a live CD).

## Window and keyboard

- **Fullscreen** or **floating window** (needs the "Display over other apps" permission). The floating window can be moved, resized, rotated, pinned, made transparent and minimized.
- An **extra keys bar** provides ESC, TAB, CTRL, ALT, arrows and more. Tap a modifier key once to use it once; long-press to lock it.
- Choose colour schemes and font size in the settings. Pinch to zoom the text.

## Privacy and security options

- PIN lock.
- Encryption of stored systems.
- Options to block screenshots, hide notifications and clear the clipboard.
- A switch to cut the app's network access.
- Scheduled wipe of the data.

## Good to know

- PRoot is **not** a security boundary. For stronger isolation, use VM mode.
- Android 12 and later may kill background processes. If your sessions end in the background, look for the "phantom process killer" setting.
- Once inside, use `kpkg` to install software. See the [kpkg guide](kpkg.md).
