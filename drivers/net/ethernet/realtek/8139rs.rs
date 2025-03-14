// SPDX-License-Identifier: GPL-2.0

//! Rust RTL8139 ethernet PCI driver.
//!
//! To make this driver probe, QEMU must be run with `-netdev user,id=mynet0 -device rtl8139,netdev=mynet0`.

use core::{fmt, hint::black_box};
use kernel::{
    c_str,
    devres::Devres,
    net::{self, dev::DeviceOperations},
    pci,
    prelude::*,
};

struct Regs;

impl Regs {
    const MAC0: usize = 0; /* Ethernet hardware address. */
    const MAR0: usize = 8; /* Multicast filter. */
    const TX_STATUS0: usize = 0x10; /* Transmit status (Four 32bit registers). */
    const TX_ADDR0: usize = 0x20; /* Tx descriptors (also four 32bit). */
    const RX_BUF: usize = 0x30;
    const CHIP_CMD: usize = 0x37;
    const RX_BUF_PTR: usize = 0x38;
    const RX_BUF_ADDR: usize = 0x3A;
    const INTR_MASK: usize = 0x3C;
    const INTR_STATUS: usize = 0x3E;
    const TX_CONFIG: usize = 0x40;
    const RX_CONFIG: usize = 0x44;
    const TIMER: usize = 0x48; /* A general-purpose counter. */
    const RX_MISSED: usize = 0x4C; /* 24 bits valid, write clears. */
    const CFG9346: usize = 0x50;
    const CONFIG0: usize = 0x51;
    const CONFIG1: usize = 0x52;
    const TIMER_INT: usize = 0x54;
    const MEDIA_STATUS: usize = 0x58;
    const CONFIG3: usize = 0x59;
    const CONFIG4: usize = 0x5A; /* absent on RTL-8139A */
    const HLT_CLK: usize = 0x5B;
    const MULTI_INTR: usize = 0x5C;
    const TX_SUMMARY: usize = 0x60;
    const BASIC_MODE_CTRL: usize = 0x62;
    const BASIC_MODE_STATUS: usize = 0x64;
    const NWAY_ADVERT: usize = 0x66;
    const NWAY_LPAR: usize = 0x68;
    const NWAY_EXPANSION: usize = 0x6A;
    /* Undocumented registers, but required for proper operation. */
    const FIFOTMS: usize = 0x70; /* FIFO Control and test. */
    const CSCR: usize = 0x74; /* Chip Status and Configuration Register. */
    const PARA78: usize = 0x78;
    const FLASH_REG: usize = 0xD4; /* Communication with Flash ROM, four bytes. */
    const PARA7C: usize = 0x7c; /* Magic transceiver parameter register. */
    const CONFIG5: usize = 0xD8; /* absent on RTL-8139A, TODO: make sure this is 1 Byte */
    const END: usize = 0xD9;
}

enum ChipCmdBits {
    CmdReset = 0x10,
    CmdRxEnb = 0x08,
    CmdTxEnb = 0x04,
    RxBufEmpty = 0x01,
}

type Bar1 = pci::Bar<{ Regs::END }>;

/// Driver private data
struct DriverData {
    pdev: pci::Device,
    bar: Devres<Bar1>,
}

#[vtable]
impl DeviceOperations for DriverData {
    fn init(dev: &mut net::dev::Device<DriverData>) -> Result {
        let priv_data = dev.drv_priv_data();
        dev_info!(priv_data.pdev.as_ref(), "init called from device ops!\n");
        Ok(())
    }

    fn open(dev: &mut net::dev::Device<DriverData>) -> Result {
        let priv_data = dev.drv_priv_data();
        dev_info!(priv_data.pdev.as_ref(), "open called from device ops!\n");
        Ok(())
    }

    fn stop(dev: &mut net::dev::Device<DriverData>) -> Result {
        let priv_data = dev.drv_priv_data();
        dev_info!(priv_data.pdev.as_ref(), "stop called from device ops!\n");
        Ok(())
    }
}

