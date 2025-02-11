/**
 * 8139c.c - RTL8139c driver
 * Please check https://wiki.osdev.org/RTL8139
 * for more information about magic values.
 */

#define DRV_NAME "8139c"
#define DRV_VERSION "0.0.1"

#include <linux/module.h>
#include <linux/pci.h>
#include <linux/init.h>
#include <linux/delay.h>
#include <linux/io.h>

#ifndef MAC_ADDRESS_MESSAGE
#define MAC_ADDRESS_MESSAGE "\b[RTL8139c] MAC address: tval_v0|%02x:%02x:%02x:%02x:%02x:%02x\n"
#endif // MAC_ADDRESS_MESSAGE

enum RTL8139c_registers {
	MAC0 = 0, /* Ethernet hardware address. */
	MAR0 = 8, /* Multicast filter. */
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

static const struct pci_device_id rtl8139c_pci_tbl[] = {
	{
		PCI_DEVICE(0x10EC, 0x8139),
	},
	{},
};
MODULE_DEVICE_TABLE(pci, rtl8139c_pci_tbl);

struct rtl8139c_priv {
	void __iomem *hwmem;
	char mac_address[6];
};

static void rtl8139c_print_mac_address(struct rtl8139c_priv *drv_priv)
{
	pr_info(MAC_ADDRESS_MESSAGE, drv_priv->mac_address[0],
		drv_priv->mac_address[1], drv_priv->mac_address[2],
		drv_priv->mac_address[3], drv_priv->mac_address[4],
		drv_priv->mac_address[5]);
}

static int rtl8139c_reset(struct rtl8139c_priv *drv_priv)
{
	writeb(CmdReset, drv_priv->hwmem + ChipCmd);
	int i = 1000;
	while (--i) {
		if (readb(drv_priv->hwmem + ChipCmd) & CmdReset) {
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
	// allocate memory for the driver private data
	struct rtl8139c_priv *drv_priv = kzalloc(sizeof(*drv_priv), GFP_KERNEL);
	// map the PCI device memory to the driver private data
	drv_priv->hwmem = ioremap(pciaddr, END);

	// save data for other functions
	pci_set_drvdata(pdev, drv_priv);

	// turn on the RTL8139
	writeb(0x0, drv_priv->hwmem + CONFIG1);

	// reset the RTL8139
	err = rtl8139c_reset(drv_priv);
	if (err) {
		pr_err("RTL8139c reset failed\n");
		pci_disable_device(pdev);
		return err;
	}

	// Get MAC address
	for (int i = 0; i < 6; i++) {
		drv_priv->mac_address[i] = readb(drv_priv->hwmem + MAC0 + i);
	}

	rtl8139c_print_mac_address(drv_priv);

	return 0;
}

static void rtl8139c_remove(struct pci_dev *pdev)
{
	struct rtl8139c_priv *drv_priv = pci_get_drvdata(pdev);
	if (drv_priv->hwmem) {
		iounmap(drv_priv->hwmem);
		kfree(drv_priv);
	}
	pr_info("RTL8139c removed\n");
}

// Can throw warns, but it's not a problem
static int __maybe_unused rtl8139c_resume(struct device *device)
{
	pr_info("RTL8139c resume\n");
	return 0;
}

static int __maybe_unused rtl8139c_suspend(struct device *device)
{
	pr_info("RTL8139c suspend\n");
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
