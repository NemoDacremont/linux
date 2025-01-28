# General Tips
- Use a recent python3 install
- be careful about menuconfig options you select


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

# MacOS

On macOS, the compiling procedure is a bit more delicate as we don't have access to the system wide linux headers.
The simplest way around that is using bee-headers `https://github.com/bee-headers/homebrew-bee-headers?tab=readme-ov-file#how-to-use-bee-headers-to-build-the-linux-kernel-in-macos` (credits to Tamir Duberstein for the tip).
