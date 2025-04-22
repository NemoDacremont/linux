// SPDX-License-Identifier: GPL-2.0

//! Rust RTL8139 ethernet PCI driver.
//!
//! To make this driver probe, QEMU must be run with `-netdev user,id=mynet0 -device rtl8139,netdev=mynet0`.
//! to setup inferface and test sending packets:
//! ```shell
//! ip link set eth0 up
//! ip addr add 192.168.100.2/24 dev eth0
//! ping 192.168.100.1
//! ```

use core::{fmt, hint::black_box, mem};
use kernel::{
    c_str,
    devres::Devres,
    dma::{self, *},
    error::Error,
    irq::request::{flags, Handler, IrqReturn, Registration},
    net::{
        self,
        dev::{DeviceOperations, SkBuff, TxCode},
    },
    new_mutex, pci,
    prelude::*,
    sync::Mutex,
};

/// max supported ethernet frame size. must be at least (mtu + 18 + 4)
const MAX_ETH_FRAME_SIZE: usize = 1792;
const TX_BUF_LEN: usize = MAX_ETH_FRAME_SIZE;
const NUM_TX_BUFS: usize = 4;
const ALL_TX_BUF_LEN: usize = NUM_TX_BUFS * TX_BUF_LEN;

struct Regs;

impl Regs {
    const MAC0: usize = 0; /* Ethernet hardware address. */
    const MAR0: usize = 8; /* Multicast filter. */
    const TX_STATUS0: usize = 0x10; /* Transmit status (Four 32bit registers). */
    const TX_ADDR0: usize = 0x20; /* Tx descriptors (also four 32bit). */
    const RX_BUF: usize = 0x30;
    const CHIP_CMD: usize = 0x37;
    const CAPR: usize = 0x38;
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

mod chip_cmd_bits {
    pub const CMD_RESET: u8 = 0x10;
    pub const CMD_RX_ENABLE: u8 = 0x08;
    pub const CMD_TX_ENABLE: u8 = 0x04;
    pub const RX_BUF_EMPTY: u8 = 0x01;
}

mod interrupt_status {
    pub const RX_OK: u16 = 1 << 0;
    pub const RX_ERR: u16 = 1 << 1;
    pub const TX_OK: u16 = 1 << 2;
    pub const RX_OVERFLOW: u16 = 1 << 4;
    pub const TX_ERR: u16 = 1 << 3;
}

mod rx_config {
    /* rx_config register */
    // Bits 11 and 12 to 1
    pub const RX_BUFFER_LENGTH_MAX: u32 = 0x1800;
    // Bits 11 and 12 to 0
    pub const RX_BUFFER_LENGTH_MIN: u32 = !0x1800;
    pub const RX_WRAP: u32 = 0x80;
    pub const ACCEPT_ERR: u32 = 0x20; /* Accept packets with CRC errors */
    pub const ACCEPT_RUNT: u32 = 0x10; /* Accept runt (<64 bytes) packets */
    pub const ACCEPT_BROADCAST: u32 = 0x08; /* Accept broadcast packets */
    pub const ACCEPT_MULTICAST: u32 = 0x04; /* Accept multicast packets */
    pub const ACCEPT_MY_PHYS: u32 = 0x02; /* Accept pkts with our MAC as dest */
    pub const ACCEPT_ALL_PHYS: u32 = 0x01; /* Accept all pkts w/ physical dest */
}

type Bar1 = pci::Bar<{ Regs::END }>;

/// Driver private data
struct DriverData {
    pdev: pci::Device,
    bar: Devres<Bar1>,
    rx_buf_dma_handle: CoherentAllocation<u8>,
    // tx_skb_buf: [Option<SkBuff>; NUM_TX_BUFS],
    tx_buf_dma_handle: CoherentAllocation<u8>,
    tx_buf_head: usize,
    tx_buf_tail: usize,
}

struct InterruptHandler {
    ndev: Pin<KBox<Mutex<net::dev::Device<DriverData>>>>,
}

// fn handle_receive(ndev_lock: Guard)

impl Handler for InterruptHandler {
    fn handle_irq(&self) -> IrqReturn {
        let ndev_lock = self.ndev.lock();
        let priv_data = ndev_lock.drv_priv_data();
        // dev_info!(priv_data.pdev.as_ref(), "irq!\n");
        let bar = if let Some(bar) = priv_data.bar.try_access() {
            bar
        } else {
            dev_err!(
                priv_data.pdev.as_ref(),
                "couldn't access bar in irq handler!\n"
            );
            return IrqReturn::None;
        };

        let status = bar.readw(Regs::INTR_STATUS);
        if status & (interrupt_status::RX_OVERFLOW | interrupt_status::RX_OK) != 0 {
            bar.writew(interrupt_status::RX_OK, Regs::INTR_STATUS);
        }
        if status & (interrupt_status::TX_OK) != 0 {
            bar.writew(interrupt_status::TX_OK, Regs::INTR_STATUS);
        }

        // unsigned char buf_empty = readb(priv->hwmem + ChipCmd);
        // let mut is_rx_buff_empty = bar.readb(Regs::CHIP_CMD) != 0;
        // pr_err!("status={} is_empty={}\n", status, is_rx_buff_empty);
        if (status & interrupt_status::RX_OK) != 0 {
            dev_info!(
                priv_data.pdev.as_ref(),
                "RX_OK interruption, card received a packet\n"
            );
            let mut is_rx_buff_empty = false;
            while !is_rx_buff_empty {
                let capr = bar.readw(Regs::CAPR);
                let start_offset = capr.wrapping_add(16);
                // Qemu source says this is offset by 16 bits
                let header_ptr = unsafe {
                    priv_data
                        .rx_buf_dma_handle
                        .start_ptr()
                        .add(start_offset as usize) as *const u16
                };
                let length = unsafe { header_ptr.wrapping_add(1).read_volatile() } as usize;

                /*                dev_info!(
                    priv_data.pdev.as_ref(),
                    "Size of received packet: {}\n",
                    length
                ); */

                let packet = unsafe {
                    core::slice::from_raw_parts(header_ptr.add(2) as *const u8, length - 4)
                };
                // dev_info!(priv_data.pdev.as_ref(), "packet: {:X?}\n", packet);

                let skb = ndev_lock.new_skb_from_slice(packet);
                // dev_info!(priv_data.pdev.as_ref(), "skb data: {:X?}\n", skb.data());
                ndev_lock.netif_receive_skb(skb);
                dev_info!(
                    priv_data.pdev.as_ref(),
                    "gave the received packet to linux\n"
                );

                bar.writew(
                    capr.wrapping_add(length as u16 + 4 + 3) & 0xFFFC, // Why 0xFFFC ?!
                    Regs::CAPR,
                );

                is_rx_buff_empty = bar.readb(Regs::CHIP_CMD) != 0;
            }
        } else if (status & interrupt_status::TX_OK) != 0 {
            dev_info!(
                priv_data.pdev.as_ref(),
                "TX_OK interruption, card sent the packet\n"
            );

            while priv_data.tx_buf_tail != priv_data.tx_buf_head {
                let entry = priv_data.tx_buf_tail;

                // free skb if not
                // match &priv_data.tx_skb_buf[entry] {
                //     Some(skb) => {
                //         skb.consume();
                //         priv_data.tx_skb_buf[entry] = None;
                //         dev_info!(priv_data.pdev.as_ref(), "Free skb at entry {}\n", entry);
                //     }
                //     None => {}
                // }

                priv_data.tx_buf_tail = (priv_data.tx_buf_tail + 1) % NUM_TX_BUFS;
            }

            // if ndev_lock.netif_queue_stopped() {
            //     return IrqReturn::None;
            // }

            if ndev_lock.netif_queue_stopped()
                && ((priv_data.tx_buf_head + 1) % NUM_TX_BUFS != priv_data.tx_buf_tail)
            {
                dev_info!(
                    priv_data.pdev.as_ref(),
                    "now got space for a packet, reactivating netif queue\n"
                );
                ndev_lock.netif_wake_queue();
            }
        } else if status & interrupt_status::TX_ERR != 0 {
            dev_info!(priv_data.pdev.as_ref(), "TX_ERR\n");
        }

        IrqReturn::Handled
    }
}

#[vtable]
impl DeviceOperations for DriverData {
    fn init(dev: &mut net::dev::Device<DriverData>) -> Result {
        let priv_data = dev.drv_priv_data();
        dev_info!(priv_data.pdev.as_ref(), "init called from device ops!\n");

        // dev.set_features(dev.get_features() & !(0 | 1));

        Ok(())
    }

