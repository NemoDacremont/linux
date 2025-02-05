BUILD_DIR?=build
LINUX_PATH?=..
QEMUFLAGS+=-kernel ${LINUX_PATH}/arch/x86_64/boot/bzImage -initrd build/initramfs.cpio.gz -nographic -append "console=ttyS0" -device pci-testdev
OS=$(shell grep '^NAME=' /etc/os-release | tr -d '"' | sed 's/^NAME=//')

REQ_CMD=musl-gcc wget xz gzip gcc make git flex bison cpio xz bzip2 bc qemu-system-x86_64
REQ_LIB=libxml2 libelf-dev libssl-dev

REQ_CMD_TARGETS=$(addprefix check_dep_,${REQ_CMD})

ifeq ("${OS}", "Ubuntu")
	DEP_CMD="apt install git make musl musl-tools wget gcc xz-utils bzip2 cpio flex bison libxml2 libelf-dev bc libssl-dev"
else ifeq ("${OS}", "Pop!_OS")
	DEP_CMD="apt install git make musl musl-tools wget gcc xz-utils bzip2 cpio flex bison libxml2 libelf-dev bc libssl-dev"
else ifeq ("${OS}", "Arch Linux")
	DEP_CMD="pacman -S musl wget gcc git make xz bzip2 cpio flex bison libxml2 libelf bc openssl"
else
	DEP_CMD="Unknown OS, needed cmd and libs : ${REQ_CMD} ${REQ_LIB}"
endif


include mk/*.mk

all: build

build: check_deps busybox_all linux_all initramfs_all

start: check_deps build
	qemu-system-x86_64 ${QEMUFLAGS}

check_deps: ${REQ_CMD_TARGETS}

# test if % is installed, recommand to use ${DEP_CMD} otherwise
check_dep_%:
	@which $* 2>&1 > /dev/null || (echo -e "\n(ERROR) couldn\'t find $*, considere installing all dependencies with : \n${DEP_CMD}\n" 1>&2 ; exit 1)

test_format: c_testformat

format: c_format

clean:
	${RM} -r build

.phony: all build clean check_deps ${REQ_CMD_TARGETS} test_format
