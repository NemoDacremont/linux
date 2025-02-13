#
# Build a x86 static executable of busybox (using musl-gcc) to embed with
# initramfs
#
# Steps:
# 1. Download the release of busybox
# 2. Extract the release archive
# 3. Download patched linux headers for musl-gcc
# 4. Create busybox config
# 5. Build the busybox/_install directory creating links to applets implemented
#	by busybox
#

# Guard to prevent including several times busybox.mk
ifndef _BUSYBOX_MK_
_BUSYBOX_MK_ := "defined"

BUILD_DIR ?= ../build

# Defaults for linux
MUSL-GCC ?= musl-gcc
CROSS_COMPILE ?= 

# Path to _install directory
BUSYBOX__INSTALL_PATH := ${BUILD_DIR}/busybox-1_36_0/_install

# Flags passed to make when building busybox
BUSYBOX_FLAGS+=ARCH=x86_64 \
	       CROSS_COMPILE="${CROSS_COMPILE}" \
	       CC=$(MUSL-GCC) \
	       CFLAGS="-Ilinux-headers-4.19.88-2/x86/include" \
	       -j$(shell nproc)


# Prevent execution of busybox_all on include
all:


# Default busybox target to build
busybox_all: busybox_build


# Build the _install directory, creating links to all applets implemented by
# busybox
busybox_build: busybox_download_release busybox_download_headers busybox_config busybox_build_applets


# Download a fixed version of busybox
busybox_download_release: ${BUILD_DIR}/busybox-1_36_0

# Begin of targets for busybox_download_release
${BUILD_DIR}/busybox-1_36_0: ${BUILD_DIR}/1_36_0.tar.gz
	tar -xf ${BUILD_DIR}/1_36_0.tar.gz -C ${dir $@}
	touch $@

${BUILD_DIR}/1_36_0.tar.gz:
	mkdir -p $(dir $@)
	wget -O "$@" https://github.com/mirror/busybox/archive/refs/tags/1_36_0.tar.gz
	[ -f "$@" ] || (echo -E '(ERROR) wget failed, check your connexion' 1>&2 ; exit 1)
# End of targets for busybox_download_release


# Download a fixed version linux headers for musl-gcc (to compile statically)
busybox_download_headers: ${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2

# Begin of targets for busybox_download_headers
${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2: ${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2.tar.xz
	tar -xf $< -C ${dir $@}
	touch $@

${BUILD_DIR}/busybox-1_36_0/linux-headers-4.19.88-2.tar.xz:
	mkdir -p $(dir $@)
	wget -O $@ https://github.com/sabotage-linux/kernel-headers/releases/download/v4.19.88-2/linux-headers-4.19.88-2.tar.xz 
	[ -f "$@" ] || (echo -E '(ERROR) wget failed, check your connexion' 1>&2 ; exit 1)
# End of targets for busybox_download_headers


busybox_config: ${BUILD_DIR}/busybox-1_36_0/.config

# Begin of targets for busybox_config
${BUILD_DIR}/busybox-1_36_0/.config: ${MK_PATH}/bbconfig
	cp $< $@
# End of targets for busybox_config


busybox_build_applets: ${BUILD_DIR}/busybox-1_36_0/_install

# Begin of targets for busybox_build_applets
# make install is phony in busybox makefile, and links applets to busybox
# executable, so we compile it only if busybox executable is more recent
${BUILD_DIR}/busybox-1_36_0/_install: ${BUILD_DIR}/busybox-1_36_0/busybox
	@# make install creates the _install directory used to create the
	@# initramfs 
	make install -C ${BUILD_DIR}/busybox-1_36_0 ${BUSYBOX_FLAGS}

${BUILD_DIR}/busybox-1_36_0/busybox:
	@# compile the executable busybox
	make -C ${BUILD_DIR}/busybox-1_36_0 ${BUSYBOX_FLAGS}
# End of targets for busybox_build_applets


busybox_clean:
	make clean -C ${BUILD_DIR}/busybox-1_36_0


.PHONY: all \
	busybox_all \
	busybox_build \
	busybox_download_release \
	busybox_download_headers \
	busybox_config \
	busybox_clean
endif  # ifndef _BUSYBOX_MK_
