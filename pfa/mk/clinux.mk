#
## Build process
#
# 1. Instal LLVM
# 2. Copy cconfig linux config (a minimal config for qemu using our C driver)
# 3. Build linux (target bzImage for x86)
#


#
BUILD_DIR?=../build
# Path to linux submodule
LINUX_PATH?=../..
MK_PATH?=.

# Use llvm for compatibility with macos and reproductibility, using a fixed
# version from llvm.mk
include ${MK_PATH}/llvm.mk

# Build using LLVM for compatibility with macos and reproductibility
CLINUX_FLAGS+=LLVM=1 \
	     PATH=${build_path} \
	     LIBCLANG_PATH=${libclang_path} \
	     -j$(shell nproc)

# Prevent execution of clinux_all on include
all:

# Default clinux target
clinux_all: clinux_build

# Build x86 Linux Kernel
# This is a PHONY target because the need to compile files is delegated to 
# Linux Kernel's Makefile
#
# NB: the hack of using yes "" is to use the default config that may be offered
# by Linux Kernel's Makefile
clinux_build: llvm_install clinux_config
	yes "" | make bzImage -C ${LINUX_PATH} ${CLINUX_FLAGS}

# Force the use of the Kernel config "cconfig"
clinux_config:
	cp ${MK_PATH}/cconfig ${LINUX_PATH}/.config

.PHONY: all linux_all clinux_build clinux_config 
