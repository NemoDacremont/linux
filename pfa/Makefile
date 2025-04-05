BUILD_DIR?=build
LINUX_PATH?=..
MK_PATH?=mk

MAC_ADDRESS=de:ad:be:ef:ca:fe
QEMUFLAGS+=-kernel ${LINUX_PATH}/arch/x86_64/boot/bzImage \
	   -initrd ${BUILD_DIR}/initramfs.cpio.gz \
	   -no-reboot \
	   -nographic \
	   -append "console=ttyS0 panic=-1 loglevel=15" \
	   -device pci-testdev \
	   -netdev tap,id=mynet0 \
	   -device rtl8139,netdev=mynet0,mac=${MAC_ADDRESS}

tval_v0_QEMUFLAGS+=-kernel ${LINUX_PATH}/arch/x86_64/boot/bzImage \
	   -initrd ${BUILD_DIR}/initramfs.cpio.gz \
	   -no-reboot \
	   -nographic \
	   -append "console=ttyS0 panic=-1 loglevel=15 init='/init tval_v0'" \
	   -device pci-testdev \
	   -netdev tap,id=mynet0 \
	   -device rtl8139,netdev=mynet0,mac=${MAC_ADDRESS}

tval_v1_QEMUFLAGS+=-kernel ${LINUX_PATH}/arch/x86_64/boot/bzImage \
	   -initrd ${BUILD_DIR}/initramfs.cpio.gz \
	   -no-reboot \
	   -nographic \
	   -append "console=ttyS0 panic=-1 loglevel=15 init='/init tval_v1" \
	   -device pci-testdev \
	   -netdev tap,id=mynet0 \
	   -device rtl8139,netdev=mynet0,mac=${MAC_ADDRESS}

UNAME_S := $(shell uname -s)

REQ_LIB := libxml2 libelf-dev libssl-dev

# On macos, some executable names are different
ifeq ($(UNAME_S),Darwin)
	REQ_CMD=x86_64-linux-musl-gcc wget xz gzip gcc make git flex bison cpio bzip2 bc qemu-system-x86_64
	DEP_CMD="Macos, needed cmd and libs : ${REQ_CMD} ${REQ_LIB}"
	# Prefix of gcc and ar on macos
	CROSS_COMPILE=x86_64-linux-musl-
	# Because there is already musl in the prefix, it's juste gcc
	MUSL-GCC=x86_64-linux-musl-gcc
endif

# If on linux, find the right distro to define DEP_CMD
ifeq ($(UNAME_S),Linux)
	OS=$(shell grep '^NAME=' /etc/os-release | tr -d '"' | sed 's/^NAME=//')
	REQ_CMD=musl-gcc wget xz gzip gcc make git flex bison cpio bzip2 bc qemu-system-x86_64
	# Required for busybox
	MUSL-GCC=musl-gcc

	ifeq ("${OS}", "Ubuntu")
		DEP_CMD="apt install git make musl musl-tools wget gcc xz-utils bzip2 cpio flex bison libxml2 libelf-dev bc libssl-dev qemu-system"
	else ifeq ("${OS}", "Pop!_OS")
		DEP_CMD="apt install git make musl musl-tools wget gcc xz-utils bzip2 cpio flex bison libxml2 libelf-dev bc libssl-dev qemu-system"
	else ifeq ("${OS}", "Arch Linux")
		DEP_CMD="pacman -S musl wget gcc git make xz bzip2 cpio flex bison libxml2 libelf bc openssl qemu-system-x86"
	else
		DEP_CMD="Unknown OS, needed cmd and libs : ${REQ_CMD} ${REQ_LIB}"
	endif
endif

REQ_CMD_TARGETS=$(addprefix check_dep_,${REQ_CMD})


