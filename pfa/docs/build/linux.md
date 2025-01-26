
# Archlinux

```sh
cd linux
wget https://mirrors.edge.kernel.org/pub/tools/llvm/rust/files/llvm-19.1.4-rust-1.83.0-x86_64.tar.gz
tar -axf llvm-19.1.4-rust-1.83.0-x86_64.tar.gz
llvm_prefix=./llvm-19.1.4-rust-1.83.0-x86_64
export PATH=$llvm_prefix/bin:$PATH
export LIBCLANG_PATH=$llvm_prefix/lib/libclang.so
cargo install --locked --root $llvm_prefix --version $(scripts/min-tool-version.sh bindgen) bindgen-cli
make defconfig
make LLVM=1 rustavailable
make menuconfig # activer Rust et PCI Rust sample ou que sais-je
make LLVM=1 -j$(nproc)
cd ..
```
