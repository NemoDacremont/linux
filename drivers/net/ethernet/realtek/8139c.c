/**
 * 8139c.c - RTL8139c driver
 * Please check https://wiki.osdev.org/RTL8139
 * for more information about magic values.
 */

#define DRV_NAME "8139c"
#define DRV_VERSION "0.0.1"

#include <linux/module.h>
#include <linux/pci.h>
#include <linux/etherdevice.h>
#include <linux/init.h>
#include <linux/delay.h>
#include <linux/io.h>

#ifndef MAC_ADDRESS_MESSAGE
#define MAC_ADDRESS_MESSAGE \
	"\b[RTL8139c] MAC address: tval_v0|%02x:%02x:%02x:%02x:%02x:%02x\n"
#endif // MAC_ADDRESS_MESSAGE

/* Number of Tx descriptor registers. */
#define NUM_TX_DESC 4
/* max supported ethernet frame size -- must be at least (dev->mtu+18+4).*/
#define MAX_ETH_FRAME_SIZE 1792
/* Size of the Tx bounce buffers -- must be at least (dev->mtu+18+4). */
#define TX_BUF_SIZE MAX_ETH_FRAME_SIZE
#define TX_BUF_TOT_LEN (TX_BUF_SIZE * NUM_TX_DESC)

enum RTL8139c_registers {
	MAC0 = 0, /* Ethernet hardware address. */
	MAR0 = 8, /* Multicast filter. */
	TxStatus0 = 0x10, /* Transmit status (Four 32bit registers). */
	TxAddr0 = 0x20, /* Tx descriptors (also four 32bit). */
	RBSTART = 0x30, /* Receive buffer start address. */
	ChipCmd = 0x37, /* Command register. */
	CAPR = 0x38,
	RxConfig = 0x44,
	IMR = 0x3C, /* Interrupt mask register. */
	ISR = 0x3E, /* Interrupt status register. */
	CONFIG1 = 0x52, /* Configuration register 1. */
	END = 0x5B, /* End of RTL8139 registers. */
};

enum ChipCmdBits {
	CmdReset = 0x10,
	CmdRxEnb = 0x08,
	CmdTxEnb = 0x04,
	RxBufEmpty = 0x01,
};

enum IntrStatus {
	RxOvw = (1 << 4),
	TxOK = (1 << 2), /* Tx */
	RxErr = (1 << 1), /* Rx error */
	RxOK = (1 << 0) /* Rx packet received */
};

enum RxConfig {
	/* RxConfig register */
	// Bits 11 and 12 to 1
	RxBufferLengthMax = 0x1800,
	// Bits 11 and 12 to 0
	RxBufferLengthMin = ~RxBufferLengthMax,
	RxWrap = 0x80,
	RxCfgFIFOShift = 13, /* Shift, to get Rx FIFO thresh value */
	RxCfgDMAShift = 8, /* Shift, to get Rx Max DMA value */
	AcceptErr = 0x20, /* Accept packets with CRC errors */
	AcceptRunt = 0x10, /* Accept runt (<64 bytes) packets */
	AcceptBroadcast = 0x08, /* Accept broadcast packets */
	AcceptMulticast = 0x04, /* Accept multicast packets */
	AcceptMyPhys = 0x02, /* Accept pkts with our MAC as dest */
	AcceptAllPhys = 0x01, /* Accept all pkts w/ physical dest */
};

struct rtl8139c_priv {
	void __iomem *hwmem;
	void *rx_ring;
	dma_addr_t dma_handle;
	char mac_address[6];
	struct pci_dev *pdev;
	struct net_device *dev;
	u8 *tx_ring;
	struct sk_buff *tx_skb[NUM_TX_DESC];
	dma_addr_t tx_ring_dma;
	unsigned int tx_head;
	unsigned int tx_tail;
};

struct cp_desc {
	__le32 opts1;
	__le32 opts2;
	__le64 addr;
};

