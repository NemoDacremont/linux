BUILD_DIR?=build
# Path to linux submodule
LINUX_PATH?=../

# Env var using path after uncompressing tar archive
LLVM=llvm-19.1.7-rust-1.84.0-x86_64
LLVM_TAR=${LLVM}.tar.gz
LLVM_TAR_URL_PREFIX=https://mirrors.edge.kernel.org/pub/tools/llvm/rust/files
llvm_prefix=${PWD}/${BUILD_DIR}/${LLVM}
build_path=${llvm_prefix}/bin:${PATH}
libclang_path=${llvm_prefix}/lib/libclang.so

# prevent execution of initramfs_all on include
all:

linux_all: linux_rustavailable ${LINUX_PATH}/vmlinux

linux_rustavailable: ${BUILD_DIR}/.bindgen.installed
	PATH=${build_path} LIBCLANG_PATH=${libclang_path} make -C ${LINUX_PATH} LLVM=1 rustavailable

${BUILD_DIR}/${LLVM_TAR}:
	mkdir -p $(dir $@)
	wget -O "$@" ${LLVM_TAR_URL_PREFIX}/${LLVM_TAR}
	[ -f "$@" ] || (echo -E '(ERROR) wget failed, check your connexion' 1>&2 ; exit 1)

${BUILD_DIR}/${LLVM}: ${BUILD_DIR}/${LLVM_TAR}
	tar -xf $< -C $(dir $@)
	touch $@

${LINUX_PATH}/.config:
	PATH=${build_path} LIBCLANG_PATH=${libclang_path} make -C ${LINUX_PATH} LLVM=1 defconfig
	echo 'CONFIG_RUST=y' >> $@
	echo 'CONFIG_SAMPLES=y' >> $@
	echo 'CONFIG_SAMPLES_RUST=y' >> $@
	echo 'CONFIG_SAMPLE_RUST_DRIVER_PCI=y' >> $@

	echo 'CONFIG_RUST_OVERFLOW_CHECKS=y' >> $@

${BUILD_DIR}/.bindgen.installed: ${BUILD_DIR}/${LLVM_TAR}
	PATH=${build_path} LIBCLANG_PATH=${libclang_path} cargo install --locked --root ${llvm_prefix} --version $(shell ${LINUX_PATH}/scripts/min-tool-version.sh bindgen) bindgen-cli
	touch $@

${LINUX_PATH}/vmlinux: ${BUILD_DIR}/.bindgen.installed ${LINUX_PATH}/.config
	yes "" | PATH=${build_path} LIBCLANG_PATH=${libclang_path} make -C ${LINUX_PATH} LLVM=1 -j$(shell nproc)

.phony: all linux_all
