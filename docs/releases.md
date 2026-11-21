# Releases

Kyanite publishes root filesystem archives for each supported
architecture through GitHub Releases.

## Version tags

A release tag must match:

``` text
vX.Y.Z
```

For example:

``` sh
git tag -a v0.1.0 -m "Kyanite 0.1.0"
git push origin v0.1.0
```

Pushing a matching tag triggers the release workflow. Other tag formats
are ignored by the automatic base-update mechanism.

## Release artifacts

Each release contains, per architecture:

-   `kyanite-rootfs-<arch>.tar.xz`
-   `kyanite-rootfs-<arch>.tar.xz.sha256`
-   `kyanite-rootfs-<arch>.tar.xz.sha256.sig`

The `.sig` file is an Ed25519 signature of the `.sha256` file.

The release workflow builds both supported architectures, generates
checksums, signs the checksums and publishes the archives.

## Signing key

Before publishing signed releases, generate a signing key pair as
described in:

``` text
rootfs-overlay/etc/kpkg/kyanite-release.pub
```

Place the public key in that file and store the private key in the
GitHub repository secret named:

``` text
KYANITE_SIGNING_KEY
```

Never commit or publish the private signing key.

For user-side verification instructions, see
[SECURITY.md](../SECURITY.md#verifying-releases).

## Changelog

If a release requires changes that cannot be represented as ordinary
file updates---for example, replacing one preinstalled package with
another---add a section for the new version to:

``` text
rootfs-overlay/usr/share/kyanite/CHANGELOG
```

The changelog documents the operations the updater can perform for these
transitions.

## Development builds

Builds from commits that are not release tags are labelled using
`git describe`, for example:

``` text
v0.1.0-3-g1a2b3c4
```

or a commit hash when no tag exists.

Development builds are not automatically updated by `kpkg`. The updater
considers only release tags matching `vX.Y.Z`.

## Android application

The Android application has its own repository and release process:

-   [KyaniteAndroid
    releases](https://github.com/StringManolo/KyaniteAndroid/releases)
-   [Download latest
    APK](https://github.com/StringManolo/KyaniteAndroid/releases/latest/download/Kyanite.apk)
