#!/bin/sh

mount -t devtmpfs devtmpfs /dev
mount -t proc none /proc
mount -t sysfs none /sys

cat <<!

Welcome to Micro Linux!
Boot took $(cut -d' ' -f1 /proc/uptime) seconds

!

ip link set eth0 up  # Enable communication with host
ip a add dev eth0 192.168.252.1/24  # Set an ip a

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

elif [ "$1" = "tval_v2'" ]
then
    # start listening before sending that we are ready to the test
    nc -lnvp 9998 &
    # Test sending, and send start signal to tester
    sleep 0.2  # to be 100% sure nc listener started
    echo "rtval_v2|rdy"
    nc 192.168.252.2 9999 << EOF
tval_v2|send
EOF

	# Force init kill to stop the vm, wait for nc 9998 writing msg to stdout
    sleep 0.5
	exit 1

elif [ "$1" = "tval_v2send'" ]
then
    /send_udp
    ls /send_udp
    sleep 0.5
	exit 1

elif [ "$1" = "tval_v2recv'" ]
then
    timeout 2 nc -lnvu -p 9999
	exit 1

elif [ "$1" = "benchmark'" ]
then
    /iperf3 -s -1
	exit 1
fi

exec /bin/sh $@
