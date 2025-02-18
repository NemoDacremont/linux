#
# Install a fixed version of LLVM
# Steps:
# 1. Install a tar.gz release from kernel.org
# 2. Extract the archive
# 3. Install bindgen, required for compiling rust 
#
# NB: it installs bindgen even if not required, because it will probably be
# required sooner or later, so the optimization wouldn't be that meaningful
#

# guard to prevent several includes
ifndef _LLVM_MK_
_LLVM_MK_:="defined"

# Env var using path after uncompressing tar archive
LLVM=llvm-19.1.7-rust-1.84.0-x86_64

LLVM_TAR=${LLVM}.tar.gz
LLVM_TAR_URL_PREFIX=https://mirrors.edge.kernel.org/pub/tools/llvm/rust/files
llvm_prefix=${PWD}/${BUILD_DIR}/${LLVM}
build_path=${llvm_prefix}/bin:${PATH}
libclang_path=${llvm_prefix}/lib/libclang.so

# prevent execution of llvm_all on include
all:

ifeq ($(UNAME_S),Darwin)
	llvm_all:
else ifeq ($(UNAME_S),Linux)
	llvm_all: llvm_install
endif


ifeq ($(UNAME_S),Darwin)
	llvm_install:
else ifeq ($(UNAME_S),Linux)
	llvm_install: llvm_download llvm_install_bindgen
endif


ifeq ($(UNAME_S),Darwin)
	llvm_download:
else ifeq ($(UNAME_S),Linux)
	llvm_download: ${BUILD_DIR}/${LLVM}
endif

# Begin of targets for llvm_download 

# Download the tar of llvm-rust
${BUILD_DIR}/${LLVM_TAR}:
	mkdir -p $(dir $@)
	wget -O "$@" ${LLVM_TAR_URL_PREFIX}/${LLVM_TAR}
	[ -f "$@" ] || (echo -E '(ERROR) wget failed, check your connexion' 1>&2 ; exit 1)

# Extract the tar of llvm-rust
${BUILD_DIR}/${LLVM}: ${BUILD_DIR}/${LLVM_TAR}
	tar -xf $< -C $(dir $@)
	touch $@

# End of targets for llvm_download 
#

llvm_install_bindgen: ${BUILD_DIR}/.bindgen.installed

# Begin of targets for llvm_install_bindgen 

# llvm-rust lock file
${BUILD_DIR}/.bindgen.installed: ${BUILD_DIR}/${LLVM}
	@# Install bindgen locally
	PATH=${build_path} LIBCLANG_PATH=${libclang_path} cargo install --locked --root ${llvm_prefix} --version $(shell ${LINUX_PATH}/scripts/min-tool-version.sh bindgen) bindgen-cli
	touch $@
# End of targets for llvm_install_bindgen 

.PHONY: all llvm_all llvm_install llvm_download llvm_install_bindgen

endif # ifndef _LLVM_MK_
