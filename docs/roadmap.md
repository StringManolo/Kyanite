# Roadmap and investigations

This page summarizes areas of possible future work. It is not a fixed release plan: priorities may change as development and testing reveal what is most useful.

## User experience

- Continue improving the default Brush configuration and login experience.
- Improve init logging to make startup and runtime issues easier to diagnose.
- Review the default MOTD and shell configuration.
- Consider adding commonly expected utilities that are not currently included in the base.
- Explore a recycle-bin feature and cloud-backup options.

## Package compatibility

- Expand testing across commonly used software and identify packages that need additional hooks.
- Continue testing development tools and command-line applications.
- Investigate compatibility issues where applications detect or assume a different Linux environment.
- Review how alternative shells interact with Kyanite's init and login flow.

## Platform support and stability

- Continue investigating QEMU startup and terminal handling.
- Test the rootfs on additional environments, including `proot-distro` and udocker.
- Improve the installation experience for `proot-distro`.
- Continue testing the x86_64 environment and other host configurations.
- Explore kernel-related work and alternative boot or deployment approaches.

## Security and maintenance

- Review the files shipped in the root filesystem.
- Continue reviewing package-manager security behavior and repository version selection.
- Investigate a Rust-based replacement for the remaining GNU `env` component.
- Review the balance between static musl binaries and dynamically linked components.

## Size and dependencies

- Review the size and resource usage of bundled components such as Brush and uutils coreutils.
- Explore whether smaller alternatives or targeted optimizations would be worthwhile.

## Other ideas

Some ideas are exploratory and may not become part of Kyanite:

- X11 support
- Additional kernel and device workflows
- A separate security-focused distribution built using Kyanite-related tooling

The project README describes the current user-facing capabilities. This page records areas under consideration, not promises or release commitments.
