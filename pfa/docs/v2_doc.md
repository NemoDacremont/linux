# Base

## From current kernel driver to our

 8439.c | 8139.cp 
------- | -------
 `rtl8139c_probe` | `cp_init_one`

### Current flow

#### Init (probe)

1. Alloc device data, including private `alloc_etherdev`
2. Set net device parent device (PCI) `SET_NETDEV_DEV`
3. Get private internal data `netdev_priv`
4. `pci_enable_device`, `pci_set_mwi`, `pci_request_regions`, `pci_resource_start`
5. Configure DMA attributes (?)
6. Define hw command (RE,TE) and netdev features
7. Map hw registers `ioremap` to private data  
> map bus memory into CPU space
8. stop hw
9. set MAC
10. set netdev ops
11. Setup NAPI `netif_napi_add_weight`
12. `register_netdev`, `pci_set_drvdata`

#### Open (net device op)
> This function is called when a network device transitions to the up state.

1. `alloc_rings`
2. enable NAPI `napi_enable`
3. enable interruptions `request_irq`
4. `init_hw` -> `reset_hw`, `start_hw` -> `set_cmd`
5. (allow transmit `netif_start_queue`)

## Roadmap

1. Enable interruptions
    - Kernel : `request_irq`, with an interrupt callback
    - Hardware : must set an interrupt mask, cf. datasheet p18  
    To receive it would be ROK (bit n°0) (and RER, bit n°1)  
    To transmit it would be
2. Enable receiver (transmitter)
    - Set RE (bit n°2) (TE, bit n°3) to Command Register 0x37

### Notes

- Ignore suspend & resume, see later
- Can we not use DMA ?
- Can we not use NAPI ?

# Interruptions

[Hardware : interrupt line](https://linux-kernel-labs.github.io/refs/heads/master/lectures/interrupts.html#programmable-interrupt-controller) :

> ![](https://linux-kernel-labs.github.io/refs/heads/master/_images/ditaa-5db1739b80a83b12505e4ff749b5e69fccd01f1b.png)
> 
> A device supporting interrupts has an output pin used for signaling an Interrupt ReQuest. IRQ pins are connected to a device named Programmable Interrupt Controller (PIC) which is connected to CPU's INTR pin.
> 
> A PIC usually has a set of ports used to exchange information with the CPU. When a device connected to one of the PIC's IRQ lines needs CPU attention the following flow happens:
> 
> device raises an interrupt on the corresponding IRQn pin
> PIC converts the IRQ into a vector number and writes it to a port for CPU to read
> PIC raises an interrupt on CPU INTR pin
> PIC waits for CPU to acknowledge an interrupt before raising another interrupt
> CPU acknowledges the interrupt then it starts handling the interrupt

An interrupt line is automatically set in `pci_dev` when a PCI device is registered.
It's then used to, for example, interact with the [generic interrupt handling layer](https://docs.kernel.org/core-api/genericirq.html#linux-generic-irq-handling).
Here, we need it for the [High-level Driver API](https://docs.kernel.org/core-api/genericirq.html#high-level-driver-api), and notably `request_irq()`, `free_irq()`, `disable_irq()`, `enable_irq()`.

# NAPI

[Kernel docs](https://docs.kernel.org/networking/napi.html) :
> NAPI is the event handling mechanism used by the Linux networking stack. The name NAPI no longer stands for anything in particular [1].
> 
> In basic operation the device notifies the host about new events via an interrupt. The host then schedules a NAPI instance to process the events. The device may also be polled for events via NAPI without receiving interrupts first (busy polling).
>
> NAPI processing usually happens in the software interrupt context, but there is an option to use separate kernel threads for NAPI processing.
>
> All in all NAPI abstracts away from the drivers the context and configuration of event (packet Rx and Tx) processing.
