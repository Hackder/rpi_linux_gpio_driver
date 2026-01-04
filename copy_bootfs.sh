#!/usr/bin/env bash

mkdir -p bootfs/overlays

cp -arv ./firmware/* ./bootfs/
cp -arv ./linux/arch/arm/boot/zImage ./bootfs/kernel.img
cp -arv ./linux/arch/arm/boot/dts/overlays/*.dtb{,o} ./bootfs/overlays/
cp -arv ./linux/arch/arm/boot/dts/broadcom/bcm2708-rpi-zero-w.dtb ./bootfs

echo "dtoverlay=miniuart-bt" > ./bootfs/config.txt
echo "8250.nr_uarts=1 earlyprintk console=ttyAMA0,115200 root=/dev/mmcblk0p2 rootwait init=/sbin/init" > ./bootfs/cmdline.txt
