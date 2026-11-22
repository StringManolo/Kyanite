# 已知限制与兼容性说明

本页汇总了当前可能影响兼容性或预期行为的实现细节，旨在作为技术参考。

## 运行时与平台

- Kyanite 以根文件系统的形式分发，目前不包含自己的内核。
- 当前运行时使用 PRoot，并与宿主共享内核。
- 该 rootfs 已通过 [DebianLiveCD4Android](https://github.com/StringManolo/DebianLiveCD4Android) 测试。
- `proot-distro`、QEMU 和 udocker 仍需进一步测试。
- 在 Android 8 及更高版本上，应用可能无法读取 `/proc` 下的某些文件。启动器可能需要绑定替代文件。

## 包管理器

- `kpkg` 不会执行 Debian 维护者脚本（`preinst`、`postinst`、`prerm`、`postrm`）。需要这些操作的软件包需要 Kyanite 钩子或手动设置。
- `Recommends` 和 `Suggests` 不会自动安装。
- 预装的基础组件通过 Kyanite 发布版更新，而不是通过普通的软件包升级。
- 目前的构建从 Ubuntu 构建环境获取部分运行时组件，而 `kpkg` 从 Debian stable 安装软件包，二者的版本可能不同。

## 证书

原生实现的 `update-ca-certificates` 会生成 `/etc/ssl/certs/ca-certificates.crt`。它不会创建依赖 `CApath` 的应用所使用的 `<hash>.0` 链接。

## Shell 与 init 的行为

`/bin/sh` 指向 Kyanite init 可执行文件。这是当前启动设计的一部分。因此，如果软件包脚本在预期的启动路径之外调用 `/bin/sh`，它可能是在与 init 而不是传统 shell 交互。MOTD 脚本明确使用 `/bin/brush` 运行。

shell 兼容性，以及假定存在传统 `/bin/sh` 的软件包的行为，都是有待进一步测试的方面。

## 安全与隔离

PRoot 不是虚拟机，也不是独立的内核级安全边界。Kyanite 环境与宿主共享内核，并依赖宿主启动器及其配置。

有关威胁模型、防护措施和安全问题报告方式，请参见 [SECURITY.md](../../SECURITY.md)（英文）。
