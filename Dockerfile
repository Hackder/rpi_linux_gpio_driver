FROM ubuntu:24.04 AS build

# Standard kernel deps
RUN apt-get update -y && apt-get upgrade -y && \
    apt-get install -y build-essential bc bison flex libssl-dev wget curl git gcc-arm-linux-gnueabi libncurses5-dev file cpio unzip rsync

# Genimage deps
RUN apt-get install -y dosfstools mtools genext2fs genimage

# LLVM and Clang
RUN apt install -y clang lld llvm

# Set bash as default shell
SHELL ["/bin/bash", "-c"]

# Rust toolchain and bindgen
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain 1.84.1

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add arm-unknown-linux-gnueabi
RUN rustup component add rust-src
RUN cargo install bindgen-cli@0.72.1

WORKDIR /home

# Download the Raspberry Pi firmware and kernel
RUN mkdir -p firmware && \
    cd firmware && \
    wget https://raw.githubusercontent.com/raspberrypi/firmware/f1ea7092589bc9627c23916132baa7841932b707/boot/{bootcode.bin,fixup.dat,start.elf} && \
    cd ..

RUN git clone -b rpi-6.18.y --depth 1 https://github.com/raspberrypi/linux.git

# Patch the kernel to support rust for armv6 (not officially supported as of 6.12)
COPY ./linux/0001-rust-for-armv6.patch .
RUN cd linux && patch -p1 < ../0001-rust-for-armv6.patch


# Cross compile env setup
ENV ARCH="arm"
ENV CROSS_COMPILE="arm-linux-gnueabi-"

# -----------------------------------
# Build the kernel
# -----------------------------------
WORKDIR /home/linux

# Configure the kernel for raspberry pi and Rust
RUN make bcmrpi_defconfig
COPY ./linux/.config .config

# Build the kernel
RUN make LLVM=1 LLVM_IAS=0 -j$(nproc) zImage modules dtbs

# Extract bootfs files
WORKDIR /home
COPY ./copy_bootfs.sh .
RUN chmod +x copy_bootfs.sh && ./copy_bootfs.sh

# -----------------------------------
# Download and build busybox
# -----------------------------------
RUN wget https://busybox.net/downloads/busybox-1.36.1.tar.bz2
RUN tar xf busybox-1.36.1.tar.bz2
WORKDIR /home/busybox-1.36.1/

# Make sure we build a static binary located in /bin/busybox
# We also disable the `tc` utility as it breaks the build for some reason
RUN make defconfig
COPY ./busybox/.config .config
RUN make oldconfig
RUN make -j$(nproc)
RUN make install

# Extract rootfs base
WORKDIR /home
COPY ./copy_rootfs_base.sh .
RUN chmod +x copy_rootfs_base.sh && ./copy_rootfs_base.sh

# -----------------------------------
# Build the rust driver
# -----------------------------------

COPY ./rust-driver ./rust-driver
WORKDIR /home/rust-driver
RUN make -j$(nproc)

# Copy the driver to the rootfs
WORKDIR /home
RUN cp ./rust-driver/rust_out_of_tree.ko ./rootfs/rust_out_of_tree.ko

# -----------------------------------
# Overlay
# -----------------------------------
COPY ./userdata /home/rootfs/userdata

# -----------------------------------
# Generate the final image
# -----------------------------------
COPY ./gen_image.cfg .
RUN mkdir -p output
RUN genimage --config gen_image.cfg --rootpath /home/rootfs --inputpath /home/bootfs --outputpath ./output --tmppath tmp

# -----------------------------------
# Export the image
# -----------------------------------
FROM scratch AS export
COPY --from=build /home/output/ /
