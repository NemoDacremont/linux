
# Archlinux

```sh
qemu-system-x86_64 -kernel linux/arch/x86_64/boot/bzImage  \
                   -initrd initramfs.cpio.gz -nographic    \
                   -append "console=ttyS0"
```
