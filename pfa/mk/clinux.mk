#
## Build process
#
# 1.check rust available
#	1. Download llvm-rust if the lock file .bindgen.installed isn't here
# 2. Create linux make config
#	1. Execute defconfig for default
#	2. Add required flags to compile with rust and samples
# 3. Build linux (target vmlinux)
#
#


#
BUILD_DIR?=build
# Path to linux submodule
LINUX_PATH?=..

all: clinux_all

clinux_all: 
	yes "" | make -C ${LINUX_PATH} -j$(shell nproc)

clinux_config:
	@# make defconfig
	PATH=${build_path} LIBCLANG_PATH=${libclang_path} make -C ${LINUX_PATH} LLVM=1 defconfig
	@# Additional configuration flag
	sed -i 's/.*8139.*///' ${LINUX_PATH}/.config
	echo 'CONFIG_8139C=y' >> ${LINUX_PATH}/.config

.phony: all linux_all clinux_config
