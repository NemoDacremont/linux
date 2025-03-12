// SPDX-License-Identifier: GPL-2.0

//! Network device.
//!
//! C headers: [`include/linux/etherdevice.h`](../../../../include/linux/etherdevice.h),
//! [`include/linux/ethtool.h`](../../../../include/linux/ethtool.h),
//! [`include/linux/netdevice.h`](../../../../include/linux/netdevice.h),
//! [`include/linux/skbuff.h`](../../../../include/linux/skbuff.h),
//! [`include/uapi/linux/if_link.h`](../../../../include/uapi/linux/if_link.h).

use crate::{bindings, build_error, error::*, prelude::vtable, str::CStr, types::ForeignOwnable};
use {core::ffi::c_void, core::marker::PhantomData};

/// Flags associated with a [`Device`].
pub mod flags {
    /// Interface is up.
    pub const IFF_UP: u32 = bindings::net_device_flags_IFF_UP;
    /// Broadcast address valid.
    pub const IFF_BROADCAST: u32 = bindings::net_device_flags_IFF_BROADCAST;
    /// Device on debugging.
    pub const IFF_DEBUG: u32 = bindings::net_device_flags_IFF_DEBUG;
    /// Loopback device.
    pub const IFF_LOOPBACK: u32 = bindings::net_device_flags_IFF_LOOPBACK;
    /// Has p-p link.
    pub const IFF_POINTOPOINT: u32 = bindings::net_device_flags_IFF_POINTOPOINT;
    /// Avoids use of trailers.
    pub const IFF_NOTRAILERS: u32 = bindings::net_device_flags_IFF_NOTRAILERS;
    /// Interface RFC2863 OPER_UP.
    pub const IFF_RUNNING: u32 = bindings::net_device_flags_IFF_RUNNING;
    /// No ARP protocol.
    pub const IFF_NOARP: u32 = bindings::net_device_flags_IFF_NOARP;
    /// Receives all packets.
    pub const IFF_PROMISC: u32 = bindings::net_device_flags_IFF_PROMISC;
    /// Receive all multicast packets.
    pub const IFF_ALLMULTI: u32 = bindings::net_device_flags_IFF_ALLMULTI;
    /// Master of a load balancer.
    pub const IFF_MASTER: u32 = bindings::net_device_flags_IFF_MASTER;
    /// Slave of a load balancer.
    pub const IFF_SLAVE: u32 = bindings::net_device_flags_IFF_SLAVE;
    /// Supports multicast.
    pub const IFF_MULTICAST: u32 = bindings::net_device_flags_IFF_MULTICAST;
    /// Capable of setting media type.
    pub const IFF_PORTSEL: u32 = bindings::net_device_flags_IFF_PORTSEL;
    /// Auto media select active.
    pub const IFF_AUTOMEDIA: u32 = bindings::net_device_flags_IFF_AUTOMEDIA;
    /// Dialup device with changing addresses.
    pub const IFF_DYNAMIC: u32 = bindings::net_device_flags_IFF_DYNAMIC;
}

