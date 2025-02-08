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

clinux_all: clinux_config
	yes "" | make -C ${LINUX_PATH} -j$(shell nproc)

clinux_config:
	cp mk/cconfig ${LINUX_PATH}/.config

.phony: all linux_all clinux_config