    fn open(dev: &mut net::dev::Device<DriverData>) -> Result<(), Error> {
        let priv_data = dev.drv_priv_data();
        // change Error later
        let bar = priv_data.bar.try_access().ok_or(ENXIO)?;
        dev_info!(priv_data.pdev.as_ref(), "open called from device ops!\n");

        bar.writel(
            priv_data.rx_buf_dma_handle.dma_handle() as u32,
            Regs::RX_BUF,
        );
        for i in 0..NUM_TX_BUFS {
            bar.writel(
                priv_data.tx_buf_dma_handle.dma_handle() as u32 + (i * TX_BUF_LEN) as u32,
                Regs::TX_ADDR0 + (i * 4), // each of these registers is 4 bytes
            );
        }

        let rx_config_read = bar.readl(Regs::RX_CONFIG);
        bar.writel(
            (rx_config_read
                | rx_config::RX_WRAP
                | rx_config::ACCEPT_BROADCAST
                | rx_config::ACCEPT_MULTICAST
                | rx_config::ACCEPT_MY_PHYS)
                & rx_config::RX_BUFFER_LENGTH_MIN,
            Regs::RX_CONFIG,
        );

        bar.writew(0xFFF0, Regs::CAPR);

        bar.writew(
            interrupt_status::RX_OVERFLOW
                | interrupt_status::RX_OK
                | interrupt_status::RX_ERR
                | interrupt_status::TX_OK,
            Regs::INTR_MASK,
        );

        dev.netif_start_queue();

        bar.writeb(
            chip_cmd_bits::CMD_RX_ENABLE | chip_cmd_bits::CMD_TX_ENABLE,
            Regs::CHIP_CMD,
        );

        Ok(())
    }