static const struct pci_device_id rtl8139c_pci_tbl[] = {
	{
		PCI_DEVICE(0x10EC, 0x8139),
	},
	{},
};
MODULE_DEVICE_TABLE(pci, rtl8139c_pci_tbl);

// To be implemented laterskb
static int rtl8139c_dev_init(struct net_device *dev)
{
	// permet d'indiquer au noyau que le driver ne supporte pas GSO : la segmentation est fait côté noyau et pas côté driver.
	//pr_info("je passe bien par la fonction init");
	dev->features &= ~(NETIF_F_TSO | NETIF_F_GSO);
	dev->hw_features &= ~(NETIF_F_TSO | NETIF_F_GSO);
	dev->vlan_features &= ~(NETIF_F_TSO | NETIF_F_GSO);

	pr_info("\b[RTL8139c] dev init\n");
	return 0;
}

/**
 * @brief Handy function to print a packet by hexadecimal groups
 * 
 * @param ptr Pointer to packet data
 * @param length Length of packet
 */
static void print_packet(void *ptr, unsigned int length)
{
	for (size_t i = 0; i < length; ++i)
		pr_debug("%02X ", *((u8 *)ptr + i));
	pr_debug("\n");
}

static void transmit_handler(struct net_device *dev, u16 status)
{
	struct rtl8139c_priv *priv = netdev_priv(dev);

	if (status & (RxOvw | RxOK))
		writew(status | TxOK, priv->hwmem + ISR);

	while (priv->tx_tail !=
	       priv->tx_head) { // while there are still descriptor treated by card
		unsigned int entry = priv->tx_tail;

		// if card has descripor (OWN=1) we stop
		// if (priv->tx_ring[entry].opts1 & cpu_to_le32(1 << 31))
		// 	break;

		// free skb if not
		if (priv->tx_skb[entry]) {
			dev_kfree_skb(priv->tx_skb[entry]);
			priv->tx_skb[entry] = NULL;
			pr_info("freed skb at entry %u\n", entry);
		}

		// next
		priv->tx_tail = (priv->tx_tail + 1) % NUM_TX_DESC;
	}

	// if queue was stopped due to a fully ring
	if (!netif_queue_stopped(dev))
		return;

	if ((priv->tx_head + 1) % NUM_TX_DESC != priv->tx_tail)
		netif_wake_queue(dev);
}

static unsigned int receive_handler(struct net_device *dev, u16 status)
{
	struct rtl8139c_priv *priv = netdev_priv(dev);

	if (status & (RxOvw | RxOK))
		writew(status | RxOK, priv->hwmem + ISR);

	unsigned char buf_empty = readb(priv->hwmem + ChipCmd);

	if (status & RxOK) {
		u16 CAPR_read;
		u16 start_offset;

		u16 *header_ptr;
		u16 length;

		do {
			CAPR_read = readw(priv->hwmem + CAPR);
			// pr_info("CAPR: %d\n", CAPR_read);

			// Make the arithmetic here to guarantee wrapping!
			// Check if we can add 16 else wrap
			if (CAPR_read < 0xFFF0) {
				start_offset = CAPR_read + 16;
			} else {
				start_offset =
					0x0010 - (0x0001 + 0xFFFF - CAPR_read);
			}

			//CAPR_read = 0xFA;
			start_offset = CAPR_read + 16;
			pr_info("Start offset : %d\n", start_offset);

			header_ptr =
				(u16 *)((u8 *)priv->rx_ring + start_offset);
			length = *(header_ptr + 1);

			pr_debug("Size of received packet: %d\n", length);
			// pr_debug("Received packet:");
			// print_packet(header_ptr + 2, length - 4);

			// Allocate a sk_buff
			struct sk_buff *skb =
				netdev_alloc_skb_ip_align(dev, length - 4);
			// Copy raw packet to sk_buff
			skb_copy_to_linear_data(skb, header_ptr + 2,
						length - 4);
			// Tell sk_buff to consume copied data
			skb_put(skb, length - 4);

			// TO REVIEW
			skb->ip_summed = CHECKSUM_UNNECESSARY;

			// Parse protocol
			skb->protocol = eth_type_trans(skb, dev);

			// Debug
			// struct iphdr *iph = ip_hdr(skb);
			// pr_debug("skb proto: %d\n", iph->protocol);

			// Send it to upper layer
			int res = netif_receive_skb(skb);
			pr_debug("netif receive status: %d\n", res);

			// Move CAPR after the read packet
			writew((CAPR_read + length + 4 + 3) & 0xFFFC,
			       priv->hwmem + CAPR);
			// Update status of empty receive buffer, should become empty (1) very fast
			buf_empty = readb(priv->hwmem + ChipCmd);
		} while (buf_empty == 0);
	}

	if (status & RxErr) {
		pr_info("Error in reception");
	}
	if (status & RxOvw) {
		pr_info("Overflow during reception");
	}

	return 0;
}

