# Releases

Kyanite publishes root filesystem archives for each supported architecture through GitHub Releases.

## Version tags

Release tags use the `vX.Y.Z` format. For example:

```sh
git tag -a v0.1.0 -m "Kyanite 0.1.0"
git push origin v0.1.0
```

Pushing a matching tag triggers the release workflow. Other tag formats are not used by the automatic base-update mechanism.

## Release files

Each release contains the following files for each architecture:

- `kyanite-rootfs-<arch>.tar.xz`
- `kyanite-rootfs-<arch>.tar.xz.sha256`
- `kyanite-rootfs-<arch>.tar.xz.sha256.sig`

The `.sig` file contains an Ed25519 signature of the `.sha256` file.

## Signing key

The public release key is stored in:

```text
rootfs-overlay/etc/kpkg/kyanite-release.pub
```

The corresponding private key is stored as the GitHub repository secret:

```text
KYANITE_SIGNING_KEY
```

Never commit or publish the private signing key.

For user-side verification, see [Verifying releases](../SECURITY.md#verifying-releases).

## Changelog

When a release requires an operation that cannot be represented as a normal file update—for example, replacing a preinstalled package—add a section for the new version to:

```text
rootfs-overlay/usr/share/kyanite/CHANGELOG
```

The changelog describes the operations the updater can perform for these transitions.

## Development builds

Builds from commits outside a release tag are labelled using `git describe`, for example:

```text
v0.1.0-3-g1a2b3c4
```

Development builds are not eligible for automatic base updates. The updater considers only `vX.Y.Z` release tags.

## Android application

The Android application has a separate repository and release process:

- [KyaniteAndroid releases](https://github.com/StringManolo/KyaniteAndroid/releases)
- [Download the latest APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
