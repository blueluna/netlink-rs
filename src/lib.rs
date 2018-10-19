#![recursion_limit = "1024"]

extern crate libc;
extern crate byteorder;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate error_chain;

mod errors;
#[macro_use] mod core;
pub mod route;
pub mod generic;

pub use errors::{Error, Result};
pub use core::{HardwareAddress, Socket, Message, Attribute, Protocol,
    MessageMode, NativeRead, NativeWrite, NativeUnpack, ConvertFrom,
    DataMessage, ErrorMessage};
