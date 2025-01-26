BUILD_DIR?=build

# prevent execution of initramfs_all on include
all:

initramfs_all: ${BUILD_DIR}/initramfs.cpio.gz

${BUILD_DIR}/initramfs: ${BUILD_DIR}/busybox-1_36_0/_install
	@# Create all ramfs directories
	mkdir -p $@/bin $@/sbin $@/etc $@/proc $@/sys $@/dev $@/usr/bin $@/usr/sbin
	cp -a $</* $@

# copy the init.sh into the final directory
${BUILD_DIR}/initramfs/init: mk/init.sh ${BUILD_DIR}/initramfs  # make sure iniramfs directory exists
	cp $< $@
	chmod +x $@

${BUILD_DIR}/initramfs.cpio.gz: ${BUILD_DIR}/initramfs ${BUILD_DIR}/initramfs/init
	(cd $< && find . -print0 | cpio --null -ov --format=newc | gzip -9) > $@

.phony: initramfs_all all