/// Private flags associated with a [`Device`].
pub mod priv_flags {
    /// 802.1Q VLAN device.
    pub const IFF_802_1Q_VLAN: u32 = bindings::netdev_priv_flags_IFF_802_1Q_VLAN;
    /// Ethernet bridging device.
    pub const IFF_EBRIDGE: u32 = bindings::netdev_priv_flags_IFF_EBRIDGE;
    /// Bonding master or slave device.
    pub const IFF_BONDING: u32 = bindings::netdev_priv_flags_IFF_BONDING;
    /// ISATAP interface (RFC4214).
    pub const IFF_ISATAP: u32 = bindings::netdev_priv_flags_IFF_ISATAP;
    /// WAN HDLC device.
    pub const IFF_WAN_HDLC: u32 = bindings::netdev_priv_flags_IFF_WAN_HDLC;
    /// dev_hard_start_xmit() is allowed to release skb->dst.
    pub const IFF_XMIT_DST_RELEASE: u32 = bindings::netdev_priv_flags_IFF_XMIT_DST_RELEASE;
    /// Disallows bridging this ether device.
    pub const IFF_DONT_BRIDGE: u32 = bindings::netdev_priv_flags_IFF_DONT_BRIDGE;
    /// Disables netpoll at run-time.
    pub const IFF_DISABLE_NETPOLL: u32 = bindings::netdev_priv_flags_IFF_DISABLE_NETPOLL;
    /// Device used as macvlan port.
    pub const IFF_MACVLAN_PORT: u32 = bindings::netdev_priv_flags_IFF_MACVLAN_PORT;
    /// Device used as bridge port.
    pub const IFF_BRIDGE_PORT: u32 = bindings::netdev_priv_flags_IFF_BRIDGE_PORT;
    /// Device used as Open vSwitch datapath port.
    pub const IFF_OVS_DATAPATH: u32 = bindings::netdev_priv_flags_IFF_OVS_DATAPATH;
    /// The interface supports sharing skbs on transmit.
    pub const IFF_TX_SKB_SHARING: u32 = bindings::netdev_priv_flags_IFF_TX_SKB_SHARING;
    /// Supports unicast filtering.
    pub const IFF_UNICAST_FLT: u32 = bindings::netdev_priv_flags_IFF_UNICAST_FLT;
    /// Device used as team port.
    pub const IFF_TEAM_PORT: u32 = bindings::netdev_priv_flags_IFF_TEAM_PORT;
    /// Device supports sending custom FCS.
    pub const IFF_SUPP_NOFCS: u32 = bindings::netdev_priv_flags_IFF_SUPP_NOFCS;
    /// Device supports hardware address change when it's running.
    pub const IFF_LIVE_ADDR_CHANGE: u32 = bindings::netdev_priv_flags_IFF_LIVE_ADDR_CHANGE;
    /// Macvlan device.
    pub const IFF_MACVLAN: u32 = bindings::netdev_priv_flags_IFF_MACVLAN;
    /// IFF_XMIT_DST_RELEASE not taking into account underlying stacked devices.
    pub const IFF_XMIT_DST_RELEASE_PERM: u32 =
        bindings::netdev_priv_flags_IFF_XMIT_DST_RELEASE_PERM;
    /// L3 master device.
    pub const IFF_L3MDEV_MASTER: u32 = bindings::netdev_priv_flags_IFF_L3MDEV_MASTER;
    /// Device can run without qdisc attached.
    pub const IFF_NO_QUEUE: u32 = bindings::netdev_priv_flags_IFF_NO_QUEUE;
    /// Device is a Open vSwitch master.
    pub const IFF_OPENVSWITCH: u32 = bindings::netdev_priv_flags_IFF_OPENVSWITCH;
    /// Device is enslaved to an L3 master.
    pub const IFF_L3MDEV_SLAVE: u32 = bindings::netdev_priv_flags_IFF_L3MDEV_SLAVE;
    /// Team device.
    pub const IFF_TEAM: u32 = bindings::netdev_priv_flags_IFF_TEAM;
    /// Device has had Rx Flow indirection table configured.
    pub const IFF_RXFH_CONFIGURED: u32 = bindings::netdev_priv_flags_IFF_RXFH_CONFIGURED;
    /// The headroom value is controlled by an external entity.
    pub const IFF_PHONY_HEADROOM: u32 = bindings::netdev_priv_flags_IFF_PHONY_HEADROOM;
    /// MACsec device.
    pub const IFF_MACSEC: u32 = bindings::netdev_priv_flags_IFF_MACSEC;
    /// Device doesn't support the rx_handler hook.
    pub const IFF_NO_RX_HANDLER: u32 = bindings::netdev_priv_flags_IFF_NO_RX_HANDLER;
    /// Failover master device.
    pub const IFF_FAILOVER: u32 = bindings::netdev_priv_flags_IFF_FAILOVER;
    /// Lower device of a failover master device.
    pub const IFF_FAILOVER_SLAVE: u32 = bindings::netdev_priv_flags_IFF_FAILOVER_SLAVE;
    /// Only invokes the rx handler of L3 master device.
    pub const IFF_L3MDEV_RX_HANDLER: u32 = bindings::netdev_priv_flags_IFF_L3MDEV_RX_HANDLER;
    /// Prevents ipv6 addrconf.
    pub const IFF_NO_ADDRCONF: u32 = bindings::netdev_priv_flags_IFF_NO_ADDRCONF;
    /// Capable of xmitting frames with skb_headlen(skb) == 0.
    pub const IFF_TX_SKB_NO_LINEAR: u32 = bindings::netdev_priv_flags_IFF_TX_SKB_NO_LINEAR;
    // /// Supports setting carrier via IFLA_PROTO_DOWN.
    // pub const IFF_CHANGE_PROTO_DOWN: u32 = bindings::netdev_priv_flags_IFF_CHANGE_PROTO_DOWN;
}

