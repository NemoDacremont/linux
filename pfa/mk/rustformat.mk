LINUX_PATH?=../..

# flags passed to `make rustfmt[check]` linux target 
RUST_FMT_FLAGS+=CLIPPY=1

all: rust_testformat

# format all rust code
rust_format:
	make -C ${LINUX_PATH} ${RUST_FMT_FLAGS} rustfmt

# check formatting of all rust code, exit with non-zero status if fails
rust_testformat:
	make -C ${LINUX_PATH} ${RUST_FMT_FLAGS} rustfmtcheck

.phony: all rust_format rust_testformat