/// Thing which linux has a reference to
struct Rtl8139Driver {
    ndev: net::dev::Device<DriverData>,
}

kernel::pci_device_table!(
    PCI_TABLE,
    MODULE_PCI_TABLE,
    <Rtl8139Driver as pci::Driver>::IdInfo,
    [(pci::DeviceId::from_id(0x10EC, 0x8139), ())]
);

#[derive(Debug)]
enum InitError {
    SoftwareResetStuck,
    BarRevoked,
}

#[derive(Debug)]
enum MacGetError {
    BarRevoked,
}

struct MacAddress([u8; 6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// hacky sleep() implemented by busy looping (kernel crate doesn't expose [`usleep()`])
fn sleep(amount: usize) {
    black_box({
        let mut _acc = 0;
        for _ in 0..amount {
            black_box({
                _acc += 2;
            });
        }
    });
}

impl DriverData {
    fn init(pdev: pci::Device, bar_res: Devres<Bar1>) -> Result<Self, InitError> {
        let bar = bar_res.try_access().ok_or(InitError::BarRevoked)?;

        // turn on
        bar.writeb(0x0, Regs::CONFIG1);

        // software reset
        bar.writeb(ChipCmdBits::CmdReset as u8, Regs::CHIP_CMD);
        for _ in 0..1000 {
            if (bar.readb(Regs::CHIP_CMD) & ChipCmdBits::CmdReset as u8) == 0 {
                break;
            }
            sleep(1_000);
        }
        if bar.readb(Regs::CHIP_CMD) & ChipCmdBits::CmdReset as u8 != 0 {
            return Err(InitError::SoftwareResetStuck);
        }

        dev_info!(pdev.as_ref(), "init done!\n");
        Ok(Self { pdev, bar: bar_res })
    }

    fn mac(&self) -> Result<MacAddress, MacGetError> {
        let bar = self.bar.try_access().ok_or(MacGetError::BarRevoked)?;
        let mut mac = [0u8; 6];
        for i in 0..6 {
            mac[i] = bar.readb(Regs::MAC0 + i);
        }
        Ok(MacAddress(mac))
    }
}

impl pci::Driver for Rtl8139Driver {
    type IdInfo = ();

    const ID_TABLE: pci::IdTable<Self::IdInfo> = &PCI_TABLE;

    fn probe(pdev: &mut pci::Device, _info: &Self::IdInfo) -> Result<Pin<KBox<Self>>> {
        dev_info!(
            pdev.as_ref(),
            "Probe Rust RTL8139 PCI driver (PCI ID: 0x{:x}, 0x{:x}).\n",
            pdev.vendor_id(),
            pdev.device_id()
        );

        pdev.enable_device_mem()?;
        pdev.set_master();

        let bar = pdev.iomap_region_sized::<{ Regs::END }>(1, c_str!("rust_rtl8139_driver"))?;
        let priv_data = match DriverData::init(pdev.clone(), bar) {
            Ok(d) => d,
            Err(e) => {
                dev_err!(pdev.as_ref(), "failed to init, reason: {:?}\n", e);
                return Err(ENXIO); // TODO: find a better error which doesn't do weird unexpected retries or anything else
            }
        };

        let mac = priv_data.mac().map_err(|_| ENXIO)?;
        dev_info!(pdev.as_ref(), "MacAddress: tval_v0|{}\n", mac);

        let mut ndev = net::dev::Device::try_new(1, 1, priv_data)?;
        ndev.set_parent(pdev);
        ndev.set_eth_hw_addr(mac.0.as_ref());

        ndev.register()?;
        dev_info!(pdev.as_ref(), "registered!\n");

        Ok(KBox::new(Rtl8139Driver { ndev }, GFP_KERNEL)?.into())
    }
}

kernel::module_pci_driver! {
    type: Rtl8139Driver,
    name: "rust_rtl8139_driver",
    author: "Team 05",
    description: "Rust RTL8139 driver",
    license: "MIT",
}
