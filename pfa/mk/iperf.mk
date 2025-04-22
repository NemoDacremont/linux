#
# Build a x86_64 static executable of iperf3 (using musl-gcc) to embed with
# initramfs
#
# Steps:
# 1. Download the release of iperf
# 2. Extract the release archive
# 3. Download patched linux headers for musl-gcc
# 4. Create iperf config
# 5. Build the iperf/_install directory creating links to applets implemented
#	by iperf
#

# Guard to prevent including several times iperf.mk
ifndef _IPERF_MK_
_IPERF_MK_ := "defined"

BUILD_DIR ?= ../build

# Defaults for linux
MUSL-GCC ?= musl-gcc
CROSS_COMPILE ?= 

# Path to _install directory
IPERF__INSTALL_PATH := $(shell pwd)/${BUILD_DIR}

# Flags passed to make when building iperf
IPERF_FLAGS+=ARCH=x86_64 \
	       CROSS_COMPILE="${CROSS_COMPILE}" \
	       CC=$(MUSL-GCC) \
	       CFLAGS="-Ilinux-headers-4.19.88-2/x86/include" \
	       -j$(shell nproc)


# Prevent execution of iperf_all on include
all:


# Default iperf target to build
iperf_all: iperf_build


# Build the _install directory, creating links to all applets implemented by
# iperf
iperf_build: iperf_download_release iperf_config iperf_install


# Download a fixed version of iperf
iperf_download_release: ${BUILD_DIR}/iperf-3.18

# Begin of targets for iperf_download_release
${BUILD_DIR}/iperf-3.18: ${BUILD_DIR}/iperf-3.18.tar.gz
	tar -xf $^ -C $(dir $@)
	touch $@

${BUILD_DIR}/iperf-3.18.tar.gz:
	mkdir -p $(dir $@)
	wget -O "$@" https://github.com/esnet/iperf/releases/download/3.18/iperf-3.18.tar.gz
	[ -f "$@" ] || (echo -E '(ERROR) wget failed, check your connexion' 1>&2 ; exit 1)
# End of targets for iperf_download_release

iperf_config: ${BUILD_DIR}/iperf-3.18/Makefile

# Begin of targets for iperf_config
${BUILD_DIR}/iperf-3.18/Makefile:
	(cd $(dir $@); ./configure "LDFLAGS=--static" "CC=musl-gcc" --prefix=$(IPERF__INSTALL_PATH) --disable-shared --without-sctp)
# End of targets for iperf_config


iperf_install: ${BUILD_DIR}/iperf3

# Begin of targets for iperf_build_applets
# make install is phony in iperf makefile, and links applets to iperf
# executable, so we compile it only if iperf executable is more recent
${BUILD_DIR}/iperf3: ${BUILD_DIR}/iperf-3.18/src/iperf3
	@# make install creates the _install directory used to create the
	@# initramfs 
	make install -C ${BUILD_DIR}/iperf-3.18 -j$(shell nproc)
	install ${BUILD_DIR}/bin/iperf3 $@

${BUILD_DIR}/iperf-3.18/src/iperf3:
	@# compile the executable iperf
	make -C ${BUILD_DIR}/iperf-3.18
# End of targets for iperf_build_applets


iperf_clean:
	make clean -C ${BUILD_DIR}/iperf-3.18


.PHONY: all \
	iperf_all \
	iperf_build \
	iperf_download_release \
	iperf_config \
	iperf_clean
endif  # ifndef _IPERF_MK_
