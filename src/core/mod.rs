//! Netlink core parts

#[macro_use]
mod helpers;
mod attribute;
mod hardware_address;
mod message;
mod pack;
mod socket;
mod system;

extended_enum!(Protocol, i32,
    Route => 0,
    Unused => 1,
    Usersock => 2,
    Firewall => 3,
    SockDiag => 4,
    Nflog => 5,
    Xfrm => 6,
    SELinux => 7,
    ISCSI => 8,
    Audit => 9,
    FibLookup => 10,
    Connector => 11,
    Netfilter => 12,
    IP6Fw => 13,
    DNRtMsg => 14,
    KObjectUevent => 15,
    Generic => 16,
    SCSITransport => 17,
    ECryptFs => 18,
    RDMA => 19,
    Crypto => 20,
    SMC => 21
);

pub use self::attribute::{nested_attribute_array, Attribute};
pub use self::hardware_address::HardwareAddress;
pub use self::message::{Header, Message, MessageFlags, MessageMode};
pub use self::pack::{pack_vec, NativePack, NativeUnpack};
pub use self::socket::{SendMessage, Socket};

/// A trait for converting a value from one type to another.
/// Any failure in converting will return None.
pub trait ConvertFrom<T: Sized>
where
    Self: Sized,
{
    /// Convert value from one type to the other, returning None if conversion failed
    fn convert_from(value: T) -> Option<Self>;
}
