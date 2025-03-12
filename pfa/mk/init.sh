#!/bin/sh

mount -t devtmpfs devtmpfs /dev
mount -t proc none /proc
mount -t sysfs none /sys

cat <<!

Welcome to Micro Linux!
Boot took $(cut -d' ' -f1 /proc/uptime) seconds

!

if [ "$1" = "tval_v0" ]
then
	# Force init kill to stop the vm
	exit 0
elif [ "$1" = "tval_v1" ]
then
	# Print the machine ip adress using /sbin/ip
	echo "ctval_v1|`/sbin/ip addr show eth0 | grep -oE '([0-9a-z]{2}:){5}[0-9a-z]{2}' | head -n 1`";
	# Force init kill to stop the vm
	exit 0
fi


exec /bin/sh $@
