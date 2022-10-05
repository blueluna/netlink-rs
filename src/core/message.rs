use errors::{NetlinkError, NetlinkErrorKind, Result};
use std::fmt;
use std::mem::size_of;

use core::pack::{NativePack, NativeUnpack};

bitflags! {
    /// Message flags
    pub struct MessageFlags: u16 {
        /// Request message
        const REQUEST     = 0x0001;
        /// Multo-part message
        const MULTIPART   = 0x0002;
        /// Acknowledge message
        const ACKNOWLEDGE = 0x0004;
        /// Dump message
        const DUMP        = 0x0100 | 0x0200;
    }
}

/// Message mode
///
/// Flags wich describes how the messages will be hadled
#[derive(PartialEq)]
pub enum MessageMode {
    /// No special flags
    None,
    /// Acknowledge message
    Acknowledge,
    /// Dump message
    Dump,
}

impl From<MessageFlags> for MessageMode {
    fn from(value: MessageFlags) -> MessageMode {
        if value.intersects(MessageFlags::DUMP) {
            MessageMode::Dump
        } else if value.intersects(MessageFlags::ACKNOWLEDGE) {
            MessageMode::Acknowledge
        } else {
            MessageMode::None
        }
    }
}

impl From<MessageMode> for MessageFlags {
    fn from(value: MessageMode) -> MessageFlags {
        let flags = MessageFlags::REQUEST;
        match value {
            MessageMode::None => flags,
            MessageMode::Acknowledge => flags | MessageFlags::ACKNOWLEDGE,
            MessageMode::Dump => flags | MessageFlags::DUMP,
        }
    }
}

#[inline]
pub(crate) fn align_to(len: usize, align_to: usize) -> usize {
    (len + align_to - 1) & !(align_to - 1)
}

#[inline]
pub(crate) fn netlink_align(len: usize) -> usize {
    align_to(len, 4usize)
}

#[inline]
pub(crate) fn netlink_padding(len: usize) -> usize {
    netlink_align(len) - len
}

/// Netlink message header
///
/// ```text
/// | length | identifier | flags | sequence | pid |
/// |--------|------------|-------|----------|-----|
/// |   u32  |     u16    |  u16  |   u32    | u32 |
/// ```
///
/// Length is the total length of the message in bytes, including the header.
/// Message data comes after the header. The data is 4 byte aligned, which
/// means that the actual length message length might be longer than indicated
/// by the length field.
///
#[repr(C)]
pub struct Header {
    /// Message length
    pub length: u32,
    /// Message identifier
    pub identifier: u16,
    /// Message flags
    pub flags: u16,
    /// Message sequence
    pub sequence: u32,
    /// Message process identifier
    pub pid: u32,
}

impl Header {
    const HEADER_SIZE: usize = 16;

    /// Returns the length including the header
    pub fn length(&self) -> usize {
        self.length as usize
    }

    /// Returns the length of the data section
    pub fn data_length(&self) -> usize {
        self.length() - size_of::<Header>()
    }

    /// Returns padding length in octets
    pub fn padding(&self) -> usize {
        netlink_padding(self.length())
    }

    /// Returns length including header and padding
    pub fn aligned_length(&self) -> usize {
        netlink_align(self.length())
    }

    /// Returns length of the data section header and padding
    pub fn aligned_data_length(&self) -> usize {
        netlink_align(self.data_length())
    }

    /// Check if the message pid equals provided pid or broadcast (0)
    pub fn check_pid(&self, pid: u32) -> bool {
        self.pid == 0 || self.pid == pid
    }

    /// Check if the message sequence number equals the  provided sequence
    /// number or broadcast (0)
    pub fn flags(&self) -> MessageFlags {
        MessageFlags::from_bits_truncate(self.flags)
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Length: {0:08x} {0}\nIdentifier: {1:04x}\nFlags: {2:04x}\n\
             Sequence: {3:08x} {3}\nPID: {4:08x} {4}",
            self.length, self.identifier, self.flags, self.sequence, self.pid,
        )
    }
}

