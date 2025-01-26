
# Archlinux

```sh
mkdir initramfs
cd initramfs
cat > init << EOF
#!/bin/sh

mount -t devtmpfs devtmpfs /dev
mount -t proc none /proc
mount -t sysfs none /sys

cat <<!

Welcome to Micro Linux!
Boot took $(cut -d' ' -f1 /proc/uptime) seconds

!
exec /bin/sh
EOF
mkdir -p initramfs/bin initramfs/sbin initramfs/etc initramfs/proc initramfs/sys initramfs/dev initramfs/usr/bin initramfs/usr/sbin

cp -a busybox/_install/* ./initramfs
chmod +x init

find . -print0 | cpio --null -ov --format=newc | gzip -9 > ../initramfs.cpio.gz
cd ..
```

