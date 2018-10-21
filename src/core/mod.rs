#[macro_use] mod helpers;
mod hardware_address;
mod pack;
mod variant;
mod system;
mod message;
mod socket;

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

pub use self::socket::{Socket, Sendable};
pub use self::message::{MessageFlags, MessageMode, Attribute,
    Header, Message, DataMessage, ErrorMessage};
pub use self::hardware_address::HardwareAddress;
pub use self::pack::{NativeUnpack, NativePack, pack_vec};

/// A trait for converting a value from one type to another.
/// Any failure in converting will return None.
pub trait ConvertFrom<T: Sized>
    where Self: Sized
{
    fn convert_from(value: T) -> Option<Self>;
}
