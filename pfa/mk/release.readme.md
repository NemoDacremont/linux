# Démonstration PFA-05

Pour lancer la démonstration en C :

```sh
make cstart
```

Pour lancer la démonstration en Rust :

```sh
make rstart
```

Dans les deux cas, à la fin des logs, les lignes devraient ressembler aux suivantes :

```
[    1.173112][    T1] 8139c 0000:00:04.0: PCI->APIC IRQ transform: INT A -> IRQ 11
[    1.187442][    T1][RTL8139c] MAC address: tval_v0|de:ad:be:ef:ca:fe
[    1.194096][    T1] serio: i8042 KBD port at 0x60,0x64 irq 1
[    1.194674][    T1] serio: i8042 AUX port at 0x60,0x64 irq 12
[    1.201059][    T1] Initializing XFRM netlink socket
[    1.201621][    T1] NET: Registered PF_INET6 protocol family
[    1.207520][    T6] input: AT Translated Set 2 keyboard as /devices/platform/i8042/serio0/input/input0
[    1.220887][    T1] Segment Routing with IPv6
[    1.221482][    T1] In-situ OAM (IOAM) with IPv6
[    1.223572][    T1] sit: IPv6, IPv4 and MPLS over IPv4 tunneling driver
[    1.227867][    T1] NET: Registered PF_PACKET protocol family
[    1.235232][    T1] sched_clock: Marking stable (1204771485, 27601369)->(1232592637, -219783)
[    1.447826][    T1] Freeing unused kernel image (initmem) memory: 616K
[    1.448276][    T1] Write protecting the kernel read-only data: 8192k
[    1.455671][    T1] Freeing unused kernel image (rodata/data gap) memory: 1420K
[    1.456118][    T1] Run /init as init process
[    1.456245][    T1]   with arguments:
[    1.456427][    T1]     /init
[    1.456537][    T1]   with environment:
[    1.456648][    T1]     HOME=/
[    1.456745][    T1]     TERM=linux

Welcome to Micro Linux!
Boot took 1.53 seconds
```

La partie importante étant la ligne suivante, affichant l'adresse MAC de la
carte (configurée dans le `Makefile`, à l'aide de `qemu`) comme définie pour la
v0

```
[    1.187442][    T1][RTL8139c] MAC address: tval_v0|de:ad:be:ef:ca:fe
```

Pour tester la présence de cette ligne, les *targets* suivantes peuvent être utilisées

Pour lancer la démonstration en C :

```sh
make ctval_v0
```

Pour lancer la démonstration en Rust :

```sh
make rtval_v0
```

