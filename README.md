# Raspberry Pi Linux GPIO Driver with Rust

This repository contains a custom Linux kernel build for the Raspberry Pi (specifically targeting ARMv6/Pi Zero W) with enabled Rust support, a custom Rust GPIO driver, and a minimal userspace based on Busybox.

## Quickstart

To build the complete SD card image, simply run the build script:

```bash
./build_image.sh
```

This command triggers a Docker-based build process. Once completed, the final artifacts, including the SD card image, will be available in the `./output` directory.

## Hardware

The driver assumes that some sort of beeper or speaker is connected to **GPIO 17**.
The PI will toggle this pin in the desired frequency to generate sounds.

## Usage

Upon boot, press enter and you will be dropped into a shell prompt.

1. Load the driver with: `insmod rust_out_of_tree.ko`
2. Use the driver, for example play a song: `cat userdata/pirates.txt > /dev/rust_out_of_tree`
3. You can also try some morse code: `echo -e "mHELLO WORLD" > /dev/rust_out_of_tree`

### Detailed usage

The driver is a miscdevice. It exposes a file `/dev/rust_out_of_tree` which takes input by writing to it.
You should write lines, where each line is a command.

There are two commands available:

1. `t<frequency> <duration>` - Plays a tone at the specified frequency (in hz) for the specified duration (in microseconds).
2. `m<text>` - Plays a morse code message.

### Creating instruction files from MIDI files

You can use the tool in the `generator` directory. It's a simple Python script that will take a MIDI file and print out the instructions.
Save them to a file and copy it to the `userdata` directory. This directory is automatically copied into the image at build time.

---

# Kernel Build with Rust Support

This section documents the process for building the Raspberry Pi Linux kernel with Rust support enabled, specifically targeting ARMv6 (Raspberry Pi Zero/1), as defined in the `Dockerfile`.

## Build Environment
The build is performed in an Ubuntu 24.04 container with the following key components:
*   **Rust Toolchain**: Version 1.84.1 is installed with the `arm-unknown-linux-gnueabi` target.
*   **LLVM & Clang**: Installed to support compiling the kernel with `LLVM=1`.
*   **Bindgen**: `bindgen-cli` (v0.72.1) is used to generate C bindings for Rust.

## Kernel Patches & ARMv6 Support
The kernel source (branch `rpi-6.18.y`) is patched to enable Rust support for ARMv6, which is not officially supported upstream. The patch `linux/0001-rust-for-armv6.patch` enables this.

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