static irqreturn_t interrupt_handler(int irq, void *dev_instance)
{
	pr_info("\b[RTL8139c] interrupted\n");

	struct net_device *dev = dev_instance;
	struct rtl8139c_priv *priv = netdev_priv(dev);

	u16 status = readw(priv->hwmem + ISR);

	// Reception
	if (status & (RxOK | RxErr | RxOvw))
		receive_handler(dev, status);

	// Transmission
	if (status & TxOK)
		// has to do everything necessary when txOk, means free, moove pointer..
		transmit_handler(dev, status);

	return IRQ_HANDLED;
}

// To be implemented later
static int rtl8139c_open(struct net_device *dev)
{
	struct rtl8139c_priv *priv = netdev_priv(dev);

	for (unsigned int i = 0; i < NUM_TX_DESC; ++i)
		writel(priv->tx_ring_dma + (TX_BUF_SIZE * i),
		       priv->hwmem + TxAddr0 + (i * 4));

	netif_start_queue(dev);

	pr_info("\b[RTL8139c] open\n");

	priv->rx_ring = dma_alloc_coherent(&priv->pdev->dev,
					   8 * 1024 + 16 + 1536,
					   &priv->dma_handle, GFP_KERNEL);
	writel(priv->dma_handle, priv->hwmem + RBSTART);

	writeb(CmdTxEnb | CmdRxEnb, priv->hwmem + ChipCmd);

	int rc = request_irq(priv->pdev->irq, interrupt_handler, IRQF_SHARED,
			     dev->name, dev);
	if (rc)
		pr_err("\b[RTL8139c] Open error on request_irq \n");

	int rx_config_read = readl(priv->hwmem + RxConfig);
	writel((rx_config_read | RxWrap | AcceptBroadcast | AcceptMulticast |
		AcceptMyPhys) &
		       RxBufferLengthMin,
	       priv->hwmem + RxConfig);

	writew(0xfff0, priv->hwmem + CAPR);

	writew(RxOvw | RxOK | RxErr | TxOK, priv->hwmem + IMR);

	return 0;
}

// tester de mettre les options first et last frags
static netdev_tx_t rtl8139c_start_xmit(struct sk_buff *skb,
				       struct net_device *dev)
{
	//skb->len+=5000;
	struct rtl8139c_priv *priv = netdev_priv(dev);
	pr_info(">>> xmit packet len=%d\n", skb->len);

	if (skb_shinfo(skb)->nr_frags > 0)
		pr_info("TX: skb contains %d fragments",
			skb_shinfo(skb)->nr_frags);

	unsigned int entry = priv->tx_head;
	unsigned int next = (entry + 1) % NUM_TX_DESC;

	if (skb->len > dev->mtu + ETH_HLEN) { // verif mtu
		pr_warn("skb too big (%u > %u), error\n", skb->len, dev->mtu);
		dev_kfree_skb_any(skb);
		dev->stats.tx_dropped++;
		return NETDEV_TX_OK;
	}