/// Corresponds to the kernel's `struct net_device_ops`.
///
/// A device driver must implement this. Only very basic operations are supported for now.
#[vtable]
pub trait DeviceOperations: ForeignOwnable + Send + Sync {
    /// Corresponds to `ndo_init` in `struct net_device_ops`.
    fn init(_dev: Device<Self>) -> Result {
        Ok(())
    }

    /// Corresponds to `ndo_uninit` in `struct net_device_ops`.
    fn uninit(_dev: Device<Self>) {}

    /// Corresponds to `ndo_open` in `struct net_device_ops`.
    fn open(_dev: Device<Self>) -> Result {
        Ok(())
    }

    /// Corresponds to `ndo_stop` in `struct net_device_ops`.
    fn stop(_dev: Device<Self>) -> Result {
        Ok(())
    }

    /// Corresponds to `ndo_start_xmit` in `struct net_device_ops`.
    fn start_xmit(_dev: Device<Self>, _skb: SkBuff) -> TxCode {
        TxCode::Busy
    }
}

/// Corresponds to the kernel's `struct net_device`.
///
/// # Invariants
///
/// The `ptr` points to the contiguous memory for `struct net_device` and a pointer,
/// which stores an address returned by `ForeignOwnable::into_foreign()`.
pub struct Device<T: DeviceOperations> {
    ptr: *mut bindings::net_device,
    is_registered: bool,
    _p: PhantomData<T>,
}

impl<T: DeviceOperations> Device<T> {
    /// Creates a new [`Device`] instance.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` must point to the contiguous memory
    /// for `struct net_device` and a pointer, which stores an address returned
    /// by `ForeignOwnable::into_foreign()`.
    unsafe fn from_ptr(ptr: *mut bindings::net_device, is_registered: bool) -> Self {
        // INVARIANT: The safety requirements ensure the invariant.
        Self {
            ptr,
            is_registered,
            _p: PhantomData,
        }
    }

    /// Creates a new [`Device`] instance for ethernet device.
    ///
    /// A device driver can pass private data.
    pub fn try_new(tx_queue_size: u32, rx_queue_size: u32, data: T) -> Result<Self> {
        // SAFETY: Just an FFI call with no additional safety requirements.
        let ptr = unsafe {
            bindings::alloc_etherdev_mqs(
                core::mem::size_of::<*const c_void>() as i32,
                tx_queue_size,
                rx_queue_size,
            )
        };
        if ptr.is_null() {
            return Err(code::ENOMEM);
        }

        // SAFETY: It's safe to write an address returned pointer
        // from `netdev_priv()` because `alloc_etherdev_mqs()` allocates
        // contiguous memory for `struct net_device` and a pointer.
        unsafe {
            let priv_ptr = bindings::netdev_priv(ptr) as *mut *const c_void;
            core::ptr::write(priv_ptr, data.into_foreign());
        }

        // SAFETY: `ptr` points to contiguous memory for `struct net_device` and a pointer,
        // which stores an address returned by `ForeignOwnable::into_foreign()`.
        Ok(unsafe { Device::from_ptr(ptr, false) })
    }