include mk/*.mk

all: build

# Check if deps are installed and download and compile busybox and initramfs
prebuild: check_deps busybox_all initramfs_all

# Default build : build rust
build: rbuild

# Build rust driver
rbuild: prebuild  rlinux_all

# Build C driver
cbuild: prebuild clinux_all

# Build C driver
cpbuild: prebuild cplinux_all

# Starts qemu with the kernel built with the rust driver
rstart: rbuild
	sudo qemu-system-x86_64 ${QEMUFLAGS}

# Starts qemu with the kernel built with the C driver
cstart: cbuild
	sudo qemu-system-x86_64 ${QEMUFLAGS}

# Starts qemu with the kernel built with the official kernel CP driver
cpstart: cpbuild
	sudo qemu-system-x86_64 ${QEMUFLAGS}

# Validation test for v0 of the rust driver
rtval_v0: rbuild
	(sudo qemu-system-x86_64 ${tval_v0_QEMUFLAGS} | grep "tval_v0|${MAC_ADDRESS}" 2>/dev/null >/dev/null) \
		|| (echo "Couldn't find the tag 'tval_v0|${MAC_ADDRESS}' in kernel logs"; exit 1) \
		&& echo -e "\n OK - tvalv0\n"

# Validation test for v0 of the C driver
ctval_v0: cbuild
	(sudo qemu-system-x86_64 ${tval_v0_QEMUFLAGS} | grep "tval_v0|${MAC_ADDRESS}" 2>/dev/null >/dev/null) \
		|| (echo "Couldn't find the tag 'tval_v0|${MAC_ADDRESS}' in kernel logs"; exit 1) \
		&& echo -e "\n OK - tvalv0\n"

# Validation test for v1 of the rust driver
rtval_v1: rbuild
	(sudo qemu-system-x86_64 ${tval_v1_QEMUFLAGS} | grep "tval_v1|${MAC_ADDRESS}" 2>/dev/null >/dev/null) \
		|| (echo -e "\n KO - tvalv1 Couldn't find the tag 'tval_v1|${MAC_ADDRESS}' in tval_v1 logs\n"; exit 1) \
		&& echo -e "\n OK - tvalv1\n"

# Validation test for v1 of the C driver
ctval_v1: cbuild
	(sudo qemu-system-x86_64 ${tval_v1_QEMUFLAGS} | grep "tval_v1|${MAC_ADDRESS}" 2>/dev/null >/dev/null) \
		|| (echo -e "\n KO - tvalv1 Couldn't find the tag 'tval_v1|${MAC_ADDRESS}' in tval_v1 logs\n"; exit 1) \
		&& echo -e "\n OK - tvalv1\n"

# Validation test for v2 of the C driver
ctval_v2: cbuild
	./tval_v2

# Validation test for v2 of the rust driver
rtval_v2: rbuild
	./tval_v2

# Used to test if the test works, uses the kernel's official driver for rtl8139
cptval_v2: cpbuild
	./tval_v2


# Validation test for v2 of the C driver
ctval_v2send: cbuild
	./tval_v2send

# Validation test for v2 of the rust driver
rtval_v2send: rbuild
	./tval_v2send

# Used to test if the test works, uses the kernel's official driver for rtl8139
cptval_v2send: cpbuild
	./tval_v2send

# Validation test for v2 of the C driver
ctval_v2recv: cbuild
	./tval_v2recv

# Validation test for v2 of the rust driver
rtval_v2recv: rbuild
	./tval_v2recv

# Used to test if the test works, uses the kernel's official driver for rtl8139
cptval_v2recv: cpbuild
	./tval_v2recv

# Test if % is installed, recommand to use ${DEP_CMD} otherwise
${REQ_CMD_TARGETS}: check_dep_%:
	@which $* 2>&1 > /dev/null || (echo -e "\n(ERROR) couldn\'t find $*, considere installing all dependencies with : \n${DEP_CMD}\n" 1>&2 ; exit 1)

# Check if all dependencies are installed, recommand to use ${DEP_CMD} otherwise
check_deps: ${REQ_CMD_TARGETS}

# Test if all driver files are formatted
test_format: c_testformat rust_testformat

# Format all driver files
format: c_format rust_format

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