    fn stop(dev: &mut net::dev::Device<DriverData>) -> Result {
        let priv_data = dev.drv_priv_data();
        dev_info!(priv_data.pdev.as_ref(), "stop called from device ops!\n");
        Ok(())
    }

    fn start_xmit(dev: &mut net::dev::Device<Self>, skb: SkBuff) -> TxCode {
        let priv_data = dev.drv_priv_data();
        dev_info!(priv_data.pdev.as_ref(), "xmit called from device ops!\n");
        dev_err!(priv_data.pdev.as_ref(), "{:02X?}\n", skb.data());

        let buf_index = priv_data.tx_buf_head;
        let next_buf_index = (buf_index + 1) % NUM_TX_BUFS;

        // TODO : 14 is ETH_HLEN
        if skb.data().len() > (dev.get_mtu() + 14) as usize {
            dev_err!(priv_data.pdev.as_ref(), "skb too big !\n");
            skb.consume();
            return TxCode::Ok;
        }

        if next_buf_index == priv_data.tx_buf_tail {
            dev_info!(priv_data.pdev.as_ref(), "no space for tx\n");
            dev.netif_stop_queue();
            // skb.consume();
            mem::forget(skb);
            return TxCode::Busy;
        }

        let tx_buf = unsafe {
            core::slice::from_raw_parts_mut(
                priv_data
                    .tx_buf_dma_handle
                    .start_ptr_mut()
                    .wrapping_add(TX_BUF_LEN * buf_index),
                TX_BUF_LEN,
            )
        };
        skb.copy_to_with_checksum(tx_buf);

        // TODO: docs
        let mut tsd: u32 = skb.data().len() as u32 & 0x1FFFu32; // get lowest 13 bits of the length
        tsd &= !((1 as u32) << 13);
        // tsd |= 0x3F << 16;

        // dev.drv_priv_data().tx_skb_buf[buf_index] = Some(skb);

        // mem::forget(skb);

        match priv_data.bar.try_access() {
            Some(bar) => {
                bar.try_writel(
                    tsd,
                    Regs::TX_STATUS0 + buf_index * core::mem::size_of::<u32>(),
                )
                .unwrap();

                priv_data.tx_buf_head = next_buf_index;

                dev_info!(priv_data.pdev.as_ref(), "told card to send!\n");
            }
            None => {
                dev_err!(priv_data.pdev.as_ref(), "failed to access bar\n");
            }
        }

        TxCode::Ok
    }
}

/// Thing which linux has a reference to
#[pin_data]
struct Rtl8139Driver {
    #[pin]
    registration: Registration<InterruptHandler>,
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
        bar.writeb(chip_cmd_bits::CMD_RESET, Regs::CHIP_CMD);
        for _ in 0..1000 {
            if (bar.readb(Regs::CHIP_CMD) & chip_cmd_bits::CMD_RESET) == 0 {
                break;
            }
            sleep(1_000);
        }
        if bar.readb(Regs::CHIP_CMD) & chip_cmd_bits::CMD_RESET as u8 != 0 {
            return Err(InitError::SoftwareResetStuck);
        }

        let rx_buf_dma_handle =
            CoherentAllocation::alloc_coherent(pdev.as_ref(), 8 * 1024 + 16 + 1536, GFP_KERNEL)
                .map_err(|_| InitError::BarRevoked)?;
        let tx_buf_dma_handle =
            CoherentAllocation::alloc_coherent(pdev.as_ref(), ALL_TX_BUF_LEN, GFP_KERNEL)
                .map_err(|_| InitError::BarRevoked)?;
        // let tx_skb_buf = [None; NUM_TX_BUFS];

        dev_info!(pdev.as_ref(), "init done!\n");
        Ok(Self {
            pdev,
            bar: bar_res,
            rx_buf_dma_handle,
            // tx_skb_buf,
            tx_buf_dma_handle,
            tx_buf_head: 0,
            tx_buf_tail: NUM_TX_BUFS - 1,
        })
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

        let handler = InterruptHandler {
            ndev: KBox::pin_init(new_mutex!(ndev), GFP_KERNEL)?,
        };

        use kernel::error::Error;

        Ok(KBox::pin_init(
            try_pin_init!(Rtl8139Driver {
                registration <- Registration::register(
                    pdev.irq(), // TODO : get irq from pdev
                    flags::SHARED,
                    c_str!("rust_rtl8139_driver"),
                    handler,
                ),
            }? Error),
            GFP_KERNEL,
        )
        .unwrap())
    }
}

kernel::module_pci_driver! {
    type: Rtl8139Driver,
    name: "rust_rtl8139_driver",
    author: "Team 05",
    description: "Rust RTL8139 driver",
    license: "MIT",
}
