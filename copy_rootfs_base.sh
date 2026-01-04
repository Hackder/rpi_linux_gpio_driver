#!/usr/bin/env sh

rsync -av ./busybox-1.36.1/_install/ ./rootfs/

mkdir ./rootfs/dev
mkdir ./rootfs/sys
mkdir ./rootfs/proc