	if (next == priv->tx_tail) { // check space
		// no space => requeue later
		pr_info("no space for transmission");
		netif_stop_queue(dev);
		return NETDEV_TX_BUSY;
	}

	// copie dans le buffer DMA
	skb_copy_and_csum_dev(skb, priv->tx_ring + (entry * TX_BUF_SIZE));
	priv->tx_skb[entry] = skb;

	// @ physique du buffer
	writel(priv->tx_ring_dma + (entry * TX_BUF_SIZE),
	       priv->hwmem + TxAddr0 + (entry * 4));

	// bit 0–12 : taille
	// bit 16–21 : Tx Threshold
	// bit 13 (OWN=0) => démarre transmission
	u32 tsd = skb->len & 0x1FFF; // Taille
	tsd |= (0x3F << 16); // Threshold max

	writel(tsd, priv->hwmem + TxStatus0 + (entry * 4));

	priv->tx_head = next;
	pr_info(">>> xmit end");
	return NETDEV_TX_OK;
}

// To be implemented later
static int rtl8139c_close(struct net_device *dev)
{
	struct rtl8139c_priv *priv = netdev_priv(dev);

	netif_stop_queue(dev); // stop transmission
	free_irq(priv->pdev->irq, dev); // free irq
	pr_info("\b[RTL8139c] close\n");
	return 0;
}

// To be implemented later
static int rtl8139c_ioctl(struct net_device *dev, struct ifreq *ifr, int cmd)
{
	pr_info("\b[RTL8139c] ioctl\n");
	return 0;
}

/**
 * Operations with the interface
 */
static const struct net_device_ops rtl8139c_netdev_ops = {
	.ndo_init = rtl8139c_dev_init,
	.ndo_open = rtl8139c_open,
	.ndo_stop = rtl8139c_close,
	.ndo_start_xmit = rtl8139c_start_xmit,
	.ndo_do_ioctl = rtl8139c_ioctl,
	.ndo_validate_addr = eth_validate_addr,
};

static void rtl8139c_print_mac_address(struct rtl8139c_priv *drv_priv)
{
	pr_info(MAC_ADDRESS_MESSAGE, drv_priv->mac_address[0],
		drv_priv->mac_address[1], drv_priv->mac_address[2],
		drv_priv->mac_address[3], drv_priv->mac_address[4],
		drv_priv->mac_address[5]);
}

/**
 * Reset network card.
 * @param drv_priv private driver's data
 * @return zero if failed positive either
 */
static int rtl8139c_reset(struct rtl8139c_priv *drv_priv)
{
	writeb(CmdReset, drv_priv->hwmem + ChipCmd);
	int i = 1000;
	while (--i) {
		if ((readb(drv_priv->hwmem + ChipCmd) & CmdReset) == 0) {
			break;
		}
		udelay(10);
	}
	return i;
}

