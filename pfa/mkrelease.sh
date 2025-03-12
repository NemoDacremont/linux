#!/bin/sh

print_usage () {
  echo "Usage: $0 <version>" >&2
}

if [ "$#" -lt "1" ]
then
  print_usage
  exit 1
fi

release_dir="release_$1"
build_dir=build
mk_path=mk
linux_path=..
BZImage_path="${linux_path}/arch/x86_64/boot/bzImage"
initramfs_path="${build_dir}/initramfs.cpio.gz"

mkdir -p "${release_dir}/src"

make initramfs_all
cp "${initramfs_path}" "${release_dir}/initramfs.cpio.gz"

cp "${linux_path}/drivers/net/ethernet/realtek/8139c.c" "${release_dir}/src/8139c.c"
cp "${linux_path}/drivers/net/ethernet/realtek/8139rs.rs" "${release_dir}/src/8139rs.rs"
cp "${mk_path}/release.makefile" "${release_dir}/Makefile"
cp "${mk_path}/release.readme.md" "${release_dir}/README.md"

make cbuild
cp -m 0644 "${BZImage_path}" "${release_dir}/cbzImage"
make cbuild
cp -m 0644 "${BZImage_path}" "${release_dir}/rbzImage"

tar -cvz -f "${release_dir}.tar.gz" "${release_dir}"
