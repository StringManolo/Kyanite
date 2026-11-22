# 发布

Kyanite 通过 GitHub Releases 为每个受支持的架构发布根文件系统压缩包。

## 版本标签

发布标签使用 `vX.Y.Z` 格式。例如：

```sh
git tag -a v0.1.0 -m "Kyanite 0.1.0"
git push origin v0.1.0
```

推送符合格式的标签会触发发布工作流。其他格式的标签不会被基础系统自动更新机制使用。

## 发布文件

每个发布版针对每种架构包含以下文件：

- `kyanite-rootfs-<架构>.tar.xz`
- `kyanite-rootfs-<架构>.tar.xz.sha256`
- `kyanite-rootfs-<架构>.tar.xz.sha256.sig`

`.sig` 文件包含对 `.sha256` 文件的 Ed25519 签名。

## 签名密钥

公开的发布密钥存放在：

```text
rootfs-overlay/etc/kpkg/kyanite-release.pub
```

对应的私钥存储为 GitHub 仓库的密钥（secret）：

```text
KYANITE_SIGNING_KEY
```

切勿提交或公开私有签名密钥。

用户侧的验证方法，请参见[验证发布版](../../SECURITY.md#verifying-releases)（英文）。

## Changelog

当某个发布版需要无法表示为普通文件更新的操作时（例如替换预装软件包），请在以下文件中为新版本添加一节：

```text
rootfs-overlay/usr/share/kyanite/CHANGELOG
```

该 changelog 描述了更新程序在这类过渡中可以执行的操作。

## 开发构建

来自发布标签之外的提交的构建，会用 `git describe` 标记，例如：

```text
v0.1.0-3-g1a2b3c4
```

开发构建不符合基础系统自动更新的条件。更新程序只考虑 `vX.Y.Z` 发布标签。

## Android 应用

Android 应用有独立的仓库和发布流程：

- [KyaniteAndroid 发布版](https://github.com/StringManolo/KyaniteAndroid/releases)
- [下载最新的 APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
