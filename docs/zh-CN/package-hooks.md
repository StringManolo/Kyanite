# 包钩子

Kyanite 使用声明式钩子来配置软件包，而不是执行 Debian 软件包的维护者脚本。

钩子描述了一组有限的操作，`kpkg` 可以在目标根文件系统内执行这些操作。钩子还可以添加或忽略软件包依赖。

## 钩子的位置

系统自带的钩子存放在：

```text
/usr/share/kpkg/hooks/<软件包>.hook
```

本地的钩子覆盖文件存放在：

```text
/etc/kpkg/hooks.d/
```

## 支持的操作

钩子引擎目前提供一组封闭的操作：

- `mkdir`
- `symlink`
- `link-default`
- `copy-default`
- `remove`
- `chmod`
- `builtin`

钩子还可以添加或忽略依赖。

这些操作是声明式的；钩子不是任意的 shell 脚本。

## 示例：CA 证书

`ca-certificates` 软件包使用 Kyanite 的原生功能来重新生成：

```text
/etc/ssl/certs/ca-certificates.crt
```

`curl` 的钩子添加了对 `ca-certificates` 的依赖，因此安装 curl 时证书包也是可用的。

## 重新运行钩子

用以下命令为已安装的软件包运行配置钩子：

```sh
kpkg configure
```

配置指定的软件包：

```sh
kpkg configure curl ca-certificates
```

## 开发钩子

添加钩子时，先弄清该软件包的 Debian 维护者脚本执行了哪些设置。判断其中哪些是 Kyanite 需要的，然后用受支持的钩子操作或已有的原生 `builtin` 来表达这些操作。

一个有用的测试流程包括：

1. 将软件包安装到干净的 rootfs 中。
2. 检查生成的文件和配置。
3. 再次运行 `kpkg configure`。
4. 卸载软件包，并检查清理行为。
5. 测试钩子引入的任何依赖。

确切的语法请参考源码树中的钩子示例和实现。本指南只列出了操作的类别，并未定义解析器的语法。