static int rtl8139c_probe(struct pci_dev *pdev, const struct pci_device_id *ent)
{
	u16 vendor, device;
	pci_read_config_word(pdev, PCI_VENDOR_ID, &vendor);
	pci_read_config_word(pdev, PCI_DEVICE_ID, &device);
	printk(KERN_INFO "Device vid: 0x%X pid: 0x%X\n", vendor, device);

	struct net_device *dev;
	struct rtl8139c_priv *priv;

	dev = alloc_etherdev(sizeof(struct rtl8139c_priv));
	if (!dev) {
		return -ENOMEM;
	}
	SET_NETDEV_DEV(dev, &pdev->dev);

	priv = netdev_priv(dev);
	priv->pdev = pdev;
	priv->dev = dev;

	// enable pci device
	int err = pci_enable_device_mem(pdev);
	// enable bus mastering
	pci_set_master(pdev);

	if (err) {
		pci_disable_device(pdev);
		return err;
	}

	// request regions for memory mapped I/O
	err = pci_request_regions(pdev, DRV_NAME);
	if (err) {
		pci_disable_device(pdev);
		return err;
	}

	// get the base address of the PCI device
	resource_size_t pciaddr = pci_resource_start(pdev, 1);

	// map the PCI device memory to the driver private data
	priv->hwmem = ioremap(pciaddr, END);

	// tx ring allocation

	priv->tx_ring = dma_alloc_coherent(&pdev->dev, TX_BUF_TOT_LEN,
					   &priv->tx_ring_dma, GFP_KERNEL);

	if (!priv->tx_ring) {
		pr_err("\b[RTL8139c] Failed to allocate TX ring\n");
		iounmap(priv->hwmem);
		pci_disable_device(pdev);
		return -ENOMEM;
	}

	// save data for other functions
	pci_set_drvdata(pdev, priv);

	// turn on the RTL8139
	writeb(0x0, priv->hwmem + CONFIG1);

	// reset the RTL8139
	err = rtl8139c_reset(priv);
	if (err == 0) {
		pr_err("\b[RTL8139c] reset failed\n");
		pci_disable_device(pdev);
		return err;
	}

	// Get MAC address
	for (int i = 0; i < 6; i++) {
		priv->mac_address[i] = readb(priv->hwmem + MAC0 + i);
	}

	rtl8139c_print_mac_address(priv);
	// mandatory to execute register_netdev line 6
	// https://stackoverflow.com/q/6726939
	dev->netdev_ops = &rtl8139c_netdev_ops;
	eth_hw_addr_set(dev, priv->mac_address);

	unsigned rc = register_netdev(dev);
	if (rc) {
		iounmap(priv->hwmem);
		return -1;
	}

	return 0;
}

static void rtl8139c_remove(struct pci_dev *pdev)
{
	struct rtl8139c_priv *drv_priv = pci_get_drvdata(pdev);

	if (drv_priv->tx_ring) {
		dma_free_coherent(&pdev->dev,
				  sizeof(struct cp_desc) * NUM_TX_DESC,
				  drv_priv->tx_ring, drv_priv->tx_ring_dma);
		drv_priv->tx_ring = NULL;
	}

	if (drv_priv->hwmem) {
		iounmap(drv_priv->hwmem);
		drv_priv->hwmem = NULL;
	}

	free_netdev(drv_priv->dev); // libÃ¨re la struct net_device + drv_priv

	pr_info("\b[RTL8139c] removed\n");
}

// Can throw warns, but it's not a problem
static int __maybe_unused rtl8139c_resume(struct device *device)
{
	pr_info("\b[RTL8139c] resume\n");
	return 0;
}

static int __maybe_unused rtl8139c_suspend(struct device *device)
{
	pr_info("\b[RTL8139c] suspend\n");
	return 0;
}

static SIMPLE_DEV_PM_OPS(rtl8139c_pm_ops, rtl8139c_suspend, rtl8139c_resume);

static struct pci_driver rtl8139c_pci_driver = {
	.name = DRV_NAME,
	.id_table = rtl8139c_pci_tbl,
	.probe = rtl8139c_probe,
	.remove = rtl8139c_remove,
	.driver.pm = &rtl8139c_pm_ops,
};

static int __init rtl8139c_init_module(void)
{
#ifdef MODULE
	pr_info(RTL8139c_DRIVER_NAME "\n");
#endif

	return pci_register_driver(&rtl8139c_pci_driver);
}

static void __exit rtl8139c_cleanup_module(void)
{
	pr_info("Cleaning up module.\n");
}

module_init(rtl8139c_init_module);
module_exit(rtl8139c_cleanup_module);

MODULE_LICENSE("MIT");
MODULE_AUTHOR("TEAM 05");
MODULE_DESCRIPTION("C RTL8139 driver");
MODULE_VERSION(DRV_VERSION);
