// SPDX-License-Identifier: GPL-2.0

//! Rust RTL8139 ethernet PCI driver.
//!
//! To make this driver probe, QEMU must be run with `-netdev user,id=mynet0 -device rtl8139,netdev=mynet0`.

use core::hint::black_box;
use kernel::{bindings, c_str, devres::Devres, pci, prelude::*};

struct Regs;

impl Regs {
    const MAC0: usize = 0; /* Ethernet hardware address. */
    const MAR0: usize = 8; /* Multicast filter. */
    const TxStatus0: usize = 0x10; /* Transmit status (Four 32bit registers). */
    const TxAddr0: usize = 0x20; /* Tx descriptors (also four 32bit). */
    const RxBuf: usize = 0x30;
    const ChipCmd: usize = 0x37;
    const RxBufPtr: usize = 0x38;
    const RxBufAddr: usize = 0x3A;
    const IntrMask: usize = 0x3C;
    const IntrStatus: usize = 0x3E;
    const TxConfig: usize = 0x40;
    const RxConfig: usize = 0x44;
    const Timer: usize = 0x48; /* A general-purpose counter. */
    const RxMissed: usize = 0x4C; /* 24 bits valid, write clears. */
    const Cfg9346: usize = 0x50;
    const Config0: usize = 0x51;
    const Config1: usize = 0x52;
    const TimerInt: usize = 0x54;
    const MediaStatus: usize = 0x58;
    const Config3: usize = 0x59;
    const Config4: usize = 0x5A; /* absent on RTL-8139A */
    const HltClk: usize = 0x5B;
    const MultiIntr: usize = 0x5C;
    const TxSummary: usize = 0x60;
    const BasicModeCtrl: usize = 0x62;
    const BasicModeStatus: usize = 0x64;
    const NWayAdvert: usize = 0x66;
    const NWayLPAR: usize = 0x68;
    const NWayExpansion: usize = 0x6A;
    /* Undocumented registers, but required for proper operation. */
    const FIFOTMS: usize = 0x70; /* FIFO Control and test. */
    const CSCR: usize = 0x74; /* Chip Status and Configuration Register. */
    const PARA78: usize = 0x78;
    const FlashReg: usize = 0xD4; /* Communication with Flash ROM, four bytes. */
    const PARA7c: usize = 0x7c; /* Magic transceiver parameter register. */
    const Config5: usize = 0xD8; /* absent on RTL-8139A, TODO: make sure this is 1 Byte */
    const END: usize = 0xD9;
}

enum ChipCmdBits {
    CmdReset = 0x10,
    CmdRxEnb = 0x08,
    CmdTxEnb = 0x04,
    RxBufEmpty = 0x01,
}

type Bar0 = pci::Bar<{ Regs::END }>;

struct Rtl8139Driver {
    pdev: pci::Device,
    bar: Devres<Bar0>,
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

enum MacGetError {
    BarRevoked,
}

/// hacky sleep() implemented by busy looping
fn sleep(amount: usize) {
    black_box({
        let mut acc = 0;
        for i in 0..amount {
            black_box({
                acc += 2;
            });
        }
    });
}

impl Rtl8139Driver {
    const DRV_NAME: &str = "8139rs";

    fn init(pdev: pci::Device, bar_res: Devres<Bar0>) -> Result<Self, InitError> {
        let bar = bar_res.try_access().ok_or(InitError::BarRevoked)?;

        // turn on
        bar.writeb(0x0, Regs::Config1);

        // software reset
        bar.writeb(ChipCmdBits::CmdReset as u8, Regs::ChipCmd);
        for i in 0..1000 {
            if (bar.readb(Regs::ChipCmd) & ChipCmdBits::CmdReset as u8) == 0 {
                break;
            }
            // TODO: find a way to sleep here (kernel crate doesn't seem to expose [`usleep()`])
            sleep(1_000);
        }
        if bar.readb(Regs::ChipCmd) & ChipCmdBits::CmdReset as u8 != 0 {
            return Err(InitError::SoftwareResetStuck);
        }

        dev_info!(pdev.as_ref(), "rtl8139rs init done!\n");
        Ok(Self { pdev, bar: bar_res }) // TODO: will return a [`net_device`] later
    }

    fn mac(&self) -> Result<[u8; 6], MacGetError> {
        let bar = self.bar.try_access().ok_or(MacGetError::BarRevoked)?;
        let mut mac = [0u8; 6];
        for i in 0..6 {
            mac[i] = bar.readb(Regs::MAC0 + i);
        }
        Ok(mac)
    }
}

impl pci::Driver for Rtl8139Driver {
    type IdInfo = ();

    const ID_TABLE: pci::IdTable<Self::IdInfo> = &PCI_TABLE;

    fn probe(pdev: &mut pci::Device, info: &Self::IdInfo) -> Result<Pin<KBox<Self>>> {
        dev_info!(
            pdev.as_ref(),
            "Probe Rust RTL8139 PCI driver (PCI ID: 0x{:x}, 0x{:x}).\n",
            pdev.vendor_id(),
            pdev.device_id()
        );

        pdev.enable_device_mem()?;
        pdev.set_master();

        let bar = pdev.iomap_region_sized::<{ Regs::END }>(1, c_str!("rust_rtl8139_driver"))?;

        // START
        let drv_instance = match Self::init(pdev.clone(), bar) {
            Ok(drv) => drv,
            Err(e) => {
                dev_err!(pdev.as_ref(), "rtl8139rs failed to init, reason: {:?}\n", e);
                return Err(ENXIO); // TODO: find a better error which doesn't do weird unexpected retries or anything else
            }
        };
        let drv_instance = KBox::new(drv_instance, GFP_KERNEL)?;

        let mac = drv_instance.mac().map_err(|_| ENXIO)?;
        dev_info!(pdev.as_ref(), "rtl8139rs mac: {:x?}\n", mac);

        Ok(drv_instance.into())
    }
}

impl Drop for Rtl8139Driver {
    fn drop(&mut self) {
        dev_dbg!(self.pdev.as_ref(), "Remove Rust RTL8139 PCI driver.\n");
    }
}

kernel::module_pci_driver! {
    type: Rtl8139Driver,
    name: "rust_rtl8139_driver",
    author: "Team 05",
    description: "Rust RTL8139 driver",
    license: "MIT",
}
