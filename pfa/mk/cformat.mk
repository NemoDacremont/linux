#
## Targets to format and check formatting of the project's drivers files
#

LINUX_PATH?=../..
C_FILES+=${LINUX_PATH}/drivers/net/ethernet/realtek/8139c.c

# flags for format and test
CLANG_FLAGS=
# flags only for format (ex: -i to format in place)
CLANG_FORMAT_FLAGS=-i
# flags to test if files are formatted
CLANG_TEST_FLAGS=--dry-run -Werror

# Prevent default target call on include
all:

c_format:
	clang-format ${CLANG_FLAGS} ${CLANG_FORMAT_FLAGS} ${C_FILES}

c_testformat:
	clang-format ${CLANG_FLAGS} ${CLANG_TEST_FLAGS} ${C_FILES} 2>/dev/null

.PHONY: all c_format c_testformat