    /// Gets the private data of a device driver.
    pub fn drv_priv_data(&self) -> T::Borrowed<'_> {
        // SAFETY: The type invariants guarantee that self.ptr is valid and
        // bindings::netdev_priv(self.ptr) returns a pointer that stores an address
        // returned by `ForeignOwnable::into_foreign()`.
        unsafe {
            T::borrow(core::ptr::read(
                bindings::netdev_priv(self.ptr) as *const *mut c_void
            ))
        }
    }

    /// Sets the name of a device.
    pub fn set_name(&mut self, name: &CStr) -> Result {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe {
            if name.len() > (*self.ptr).name.len() {
                return Err(code::EINVAL);
            }
            (*self.ptr)
                .name
                .as_mut_ptr()
                .copy_from_nonoverlapping(name.as_char_ptr(), name.len());
        }
        Ok(())
    }

    /// Sets carrier.
    pub fn netif_carrier_on(&mut self) {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe { bindings::netif_carrier_on(self.ptr) }
    }

    /// Clears carrier.
    pub fn netif_carrier_off(&mut self) {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe { bindings::netif_carrier_off(self.ptr) }
    }

    /// Sets the max mtu of the device.
    pub fn set_max_mtu(&mut self, max_mtu: u32) {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe {
            (*self.ptr).max_mtu = max_mtu;
        }
    }

    /// Sets the minimum mtu of the device.
    pub fn set_min_mtu(&mut self, min_mtu: u32) {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe {
            (*self.ptr).min_mtu = min_mtu;
        }
    }

    /// Returns the flags of the device.
    pub fn get_flags(&self) -> u32 {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe { (*self.ptr).flags }
    }

    /// Sets the flags of the device.
    pub fn set_flags(&mut self, flags: u32) {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe {
            (*self.ptr).flags = flags;
        }
    }

    // /// Returns the priv_flags of the device.
    // pub fn get_priv_flags(&self) -> u32 {
    //     // SAFETY: The type invariants guarantee that `self.ptr` is valid.
    //     unsafe { (*self.ptr).priv_flags }
    // }

    // /// Sets the priv_flags of the device.
    // pub fn set_priv_flags(&mut self, flags: u32) {
    //     // SAFETY: The type invariants guarantee that `self.ptr` is valid.
    //     unsafe { (*self.ptr).priv_flags = flags }
    // }

    /// Generate a random Ethernet address (MAC) to be used by a net device
    /// and set addr_assign_type.
    pub fn set_random_eth_hw_addr(&mut self) {
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        unsafe { bindings::eth_hw_addr_random(self.ptr) }
    }

    /// Registers a network device.
    pub fn register(&mut self) -> Result {
        if self.is_registered {
            return Err(code::EINVAL);
        }
        // SAFETY: The type invariants guarantee that `self.ptr` is valid.
        let ret = unsafe {
            (*self.ptr).netdev_ops = &Self::DEVICE_OPS;
            bindings::register_netdev(self.ptr)
        };
        if ret != 0 {
            Err(Error::from_errno(ret))
        } else {
            self.is_registered = true;
            Ok(())
        }
    }

    const DEVICE_OPS: bindings::net_device_ops = bindings::net_device_ops {
        ndo_init: if <T>::HAS_INIT {
            Some(Self::init_callback)
        } else {
            None
        },
        ndo_uninit: if <T>::HAS_UNINIT {
            Some(Self::uninit_callback)
        } else {
            None
        },
        ndo_open: if <T>::HAS_OPEN {
            Some(Self::open_callback)
        } else {
            None
        },
        ndo_stop: if <T>::HAS_STOP {
            Some(Self::stop_callback)
        } else {
            None
        },
        ndo_start_xmit: if <T>::HAS_START_XMIT {
            Some(Self::start_xmit_callback)
        } else {
            None
        },
        // SAFETY: The rest is zeroed out to initialize `struct net_device_ops`,
        // set `Option<&F>` to be `None`.
        ..unsafe { core::mem::MaybeUninit::<bindings::net_device_ops>::zeroed().assume_init() }
    };

    unsafe extern "C" fn init_callback(netdev: *mut bindings::net_device) -> core::ffi::c_int {
        from_result(|| {
            // SAFETY: The C API guarantees that `netdev` is valid while this function is running.
            let dev = unsafe { Device::from_ptr(netdev, true) };
            T::init(dev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn uninit_callback(netdev: *mut bindings::net_device) {
        // SAFETY: The C API guarantees that `netdev` is valid while this function is running.
        let dev = unsafe { Device::from_ptr(netdev, true) };
        T::uninit(dev);
    }

    unsafe extern "C" fn open_callback(netdev: *mut bindings::net_device) -> core::ffi::c_int {
        from_result(|| {
            // SAFETY: The C API guarantees that `netdev` is valid while this function is running.
            let dev = unsafe { Device::from_ptr(netdev, true) };
            T::open(dev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn stop_callback(netdev: *mut bindings::net_device) -> core::ffi::c_int {
        from_result(|| {
            // SAFETY: The C API guarantees that `netdev` is valid while this function is running.
            let dev = unsafe { Device::from_ptr(netdev, true) };
            T::stop(dev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn start_xmit_callback(
        skb: *mut bindings::sk_buff,
        netdev: *mut bindings::net_device,
    ) -> bindings::netdev_tx_t {
        // SAFETY: The C API guarantees that `netdev` is valid while this function is running.
        let dev = unsafe { Device::from_ptr(netdev, true) };
        // SAFETY: The C API guarantees that `skb` is valid until a driver releases the skb.
        let skb = unsafe { SkBuff::from_ptr(skb) };
        T::start_xmit(dev, skb) as bindings::netdev_tx_t
    }
}

impl<T: DeviceOperations> Drop for Device<T> {
    fn drop(&mut self) {
        // SAFETY: The type invariants of `Device` guarantee that `self.ptr` is valid and
        // bindings::netdev_priv(self.ptr) returns a pointer that stores an address
        // returned by `ForeignOwnable::into_foreign()`.
        unsafe {
            let _ = T::from_foreign(core::ptr::read(
                bindings::netdev_priv(self.ptr) as *const *mut c_void
            ));
        }
        // SAFETY: The type invariants of `Device` guarantee that `self.ptr` is valid.
        unsafe {
            if self.is_registered {
                bindings::unregister_netdev(self.ptr);
            }
            bindings::free_netdev(self.ptr);
        }
    }
}

// SAFETY: `Device` is just a wrapper for the kernel`s `struct net_device`, which can be used
// from any thread. `struct net_device` stores a pointer to an object, which is `Sync`
// so it's safe to sharing its pointer.
unsafe impl<T: DeviceOperations> Send for Device<T> {}
// SAFETY: `Device` is just a wrapper for the kernel`s `struct net_device`, which can be used
// from any thread. `struct net_device` stores a pointer to an object, which is `Sync`,
// can be used from any thread too.
unsafe impl<T: DeviceOperations> Sync for Device<T> {}

/// Corresponds to the kernel's `enum netdev_tx`.
#[repr(i32)]
pub enum TxCode {
    /// Driver took care of packet.
    Ok = bindings::netdev_tx_NETDEV_TX_OK,
    /// Driver tx path was busy.
    Busy = bindings::netdev_tx_NETDEV_TX_BUSY,
}

/// Corresponds to the kernel's `struct sk_buff`.
///
/// A driver manages `struct sk_buff` in two ways. In both ways, the ownership is transferred
/// between C and Rust. The allocation and release are done asymmetrically.
///
/// On the tx side (`ndo_start_xmit` operation in `struct net_device_ops`), the kernel allocates
/// a `sk_buff' object and passes it to the driver. The driver is responsible for the release
/// after transmission.
/// On the rx side, the driver allocates a `sk_buff` object then passes it to the kernel
/// after receiving data.
///
/// A driver must explicitly call a function to drop a `sk_buff` object.
/// The code to let a `SkBuff` object go out of scope can't be compiled.
///
/// # Invariants
///
/// The pointer is valid.
pub struct SkBuff(*mut bindings::sk_buff);

impl SkBuff {
    /// Creates a new [`SkBuff`] instance.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` must be valid.
    unsafe fn from_ptr(ptr: *mut bindings::sk_buff) -> Self {
        // INVARIANT: The safety requirements ensure the invariant.
        Self(ptr)
    }

    /// Provides a time stamp.
    pub fn tx_timestamp(&mut self) {
        // SAFETY: The type invariants guarantee that `self.0` is valid.
        unsafe {
            bindings::skb_tx_timestamp(self.0);
        }
    }

    // /// Consumes a [`sk_buff`] object.
    // pub fn consume(self) {
    //     // SAFETY: The type invariants guarantee that `self.0` is valid.
    //     unsafe {
    //         bindings::kfree_skb_reason(self.0, bindings::skb_drop_reason_SKB_CONSUMED);
    //     }
    //     core::mem::forget(self);
    // }
}

impl Drop for SkBuff {
    #[inline(always)]
    fn drop(&mut self) {
        build_error!("skb must be released explicitly");
    }
}