impl NativePack for Header {
    fn pack_size(&self) -> usize {
        Self::HEADER_SIZE
    }
    fn pack_unchecked(&self, buffer: &mut [u8]) {
        self.length.pack_unchecked(buffer);
        self.identifier.pack_unchecked(&mut buffer[4..]);
        self.flags.pack_unchecked(&mut buffer[6..]);
        self.sequence.pack_unchecked(&mut buffer[8..]);
        self.pid.pack_unchecked(&mut buffer[12..]);
    }
}

impl NativeUnpack for Header {
    fn unpack_unchecked(buffer: &[u8]) -> Self {
        let length = u32::unpack_unchecked(&buffer[..]);
        let identifier = u16::unpack_unchecked(&buffer[4..]);
        let flags = u16::unpack_unchecked(&buffer[6..]);
        let sequence = u32::unpack_unchecked(&buffer[8..]);
        let pid = u32::unpack_unchecked(&buffer[12..]);
        Header {
            length: length,
            identifier: identifier,
            flags: flags,
            sequence: sequence,
            pid: pid,
        }
    }
}

/// Netlink error message
///
/// ```text
/// | header |  error code  | Original Header |
/// |--------|--------------|-----------------|
/// | Header |      i32     |     Header      |
/// ```
///
/// Header is the message header, See [Header](struct.Header.html).
/// The error code is an errno number reported by the kernel.
/// The original header is the header of the message that caused this error.
pub(crate) struct ErrorMessage {
    pub header: Header,
    pub code: i32,
    pub original_header: Header,
}

impl ErrorMessage {
    pub fn unpack(data: &[u8], header: Header) -> Result<(usize, ErrorMessage)> {
        let size = 4 + Header::HEADER_SIZE;
        if data.len() < size {
            return Err(NetlinkError::new(NetlinkErrorKind::NotEnoughData).into());
        }
        let code = i32::unpack_unchecked(data);
        let (_, original) = Header::unpack_with_size(&data[4..])?;
        Ok((
            size,
            ErrorMessage {
                header: header,
                code: code,
                original_header: original,
            },
        ))
    }
}

/// Netlink data message
///
/// ```text
/// | header |    data     | padding |
/// |--------|-------------|---------|
/// | Header | u8 * length |         |
/// ```
///
/// Header is the message header, See [Header](struct.Header.html).
/// The data is 4 byte aligned.
pub struct Message {
    /// Message header
    pub header: Header,
    /// Message data
    pub data: Vec<u8>,
}

impl Message {
    /// Unpack Message from byte slice and message header
    pub fn unpack(data: &[u8], header: Header) -> Result<(usize, Message)> {
        let size = header.data_length();
        let aligned_size = netlink_align(size);
        if data.len() < aligned_size {
            return Err(NetlinkError::new(NetlinkErrorKind::NotEnoughData).into());
        }
        Ok((
            aligned_size,
            Message {
                header: header,
                data: (&data[..size]).to_vec(),
            },
        ))
    }

    /// Pack data into byte slice
    pub fn pack<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
        let slice = self.header.pack(buffer)?;
        let slice = self.data.pack(slice)?;
        let padding = self.header.padding();
        Ok(&mut slice[padding..])
    }
}

