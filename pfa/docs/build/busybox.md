
# Building busybox statically

# Compile busybox on osx (arm64)

```sh
git clone https://github.com/mirror/busybox.git
cd busybox
git clone https://github.com/sabotage-linux/kernel-headers.git
make ARCH=arm CROSS_COMPILE=aarch64-linux-musl- CFLAGS="-Ikernel-headers/arm64/include" install
```

# Compile busybox (x86\_64)

```sh
git clone https://github.com/mirror/busybox.git
cd busybox
git clone https://github.com/sabotage-linux/kernel-headers.git
make CC=musl-gcc CFLAGS="-Ikernel-headers/x86_64/include" install 
```
