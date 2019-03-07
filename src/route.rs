//! Netlink route messages

use libc;

use errors::{Result, NetlinkError, NetlinkErrorKind};
use core::{SendMessage, Attribute, MessageFlags, NativeUnpack, NativePack,
    pack_vec, ConvertFrom};

/// Netlinkt route command
extended_enum!(FamilyId, u16,
    NewLink => 16,
    DeleteLink => 17,
    GetLink => 18,
    SetLink => 19,
);

extended_enum_default!(AddressFamilyAttribute, u16,
    Unspecified => 0,
    Address => 1,
    Broadcast => 2,
    InterfaceName => 3,
    MTU => 4,
    Link => 5,
    QDisc => 6,
    Stats => 7,
    Cost => 8,
    Priority => 9,
    Master => 10,
    WirelessExtension => 11,
    ProtocolInformation => 12,
    TransmitQueueLength => 13,
    Map => 14,
    Weight => 15,
    OperationState => 16,
    LinkMode => 17,
    LinkInfo => 18,
    NetworkNameSpacePid => 19,
    InterfaceAlias => 20,
    NumberVf => 21,
    VfInfoList => 22,
    Stats64 => 23,
    VfPorts => 24,
    PortSelf => 25,
    AfSpecification => 26,
    Group => 27,
    NetworkNameSpaceFileDescriptor => 28,
    ExtendedMask => 29,
    PromiscuityCount => 30,
    TransmitQueueCount => 31,
    ReceiveQueueCount => 32,
    Carrier => 33,
    PhysPortId => 34,
    CarrierChanges => 35,
    PhysSwitchId => 36,
    LinkNetworkNameSpaceId => 37,
    PhysPortName => 38,
    ProtocolDown => 39,
    GsoMaximumSegs => 40,
    GsoMaximumSize => 41,
    Padding => 42,
    Xdp => 43,
    Event => 44,
    NewNetworkNameSpaceId => 45,
    InterfaceNetworkNameSpaceId => 46,
);

/// Netlink message for route messages
pub struct Message {
    /// The family
    pub family: u16,
    /// Message attributes
    pub attributes: Vec<Attribute>,
}

impl Message {
    /// Create a new message with the provided family
    pub fn new<F: Into<u16>>(family: F) -> Message {
        return Message { family: family.into(), attributes: vec!(), };
    }
}

impl SendMessage for Message {
    fn pack(&self, data: &mut [u8]) -> Result<usize>
    {
        let kind: u8 = libc::AF_PACKET as u8;
        let slice = kind.pack(data)?;
        let size = pack_vec(slice, &self.attributes)?;
        Ok(size + 1)
    }
    fn message_type(&self) -> u16 { self.family }
    fn query_flags(&self) -> MessageFlags {
        MessageFlags::REQUEST | MessageFlags::DUMP
    }
}

/// Interface information message
/// 
/// Used to get information aabout a network interface
pub struct InterfaceInformationMessage {
    /// Message family
    pub family: u8,
    /// Message type
    pub kind: u16,
    /// Message index
    pub index: i32,
    /// Message flags
    pub flags: u32,
    /// Message change
    pub change: u32,
    /// Message attributes
    pub attributes: Vec<Attribute>,
}

impl InterfaceInformationMessage {
    /// Unpack byte slice into InterfaceInformationMessage
    pub fn unpack(data: &[u8]) -> Result<(usize, InterfaceInformationMessage)>
    {
        if data.len() < 16 {
            return Err(NetlinkError::new(NetlinkErrorKind::NotEnoughData)
                .into());
        }
        let family = u8::unpack_unchecked(&data[0..]);
        // reserved u8
        let kind = u16::unpack_unchecked(&data[2..]);
        let index = i32::unpack_unchecked(&data[4..]);
        let flags = u32::unpack_unchecked(&data[8..]);
        let change = u32::unpack_unchecked(&data[12..]);
        let (used, attributes) = Attribute::unpack_all(&data[16..]);
        Ok((used + 16, InterfaceInformationMessage {
            family: family,
            kind: kind,
            index: index,
            flags: flags,
            change: change,
            attributes: attributes,
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{Socket, Protocol};

    #[test]
    fn route_get_link() {
        let mut socket = Socket::new(Protocol::Route).unwrap();
        let msg = Message::new(FamilyId::GetLink);
        socket.send_message(&msg).unwrap();
        for m in socket.receive_messages().unwrap() {
            assert_eq!(m.header.identifier, FamilyId::NewLink);
        }
    }
}