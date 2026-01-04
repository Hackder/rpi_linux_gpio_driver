# Raspberry Pi Linux GPIO Driver with Rust

This repository contains a custom Linux kernel build for the Raspberry Pi (specifically targeting ARMv6/Pi Zero W) with enabled Rust support, a custom Rust GPIO driver, and a minimal userspace based on Busybox.

## Quickstart

To build the complete SD card image, simply run the build script:

```bash
./build_image.sh
```

This command triggers a Docker-based build process. Once completed, the final artifacts, including the SD card image, will be available in the `./output` directory.

---

# Kernel Build with Rust Support

This section documents the process for building the Raspberry Pi Linux kernel with Rust support enabled, specifically targeting ARMv6 (Raspberry Pi Zero/1), as defined in the `Dockerfile`.

## Build Environment
The build is performed in an Ubuntu 24.04 container with the following key components:
*   **Rust Toolchain**: Version 1.84.1 is installed with the `arm-unknown-linux-gnueabi` target.
*   **LLVM & Clang**: Installed to support compiling the kernel with `LLVM=1`.
*   **Bindgen**: `bindgen-cli` (v0.72.1) is used to generate C bindings for Rust.

## Kernel Patches & ARMv6 Support
The kernel source (branch `rpi-6.12.y`) is patched to enable Rust support for ARMv6, which is not officially supported upstream. The patch `linux/0001-rust-support-for-armv6.patch` performs the following critical modifications:

*   **Enabling Rust**: Adds `select HAVE_RUST` to `arch/arm/Kconfig`.
*   **Linking libgcc**: Modifies `arch/arm/Makefile` to link against `libgcc`. This is required because the Rust compiler expects symbols like `__aeabi_uldivmod` (used for 64-bit division) to be defined, which are provided by this library.
*   **Missing Symbols**: Adds `div64.o` to the build and implements a `raise` function (in `arch/arm/lib/raise.c`) which handles division-by-zero exceptions expected by `libgcc`.
*   **Bindgen Configuration**: Updates `rust/Makefile` to ensure the correct target (`arm-linux-gnueabi`) is passed to bindgen.

## Busybox
Busybox is built as a static binary to provide essential utilities.
*   It is configured to be placed in `/etc/busybox`.
*   The `tc` (Traffic Control) utility is disabled as it causes build failures in this specific configuration.

## Image Generation

The final SD card image is assembled using the `genimage` tool, controlled by the configuration file `gen_image.cfg`. This tool automates the creation of the partition table and filesystem images.

The generated `sdcard.img` consists of two main partitions:

1.  **Boot Partition (`boot.vfat`)**:
    *   Formatted as VFAT.
    *   Contains the Raspberry Pi firmware files (`bootcode.bin`, `start.elf`, `fixup.dat`).
    *   Includes the compiled kernel (`kernel.img`), device tree blobs (`.dtb`), and overlays.
    *   Holds the boot configuration files (`config.txt`, `cmdline.txt`).

2.  **Root Partition (`root.ext4`)**:
    *   Formatted as ext4.
    *   Contains the minimal root filesystem.
    *   Includes the statically compiled Busybox binary.
    *   Hosts the custom Rust kernel module (`rust_out_of_tree.ko`) and user data.
