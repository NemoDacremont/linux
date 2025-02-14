#
## Targets to abstract rust for linux's formatting rules
#

LINUX_PATH?=../..

# flags passed to `make rustfmt[check]` linux target 
RUST_FMT_FLAGS+=CLIPPY=1

# Prevent default target call on include
all:

# format all rust code
rust_format:
	make -C ${LINUX_PATH} ${RUST_FMT_FLAGS} rustfmt

# check formatting of all rust code, exit with non-zero status if fails
rust_testformat:
	make -C ${LINUX_PATH} ${RUST_FMT_FLAGS} rustfmtcheck

.PHONY: all rust_format rust_testformat
