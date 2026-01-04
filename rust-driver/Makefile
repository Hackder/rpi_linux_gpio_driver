# SPDX-License-Identifier: GPL-2.0

KDIR ?= /home/linux
ARCH ?= arm
CROSS_COMPILE ?= arm-linux-gnueabi-
LLVM ?= 1
LLVM_IAS ?= 0
KRUSTFLAGS ?= "--target=arm-unknown-linux-gnueabi"


default:
	$(MAKE) -C $(KDIR) ARCH=$(ARCH) CROSS_COMPILE=$(CROSS_COMPILE) M=$$PWD LLVM=$(LLVM) LLVM_IAS=$(LLVM_IAS) KRUSTFLAGS=$(KRUSTFLAGS) modules
clean:
	$(MAKE) -C $(KDIR) ARCH=$(ARCH) CROSS_COMPILE=$(CROSS_COMPILE) M=$$PWD clean
