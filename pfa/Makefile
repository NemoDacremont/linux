BUILD_DIR?=build
LINUX_PATH?=..
MK_PATH?=mk

MAC_ADDRESS=de:ad:be:ef:ca:fe
QEMUFLAGS+=-kernel ${LINUX_PATH}/arch/x86_64/boot/bzImage \
	   -initrd build/initramfs.cpio.gz \
	   -no-reboot \
	   -nographic \
	   -append "console=ttyS0 panic=-1 loglevel=15" \
	   -device pci-testdev \
	   -netdev user,id=mynet0 \
	   -device rtl8139,netdev=mynet0,mac=${MAC_ADDRESS}

tval_v0_QEMUFLAGS+=-kernel ${LINUX_PATH}/arch/x86_64/boot/bzImage \
	   -initrd build/initramfs.cpio.gz \
	   -no-reboot \
	   -nographic \
	   -append "console=ttyS0 panic=-1 loglevel=15 init='/init -c /sbin/poweroff -f'" \
	   -device pci-testdev \
	   -netdev user,id=mynet0 \
	   -device rtl8139,netdev=mynet0,mac=${MAC_ADDRESS}


UNAME_S := $(shell uname -s)

REQ_LIB := libxml2 libelf-dev libssl-dev

REQ_CMD_TARGETS=$(addprefix check_dep_,${REQ_CMD})

# On macos, some executable names are different
ifeq ($(UNAME_S),Darwin)
	REQ_CMD=x86_64-linux-musl-gcc wget xz gzip gcc make git flex bison cpio xz bzip2 bc qemu-system-x86_64
	DEP_CMD="Macos, needed cmd and libs : ${REQ_CMD} ${REQ_LIB}"
	# Prefix of gcc and ar on macos
	CROSS_COMPILE=x86_64-linux-musl-
	# Because there is already musl in the prefix, it's juste gcc
	MUSL-GCC=gcc
endif

# If on linux, find the right distro to define DEP_CMD
ifeq ($(UNAME_S),Linux)
	OS=$(shell grep '^NAME=' /etc/os-release | tr -d '"' | sed 's/^NAME=//')
	# Required for busybox
	MUSL-GCC=musl-gcc

	ifeq ("${OS}", "Ubuntu")
		DEP_CMD="apt install git make musl musl-tools wget gcc xz-utils bzip2 cpio flex bison libxml2 libelf-dev bc libssl-dev"
	else ifeq ("${OS}", "Pop!_OS")
		DEP_CMD="apt install git make musl musl-tools wget gcc xz-utils bzip2 cpio flex bison libxml2 libelf-dev bc libssl-dev"
	else ifeq ("${OS}", "Arch Linux")
		DEP_CMD="pacman -S musl wget gcc git make xz bzip2 cpio flex bison libxml2 libelf bc openssl"
	else
		DEP_CMD="Unknown OS, needed cmd and libs : ${REQ_CMD} ${REQ_LIB}"
	endif
endif


include mk/*.mk

all: build

# Check if deps are installed and download and compile busybox and initramfs
prebuild: check_deps busybox_all initramfs_all

build: rbuild

rbuild: prebuild  rlinux_all 

cbuild: prebuild clinux_all 

rstart: rbuild
	qemu-system-x86_64 ${QEMUFLAGS}

cstart: cbuild
	qemu-system-x86_64 ${QEMUFLAGS}

rtval_v0: rbuild
	(qemu-system-x86_64 ${tval_v0_QEMUFLAGS} | grep "tval_v0|${MAC_ADDRESS}" 2>/dev/null >/dev/null) \
		|| (echo "Couldn't find the tag 'tval_v0|${MAC_ADDRESS}' in kernel logs"; exit 1)

ctval_v0: cbuild
	(qemu-system-x86_64 ${tval_v0_QEMUFLAGS} | grep "tval_v0|${MAC_ADDRESS}" 2>/dev/null >/dev/null) \
		|| (echo "Couldn't find the tag 'tval_v0|${MAC_ADDRESS}' in kernel logs"; exit 1)

# Check if all dependencies are installed, recommand to use ${DEP_CMD} otherwise
check_deps: ${REQ_CMD_TARGETS}

# Test if % is installed, recommand to use ${DEP_CMD} otherwise
check_dep_%:
	@which $* 2>&1 > /dev/null || (echo -e "\n(ERROR) couldn\'t find $*, considere installing all dependencies with : \n${DEP_CMD}\n" 1>&2 ; exit 1)

# Test if all driver files are formatted
test_format: c_testformat

# Format all driver files
format: c_format

clean:
	${RM} -r build

.PHONY: all \
	build \
	clean \
	check_deps \
	test_format \
	format \
	${REQ_CMD_TARGETS} \
	ctval_v0 \
	rtval_v0
