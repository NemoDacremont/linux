#
## Create an initramfs.cpio.gz file to launch a virtual machine
#
# Steps:
# 1. Use busybox.mk to build a static x86 busybox executable
# 2. Create an initramfs folder including usuals linux root directories
# 3. Download patched linux headers for musl-gcc
#

BUILD_DIR?=../build
SRC_DIR?=../src
MK_PATH?=.

include ${MK_PATH}/busybox.mk


# Prevent execution of initramfs_all on include
all:


initramfs_all: initramfs_build


initramfs_build: busybox_build initramfs_create_root initramfs_create_cpio


initramfs_create_root: busybox_build ${BUILD_DIR}/send_udp ${BUILD_DIR}/initramfs ${BUILD_DIR}/initramfs/init

# Begin of initramfs_root
# **Make sure busybox_build as been called before**, otherwise it will not
# create initramfs correctly
${BUILD_DIR}/initramfs: ${BUILD_DIR}/initramfs/send_udp
	@# Create usual Linux directories
	mkdir -p $@/bin $@/sbin $@/etc $@/proc $@/sys $@/dev $@/usr/bin $@/usr/sbin
	@# Copy busybox applets to ramfs
	cp -a ${BUSYBOX__INSTALL_PATH}/* $@
	touch $@

${BUILD_DIR}/send_udp: ${SRC_DIR}/send_udp.c
	$(CC) -o $@ $^ -static

${BUILD_DIR}/initramfs/send_udp: ${BUILD_DIR}/send_udp
	mkdir -p $(dir $@)
	cp -a $^ $@

# copy the init.sh into the final directory
${BUILD_DIR}/initramfs/init: ${BUILD_DIR}/initramfs ${MK_PATH}/init.sh  # make sure iniramfs directory exists
	cp ${MK_PATH}/init.sh $@
	chmod +x $@
# End of initramfs_root


initramfs_create_cpio: ${BUILD_DIR}/initramfs.cpio.gz

# Begin of targets for initramfs_create_cpio
${BUILD_DIR}/initramfs.cpio.gz: ${BUILD_DIR}/initramfs ${BUILD_DIR}/initramfs/init
	(cd $< && find . -print0 | cpio --null -ov --format=newc | gzip -9) > $@
# End of targets for initramfs_create_cpio


.PHONY: all \
	initramfs_all \
	initramfs_build \
	initramfs_create_root \
	initramfs_create_cpio
