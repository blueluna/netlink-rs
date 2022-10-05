//! Partial Rust implementation of the Linux kernel Netlink protocol
//!
//! Currently this crate is mostly used together with the nl80211-rs crate to
//! explore the Linux kernel Netlink interface for 802.11 devices.

mod errors;
#[macro_use]
pub mod core;
pub mod generic;
pub mod route;

pub use crate::core::nested_attribute_array;
pub use crate::core::{
    Attribute, ConvertFrom, HardwareAddress, Message, MessageMode, NativePack, NativeUnpack,
    Protocol, Socket,
};
pub use crate::errors::{Error, Result};
