BUILD_DIR?=../build
CROSS_COMPILE ?= 

BUSYBOX_FLAGS+=ARCH=x86_64 CROSS_COMPILE="${CROSS_COMPILE}" CC=$(MUSL-GCC) CFLAGS="-Ilinux-headers-4.19.88-2/x86/include"

# prevent execution of busybox_all on include
all:

busybox_all: ${BUILD_DIR}/busybox-1_36_0/_install

${BUILD_DIR}/1_36_0.tar.gz:
	mkdir -p $(dir $@)
	wget -O "$@" https://github.com/mirror/busybox/archive/refs/tags/1_36_0.tar.gz
	[ -f "$@" ] || (echo -E '(ERROR) wget failed, check your connexion' 1>&2 ; exit 1)

${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2.tar.xz:
	mkdir -p $(dir $@)
	wget -O $@ https://github.com/sabotage-linux/kernel-headers/releases/download/v4.19.88-2/linux-headers-4.19.88-2.tar.xz 
	[ -f "$@" ] || (echo -E '(ERROR) wget failed, check your connexion' 1>&2 ; exit 1)

${BUILD_DIR}/busybox-1_36_0: ${BUILD_DIR}/1_36_0.tar.gz
	tar -xf ${BUILD_DIR}/1_36_0.tar.gz -C ${dir $@}
	touch $@

${BUILD_DIR}/busybox-1_36_0/.config:
	make defconfig ${BUSYBOX_FLAGS} -C ${BUILD_DIR}/busybox-1_36_0 || (rm $@; exit 1)
	sed -i 's/CONFIG_TC=y/CONFIG_TC=n/' $@
	echo 'CONFIG_STATIC=y' >> ${BUILD_DIR}/busybox-1_36_0/.config

${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2: ${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2.tar.xz
	tar -xf $< -C ${dir $@}
	touch $@

${BUILD_DIR}/busybox-1_36_0/_install: ${BUILD_DIR}/busybox-1_36_0 ${BUILD_DIR}/busybox-1_36_0/.config ${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2
	make install ${BUSYBOX_FLAGS} -C ${BUILD_DIR}/busybox-1_36_0 -j$(shell nproc)

.PHONY: busybox_all all
