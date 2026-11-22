# 开发

本指南介绍 Kyanite 的源码结构和构建流程。具体的构建命令和依赖版本，以 GitHub Actions 工作流为准。

## 构建流程

构建定义在：

```text
.github/workflows/build-rootfs.yml
```

目标平台：

- `aarch64-unknown-linux-musl`
- `x86_64-unknown-linux-musl`

概括来说，工作流会：

1. 用 `git describe --tags --always` 计算版本号。
2. 用 Cargo 构建 Kyanite init 和 `kpkg`。
3. 为 musl 目标构建 Brush 和 uutils coreutils。
4. 组装根文件系统。
5. 添加运行时库和辅助软件包。
6. 应用 `rootfs-overlay/`。
7. 生成基础系统清单。
8. 创建压缩包和 SHA256 校验和。
9. 在发布构建中为校验和签名。

## 构建前提

目前文档中列出的本地前提条件有：

- Rust 工具链
- 所需的 Rust musl 目标
- `musl-tools`
- 用于 aarch64 交叉编译的 `gcc-aarch64-linux-gnu`

请查看工作流，了解当前构建是否还需要其他工具或软件包。

## 根文件系统覆盖层

`rootfs-overlay/` 目录包含会被复制到生成的根文件系统中的文件，包括配置文件和 shell 环境文件。

修改覆盖层时，请检查文件路径、权限，以及这些更改与基础系统清单和更新机制之间的相互影响。

## 测试更改

请使用一次性的根文件系统来测试更改。

如果你在开发包管理器，请测试相关的安装、升级、卸载、修复、配置和审计流程。如果你在开发基础系统更新，请同时测试未修改的文件和被用户改动过的文件。

## 版本标签

构建使用 `git describe --tags --always`。发布标签之后的提交可能会被标记为类似 `v0.1.0-3-g1a2b3c4` 的值。如果没有可用的标签，版本号可能就是提交哈希。

只有形如 `vX.Y.Z` 的发布标签才有资格进行基础系统的自动更新。

## 相关文档

- [架构](architecture.md)
- [包管理器](kpkg.md)
- [包钩子](package-hooks.md)
- [发布流程](releases.md)
- [路线图与调研](roadmap.md)
