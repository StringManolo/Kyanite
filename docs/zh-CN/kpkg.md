# kpkg

`kpkg` 是 Kyanite 的包管理器。它提供 apk 风格的命令行界面，但从 Debian 软件源安装软件包。它还提供与 apt 兼容的模式。

## 快速开始

```sh
kpkg update
kpkg search curl
kpkg add curl
kpkg info curl
```

## 命令

| 命令 | 说明 |
| --- | --- |
| `kpkg update` | 获取并验证软件包索引；检查是否有 Kyanite 基础系统更新 |
| `kpkg search <关键词>` | 在软件包名称和描述中搜索 |
| `kpkg add <软件包>...` | 安装软件包及其依赖 |
| `kpkg del <软件包>...` | 卸载软件包 |
| `kpkg purge <软件包>...` | 卸载软件包，包括已修改的配置文件 |
| `kpkg autoremove` | 移除不再需要的依赖 |
| `kpkg upgrade` | 应用待处理的 Kyanite 基础系统更新，并升级已安装的软件包 |
| `kpkg fix <软件包>...` | 重新安装软件包以进行修复 |
| `kpkg info [软件包]` | 显示软件包信息 |
| `kpkg list [模式]` | 列出可用的软件包 |
| `kpkg policy <软件包>...` | 显示已安装版本和候选版本 |
| `kpkg configure [软件包]...` | 重新运行软件包的配置钩子 |
| `kpkg audit` | 用记录的 SHA256 校验和核对已安装的文件 |
| `kpkg cache clean` | 删除缓存的软件包文件 |
| `kpkg cache size` | 显示缓存占用 |
| `kpkg base` | 显示 Kyanite 基础系统版本和待处理的更新信息 |

`kpkg info` 还支持针对文件和依赖的附加查询：

- `-L`：列出文件
- `-W`：显示某个路径属于哪个软件包
- `-R`：显示依赖
- `-r`：显示反向依赖

要了解你所用版本支持的确切语法，请运行：

```sh
kpkg --help
```

## 与 APT 兼容的模式

Kyanite 提供 `apt`、`apt-get` 和 `apt-cache`，它们都是指向 `kpkg` 的符号链接。

以这些名称之一调用，或设置了 `KPKG_SYNTAX=apt` 时，程序会显示 apt 风格的帮助，并在做出更改前请求确认。使用 `-y` 可跳过确认。

```sh
apt update
apt install curl
apt remove curl
apt autoremove
apt list --installed
apt show curl
```

像 `alias apt=kpkg` 这样的 shell 别名不会改变可执行文件的名称。如需通过别名启用 apt 模式，请使用：

```sh
alias apt='KPKG_SYNTAX=apt kpkg'
```

## 选项

| 选项 | 说明 |
| --- | --- |
| `-s`、`--simulate` | 只显示计划执行的操作，不做任何更改 |
| `-f`、`--force` | 覆盖无归属或外来的文件，并忽略反向依赖 |
| `-y`、`--yes` | 在 apt 模式下跳过确认提示 |
| `--root <目录>` | 对另一个根目录进行操作；默认为 `/` |
| `--arch <架构>` | 设置 Debian 架构 |
| `--mirror <url>` | 设置 Debian 软件源镜像 |
| `--security-mirror <url>` | 设置 Debian 安全软件源镜像 |
| `--suite <名称>` | 设置 Debian suite；默认为 `stable` |
| `--keyring <文件>` | 使用指定的 OpenPGP 密钥环 |
| `--no-security` | 禁用 `<suite>-security` 软件源 |

## 软件源与软件包选择

软件包索引由 Debian 的 `stable`、`stable-updates` 和 `stable-security` 构建而成。对于每个软件包，`kpkg` 会选择可用的最高版本。

软件源元数据通过 OpenPGP 验证。软件包下载后，会先依据已签名的索引检查大小和 SHA256，再写入最终位置。

依赖解析支持：

- `Pre-Depends`
- `Depends`，包括备选项（`a | b`）和版本约束
- 通过 `Provides` 提供的虚拟软件包
- `Conflicts` 和 `Breaks`

版本比较遵循 dpkg 算法。

## 软件包配置

Kyanite 使用声明式钩子来配置软件包。有关钩子的位置、支持的操作和开发说明，请阅读[包钩子指南](package-hooks.md)。

## 软件包状态与缓存

包管理器的状态保存在 `/var/lib/kpkg`。下载的 `.deb` 文件缓存在 `/var/cache/kpkg/pkgs`。

基础根文件系统中包含的软件包列在 `/etc/kpkg/assume-installed` 中，这样 `kpkg` 就不会再次安装它们。

<a id="updating-the-kyanite-base"></a>

## 更新 Kyanite 基础系统

`kpkg update` 会检查 GitHub 上最新的符合条件的 Kyanite 发布版。`kpkg upgrade` 会应用待处理的基础系统更新，并升级已安装的软件包。

只考虑形如 `vX.Y.Z` 的发布标签。以提交哈希或 `git describe` 输出标识的开发构建，不符合基础系统自动更新的条件。

更新程序会验证发布版签名和 SHA256 校验和，先解压到临时目录，再依据清单应用文件。本地修改过的文件会被保留；新版本会以 `.kyanite-new` 后缀保存在它们旁边。

发布版中的 `CHANGELOG` 记录了那些无法用普通文件更新表示的更改所需的操作。
