use std::fmt;
use std::io;

use std::convert::{From, Into};

use errors::Result;

use core;
use core::{Attribute, Sendable, MessageFlags, MessageMode, NativePack,
    ConvertFrom};

extended_enum!(FamilyId, u16,
    Control => 16,
    VirtualFileSystemDiskQuota => 17,
    Raid => 18,
);

extended_enum_default!(Command, u8,
    Unspecified => 0,
    NewFamily => 1,
    DelFamily => 2,
    GetFamily => 3,
    NewOps => 4,
    DelOps => 5,
    GetOps => 6,
    NewMulticastGroup => 7,
    DelMulticastGroup => 8,
    GetMulticastGroup => 9,
);

extended_enum_default!(AttributeId, u16,
    Unspecified => 0,
    FamilyId => 1,
    FamilyName => 2,
    Version => 3,
    HeaderSize => 4,
    MaximumAttributes => 5,
    Operations => 6,
    MulticastGroups => 7,
);

extended_enum_default!(OperationAttributeId, u16,
    Unspecified => 0,
    Id => 1,
    Flags => 2,
);

extended_enum_default!(MulticastAttributeId, u16,
    Unspecified => 0,
    Name => 1,
    Id => 2,
);

/// Netlink generic message
#[derive(Clone)]
pub struct Message {
    pub family: u16,
    pub command: u8,
    pub version: u8,
    pub flags: MessageFlags,
    pub attributes: Vec<Attribute>,
}

impl Message {
    /// Create a new message
    pub fn new<F: Into<u16>, C: Into<u8>, M: Into<MessageFlags>>
        (family: F, command: C, mode: M) -> Message {
        return Message {
            family: family.into(),
            command: command.into(),
            version: 1u8,
            flags: mode.into(),
            attributes: vec!(),
            };
    }

    /// unpack message from slice
    pub fn unpack(data: &[u8]) -> Result<(usize, Message)> {
        let command = data[0];
        let version = data[1];
        // skip reserved u16
        let (consumed, attributes) = core::Attribute::unpack_all(&data[4..]);
        Ok((consumed + 4usize,
            Message {
                family: 0xffff,
                command: command,
                version: version,
                flags: MessageFlags::from_bits_truncate(0),
                attributes: attributes,
            }))
    }

    /// Get the message family as u16
    pub fn family(&self) -> u16 { self.family.clone().into() }

    /// Set message flags
    pub fn set_flags(&mut self, flags: MessageFlags) { self.flags = flags; }

    // Append a attribute to the message
    pub fn append_attribute(&mut self, attr: Attribute)
    {
        self.attributes.push(attr);
    }
}

impl Sendable for Message {
    fn pack(&self, data: &mut [u8]) -> Result<usize>
    {
        let slice = self.command.pack(data)?;
        let slice = self.version.pack(slice)?;
        let slice = 0u16.pack(slice)?;
        let size = core::pack_vec(slice, &self.attributes)?;
        Ok(size + 4)
    }
    fn message_type(&self) -> u16 { self.family.clone().into() }
    fn query_flags(&self) -> MessageFlags { self.flags }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "Family: {} Command: {} Version: {} Flags: {:x} Attribute Count: {}",
            self.family, self.command, self.version, self.flags.bits(),
            self.attributes.len()
        )
    }
}

/// Netlink generic Multi-cast group
/// 
/// Maps a identifier with a name.
#[derive(Clone)]
pub struct MulticastGroup {
    pub id: u32,
    pub name: String,
}

impl MulticastGroup {
    fn from_bytes(bytes: &[u8]) -> Result<MulticastGroup>
    {
        let (_, attributes) = core::Attribute::unpack_all(bytes);
        let mut group_name = String::new();
        let mut group_id = None;
        for attribute in attributes {
            match MulticastAttributeId::from(attribute.identifier) {
                MulticastAttributeId::Unspecified => {}
                MulticastAttributeId::Id => {
                    group_id = attribute.as_u32().ok();
                }
                MulticastAttributeId::Name => {
                    group_name = attribute.as_string()?;
                }
            }
        }
        if let Some(id) = group_id {
            return Ok(MulticastGroup {
                id: id,
                name: group_name,
            });
        }
        Err(io::Error::new(io::ErrorKind::InvalidData, "").into())
    }
}