pub type Messages = Vec<Message>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_header() {
        let data = [
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // sequence
            0x04, 0x00, 0x00,
        ]; // pid
        assert!(Header::unpack(&data).is_err());
        let data = [
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // sequence
            0x04, 0x00, 0x00, 0x00,
        ]; // pid
        let (used, header) = Header::unpack_with_size(&data).unwrap();
        assert_eq!(used, Header::HEADER_SIZE);
        assert_eq!(header.length, 18u32);
        assert_eq!(header.length(), 18usize);
        assert_eq!(header.data_length(), 2usize);
        assert_eq!(header.identifier, 0x1000u16);
        assert_eq!(header.flags, 0x0010u16);
        assert_eq!(header.sequence, 0x00000001u32);
        assert_eq!(header.pid, 0x00000004u32);
    }

    #[test]
    fn pack_header() {
        let header = Header {
            length: 18,
            identifier: 0x1000,
            flags: 0x0010,
            sequence: 1,
            pid: 4,
        };
        let mut buffer = [0u8; 32];
        {
            let slice = header.pack(&mut buffer).unwrap();
            assert_eq!(slice.len(), 16usize);
        }
        let data = [
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // sequence
            0x04, 0x00, 0x00, 0x00,
        ]; // pid
        assert_eq!(&buffer[..data.len()], data);
    }

    #[test]
    fn unpack_data_message() {
        let data = [
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // sequence
            0x04, 0x00, 0x00, 0x00, // pid
            0xaa, 0x55, 0x00, 0x00,
        ]; // data with padding
        let (used, header) = Header::unpack_with_size(&data).unwrap();
        assert_eq!(used, Header::HEADER_SIZE);
        assert_eq!(header.length, 18u32);
        assert_eq!(header.length(), 18usize);
        assert_eq!(header.data_length(), 2usize);
        assert_eq!(header.aligned_data_length(), 4usize);
        assert_eq!(header.identifier, 0x1000u16);
        assert_eq!(header.flags, 0x0010u16);
        assert_eq!(header.sequence, 0x00000001u32);
        assert_eq!(header.pid, 0x00000004u32);
        let (used, msg) = Message::unpack(&data[used..], header).unwrap();
        assert_eq!(used, 4usize);
        assert_eq!(msg.data.len(), 2usize);
        assert_eq!(msg.data[0], 0xaau8);
        assert_eq!(msg.data[1], 0x55u8);
    }

    #[test]
    fn pack_data_message() {
        let message = Message {
            header: Header {
                length: 18,
                identifier: 0x1000,
                flags: 0x0010,
                sequence: 0x12345678,
                pid: 1,
            },
            data: vec![0xaa, 0x55],
        };
        let mut buffer = [0xffu8; 32];
        {
            let slice = message.pack(&mut buffer).unwrap();
            assert_eq!(slice.len(), 12usize);
        }
        let data = [
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x78, 0x56, 0x34, 0x12, // sequence
            0x01, 0x00, 0x00, 0x00, // pid
            0xaa, 0x55, 0xff, 0xff,
        ]; // padded data
        assert_eq!(&buffer[..data.len()], data);
    }

    #[test]
    fn unpack_error_message() {
        let data = [
            0x24, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // sequence
            0x04, 0x00, 0x00, 0x00, // pid
            0xff, 0xff, 0xff, 0xff, // error code
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x11, // identifier
            0x11, 0x00, // flags
            0xff, 0xff, 0xff, 0xff, // sequence
            0x05, 0x00, 0x00, 0x00, // pid
        ];
        let (used, header) = Header::unpack_with_size(&data).unwrap();
        assert_eq!(used, Header::HEADER_SIZE);
        assert_eq!(header.length, 36u32);
        assert_eq!(header.length(), 36usize);
        assert_eq!(header.data_length(), 20usize);
        assert_eq!(header.aligned_data_length(), 20usize);
        assert_eq!(header.identifier, 0x1000u16);
        assert_eq!(header.flags, 0x0010u16);
        assert_eq!(header.sequence, 0x00000001u32);
        assert_eq!(header.pid, 0x00000004u32);
        let (used, msg) = ErrorMessage::unpack(&data[used..], header).unwrap();
        assert_eq!(used, 20usize);
        assert_eq!(msg.code, -1);
        assert_eq!(msg.original_header.length, 18u32);
        assert_eq!(msg.original_header.identifier, 0x1100u16);
        assert_eq!(msg.original_header.flags, 0x0011u16);
        assert_eq!(msg.original_header.sequence, u32::max_value());
        assert_eq!(msg.original_header.pid, 5u32);
    }
}
