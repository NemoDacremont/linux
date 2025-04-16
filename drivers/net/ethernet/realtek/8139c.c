/**
 * 8139c.c - RTL8139c driver
 * Please check https://wiki.osdev.org/RTL8139
 * for more information about magic values.
 */

#include "linux/netdevice.h"
#define DRV_NAME "8139c"
#define DRV_VERSION "0.0.1"

#include <linux/module.h>
#include <linux/pci.h>
#include <linux/etherdevice.h>
#include <linux/init.h>
#include <linux/delay.h>
#include <linux/io.h>

#include <linux/gfp.h> // has def of GFP_KERNEL
#include <linux/slab.h> // kmalloc, kfree
#include <linux/dma-mapping.h> // for dma_alloc_coherent
//#include <stdint.h>

#ifndef MAC_ADDRESS_MESSAGE
#define MAC_ADDRESS_MESSAGE \
	"\b[RTL8139c] MAC address: tval_v0|%02x:%02x:%02x:%02x:%02x:%02x\n"
#endif // MAC_ADDRESS_MESSAGE

#define TXPOLL 0x38

/* Number of Tx descriptor registers. */
#define NUM_TX_DESC 4
/* max supported ethernet frame size -- must be at least (dev->mtu+18+4).*/
#define MAX_ETH_FRAME_SIZE 1792
/* Size of the Tx bounce buffers -- must be at least (dev->mtu+18+4). */
#define TX_BUF_SIZE MAX_ETH_FRAME_SIZE
#define TX_BUF_TOT_LEN (TX_BUF_SIZE * NUM_TX_DESC)

static irqreturn_t rtl8139c_interrupt(int irq, void *dev_id);
static netdev_tx_t rtl8139c_start_xmit(struct sk_buff *skb,
				       struct net_device *dev);
static void rtl8139c_tx_interrupt(struct net_device *dev);
static int rtl8139c_dev_init(struct net_device *dev);
static int rtl8139c_open(struct net_device *dev);
static int rtl8139c_close(struct net_device *dev);
static int rtl8139c_ioctl(struct net_device *dev, struct ifreq *ifr, int cmd);

enum RTL8139c_registers {
	MAC0 = 0, /* Ethernet hardware address. */
	MAR0 = 8, /* Multicast filter. */
	TxStatus0 = 0x10, /* Transmit status (Four 32bit registers). */
	TxAddr0 = 0x20, /* Tx descriptors (also four 32bit). */
	RBSTART = 0x30, /* Receive buffer start address. */
	ChipCmd = 0x37, /* Command register. */
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

struct cp_desc {
	__le32 opts1;
	__le32 opts2;
	__le64 addr;
};

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

struct rtl8139c_priv {
	void __iomem *hwmem;
	char mac_address[6];
	struct pci_dev *pdev;
	struct net_device *dev;
	// send paquet

	u8 *tx_ring;
	struct sk_buff *tx_skb[NUM_TX_DESC];
	dma_addr_t tx_ring_dma;
	unsigned int tx_head;
	unsigned int tx_tail;
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

// To be implemented later
static int rtl8139c_open(struct net_device *dev)
{
	struct rtl8139c_priv *priv = netdev_priv(dev);
	int err;

	// Enable interrupt for transmission (TxOK = bit 2)
	writew((1 << 2), priv->hwmem + IMR);

	// enable Tx + Rx
	writeb(CmdTxEnb | CmdRxEnb, priv->hwmem + ChipCmd);

	for (unsigned int i = 0; i < NUM_TX_DESC; ++i)
		writel(priv->tx_ring_dma + (TX_BUF_SIZE * i),
		       priv->hwmem + TxAddr0 + (i * 4));

	// save interrupt
	err = request_irq(priv->pdev->irq, rtl8139c_interrupt, IRQF_SHARED,
			  dev->name, dev);
	if (err) {
		pr_err("[RTL8139c] request_irq failed\n");
		return err;
	}

	netif_start_queue(dev);

	pr_info("\b[RTL8139c] open\n");
	return 0;
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

// tester de mettre les options first et last frags
static netdev_tx_t rtl8139c_start_xmit(struct sk_buff *skb,
				       struct net_device *dev)
{
	//skb->len+=5000;
	struct rtl8139c_priv *priv = netdev_priv(dev);
	pr_info(">>> xmit packet len=%d\n", skb->len);

	if (skb_shinfo(skb)->nr_frags > 0)
		pr_info("TX: skb contains %d fragments", skb_shinfo(skb)->nr_frags);

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
	u32 tsd = skb->len & 0x1FFF;         // Taille
	tsd |= (0x3F << 16);                 // Threshold max

	writel(tsd, priv->hwmem + TxStatus0 + (entry * 4));

	priv->tx_head = next;
	pr_info(">>> xmit end");
	return NETDEV_TX_OK;
}

static irqreturn_t rtl8139c_interrupt(
	int irq,
	void *dev_id) // interrupt handler of driver (cp_interrupt in official), irq : nÂ° interrupt
{
	struct net_device *dev = dev_id;
	struct rtl8139c_priv *priv = netdev_priv(dev);

	u16 status = readw(priv->hwmem + ISR); // ISR has interrupt cause
	if (!status) // as in official, if any bit, not our interrupt
		return IRQ_NONE;

	writew(status,
	       priv->hwmem +
		       ISR); // as in official, uses to clear 1 bits to 0. prevent loop interrupt

	if (status & (1 << 2)) // TxOK
		pr_info("INTERRUPT STATUS: 0x%x\n", status);
		rtl8139c_tx_interrupt(
			dev); // has to do everything necessary when txOk, means free, moove pointer..

	return IRQ_HANDLED;
}

static void rtl8139c_tx_interrupt(struct net_device *dev)
{
	struct rtl8139c_priv *priv = netdev_priv(dev);

	while (priv->tx_tail != priv->tx_head) { // while there are still descriptor treated by card
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