impl fmt::Display for MulticastGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Multicast Group: {} Name: {}", self.id, self.name)
    }
}

/// Netlink generic family
/// 
/// Contains identifier, name and multi-cast groups for a Netlink family.
#[derive(Clone)]
pub struct Family {
    pub id: u16,
    pub name: String,
    pub multicast_groups: Vec<MulticastGroup>,
}

impl Family {
    fn from_message(message: Message) -> Result<Family>
    {
        let mut family_name = String::new();
        let mut family_id = 0u16;
        let mut groups = vec![];
        for attr in message.attributes {
            match AttributeId::from(attr.identifier) {
                AttributeId::Unspecified => {}
                AttributeId::FamilyName => {
                    family_name = attr.as_string()?;
                }
                AttributeId::FamilyId => {
                    family_id = attr.as_u16()?;
                }
                AttributeId::MulticastGroups => {
                    let (_, mcs_attributes) = core::Attribute::unpack_all(
                        &attr.as_bytes());
                    for mcs_attr in mcs_attributes {
                        groups.push(MulticastGroup::from_bytes(
                            &mcs_attr.as_bytes())?);
                    }
                }
                _ => {}
            }
        }
        if family_id > 0 {
            return Ok(Family {
                id: family_id,
                name: family_name,
                multicast_groups: groups });
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Family Not Found").into())
    }

    pub fn from_name(socket: &mut core::Socket, name: &str)
        -> Result<Family>
    {
        {
            let mut tx_msg = Message::new(FamilyId::Control,
                Command::GetFamily, MessageMode::Acknowledge);
            tx_msg.attributes.push(
                Attribute::new_string(AttributeId::FamilyName, name));
            socket.send_message(&tx_msg)?;
        }
        loop {
            let messages = socket.receive_messages()?;
            if messages.is_empty() {
                break;
            }
            for message in messages {
                match message {
                    core::Message::Data(m) => {
                        if FamilyId::convert_from(m.header.identifier) ==
                            Some(FamilyId::Control) {
                            let (_, msg) = Message::unpack(&m.data)?;
                            let family = Family::from_message(msg)?;
                            if family.name == name {
                                return Ok(family);
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound,
            "Generic family not found").into())
    }

    pub fn from_id<ID: Into<u16>>(socket: &mut core::Socket, id: ID)
        -> Result<Family>
    {
        let id = id.into().clone();
        {
            let mut tx_msg = Message::new(FamilyId::Control,
                Command::GetFamily, MessageMode::Acknowledge);
            tx_msg.attributes.push(Attribute::new(AttributeId::FamilyId, id));
            socket.send_message(&tx_msg)?;
        }
        loop {
            let messages = socket.receive_messages()?;
            if messages.is_empty() {
                break;
            }
            for message in messages {
                match message {
                    core::Message::Data(m) => {
                        if FamilyId::convert_from(m.header.identifier) == Some(FamilyId::Control) {
                            let (_, msg) = Message::unpack(&m.data)?;
                            let family = Family::from_message(msg)?;
                            if family.id == id {
                                return Ok(family);
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Generic family not found").into())
    }

    pub fn all(socket: &mut core::Socket) -> Result<Vec<Family>>
    {
        {
            let tx_msg = Message::new(FamilyId::Control, Command::GetFamily,
                MessageMode::Dump);
            socket.send_message(&tx_msg)?;
        }
        let messages = socket.receive_messages()?;
        let mut families = vec![];
        for message in messages {
            match message {
                core::Message::Data(m) => {
                    if FamilyId::from(m.header.identifier) == FamilyId::Control {
                        let (_, msg) = Message::unpack(&m.data)?;
                        families.push(Family::from_message(msg)?);
                    }
                },
                core::Message::Acknowledge => (),
                core::Message::Done => { break; }
            }
        }
        return Ok(families)
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Generic Family: {} Name: {}", self.id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc;

    #[test]
    fn check_family_ids() {
        assert_eq!(u16::from(FamilyId::Control), libc::GENL_ID_CTRL as u16);
        assert_eq!(u16::from(FamilyId::VirtualFileSystemDiskQuota),
            libc::GENL_ID_VFS_DQUOT as u16);
        assert_eq!(u16::from(FamilyId::Raid), libc::GENL_ID_PMCRAID as u16);
    }

    #[test]
    fn check_commands() {
        assert_eq!(u8::from(Command::Unspecified),
            libc::CTRL_CMD_UNSPEC as u8);
        assert_eq!(u8::from(Command::NewFamily),
            libc::CTRL_CMD_NEWFAMILY as u8);
        assert_eq!(u8::from(Command::DelFamily),
            libc::CTRL_CMD_DELFAMILY as u8);
        assert_eq!(u8::from(Command::GetFamily),
            libc::CTRL_CMD_GETFAMILY as u8);
        assert_eq!(u8::from(Command::NewOps), libc::CTRL_CMD_NEWOPS as u8);
        assert_eq!(u8::from(Command::DelOps), libc::CTRL_CMD_DELOPS as u8);
        assert_eq!(u8::from(Command::GetOps), libc::CTRL_CMD_GETOPS as u8);
        assert_eq!(u8::from(Command::NewMulticastGroup),
            libc::CTRL_CMD_NEWMCAST_GRP as u8);
        assert_eq!(u8::from(Command::DelMulticastGroup),
            libc::CTRL_CMD_DELMCAST_GRP as u8);
        assert_eq!(u8::from(Command::GetMulticastGroup),
            libc::CTRL_CMD_GETMCAST_GRP as u8);
    }

    #[test]
    fn check_attributes() {
        assert_eq!(u16::from(AttributeId::Unspecified),
            libc::CTRL_ATTR_UNSPEC as u16);
        assert_eq!(u16::from(AttributeId::FamilyId),
            libc::CTRL_ATTR_FAMILY_ID as u16);
        assert_eq!(u16::from(AttributeId::FamilyName),
            libc::CTRL_ATTR_FAMILY_NAME as u16);
        assert_eq!(u16::from(AttributeId::Version),
            libc::CTRL_ATTR_VERSION as u16);
        assert_eq!(u16::from(AttributeId::HeaderSize),
            libc::CTRL_ATTR_HDRSIZE as u16);
        assert_eq!(u16::from(AttributeId::MaximumAttributes),
            libc::CTRL_ATTR_MAXATTR as u16);
        assert_eq!(u16::from(AttributeId::Operations),
            libc::CTRL_ATTR_OPS as u16);
        assert_eq!(u16::from(AttributeId::MulticastGroups),
            libc::CTRL_ATTR_MCAST_GROUPS as u16);
    }

    #[test]
    fn check_operation_attributes() {
        assert_eq!(u16::from(OperationAttributeId::Unspecified),
            libc::CTRL_ATTR_OP_UNSPEC as u16);
        assert_eq!(u16::from(OperationAttributeId::Id),
            libc::CTRL_ATTR_OP_ID as u16);
        assert_eq!(u16::from(OperationAttributeId::Flags),
            libc::CTRL_ATTR_OP_FLAGS as u16);
    }

    #[test]
    fn check_multicast_attributes() {
        assert_eq!(u16::from(MulticastAttributeId::Unspecified),
            libc::CTRL_ATTR_MCAST_GRP_UNSPEC as u16);
        assert_eq!(u16::from(MulticastAttributeId::Name),
            libc::CTRL_ATTR_MCAST_GRP_NAME as u16);
        assert_eq!(u16::from(MulticastAttributeId::Id),
            libc::CTRL_ATTR_MCAST_GRP_ID as u16);
    }
}