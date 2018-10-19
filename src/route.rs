use std::io::{Write, Error, ErrorKind};
use libc;

use errors::Result;
use core::{Sendable, Attribute, MessageFlags, NativeParse, NativeWrite,
    ConvertFrom};

/// Family Id?!?
/// From Linux kernel header
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

pub struct Message {
    pub family: u16,
    pub attributes: Vec<Attribute>,
}

impl Message {
    pub fn new<F: Into<u16>>(family: F) -> Message {
        return Message { family: family.into(), attributes: vec!(), };
    }
}

impl Sendable for Message {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let kind: u8 = libc::AF_PACKET as u8;
        kind.write(writer)?;
        for attr in self.attributes.iter() {
            attr.write(writer)?;
        }
        Ok(())
    }
    fn message_type(&self) -> u16 { self.family }

    fn query_flags(&self) -> MessageFlags {
        MessageFlags::REQUEST | MessageFlags::DUMP
    }
}

pub struct InterfaceInformationMessage {
    pub family: u8,
    pub kind: u16,
    pub index: i32,
    pub flags: u32,
    pub change: u32,
    pub attributes: Vec<Attribute>,
}

impl InterfaceInformationMessage {
    pub fn parse(data: &[u8]) -> Result<(usize, InterfaceInformationMessage)>
    {
        if data.len() < 16 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        let family = u8::parse_unchecked(&data[0..]);
        // reserved u8
        let kind = u16::parse_unchecked(&data[2..]);
        let index = i32::parse_unchecked(&data[4..]);
        let flags = u32::parse_unchecked(&data[8..]);
        let change = u32::parse_unchecked(&data[12..]);
        let (used, attributes) = Attribute::parse_all(&data[16..]);
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
        let _ = socket.receive_messages().unwrap();
    }
}