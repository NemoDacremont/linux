#
## Build process
#
# 1. Instal LLVM
# 1. Check rust available (to make sure everything goes as expected)
# 2. Copy rconfig linux config (a minimal config for qemu using our rust driver)
# 3. Build linux (target bzImage for x86)
#

#
BUILD_DIR?=../build
# Path to linux submodule
LINUX_PATH?=../..
MK_PATH?=.

# Requires to install llvm, using a fixed version from llvm.mk
include ${MK_PATH}/llvm.mk

ifeq ($(UNAME_S),Darwin)
	path_flags:=""
else ifeq ($(UNAME_S),Linux)
	path_flags:=PATH=${build_path} \
		   LIBCLANG_PATH=${libclang_path}
endif

# Build using LLVM is required for rust
RLINUX_FLAGS+=LLVM=1 \
	      ARCH=x86 \
	      ${path_flags} \
	     -j$(shell nproc)

# Prevent execution of rlinux_all on include
all:

# Default rlinux target
rlinux_all: rlinux_build

# Build x86 Linux Kernel
# This is a PHONY target because the need to compile files is delegated to 
# Linux Kernel's Makefile
#
# NB: the hack of using yes "" is to use the default config that may be offered
# by Linux Kernel's Makefile
rlinux_build: llvm_install rlinux_rustavailable rlinux_config
	yes "" | make bzImage -C ${LINUX_PATH} ${RLINUX_FLAGS}

# To test if rust is available (and that everything is working)
rlinux_rustavailable: llvm_install
	make rustavailable -C ${LINUX_PATH} ${RLINUX_FLAGS}

# Force the use of the kernel config "rconfig"
rlinux_config:
	cp ${MK_PATH}/rconfig ${LINUX_PATH}/.config

.PHONY: all rlinux_all rlinux_rustavailable rlinux_config rlinux_build
