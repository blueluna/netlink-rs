use errors::Result;
use std::fmt;
use std::str;
use std::mem::size_of;
use std::io;
use std::io::{Write, Error, ErrorKind};
use std::ffi::{CStr, CString};

use core::variant::{NativeWrite};
use core::pack::{NativeUnpack, NativePack};
use core::hardware_address::HardwareAddress;

bitflags! {
    pub struct MessageFlags: u16 {
        const REQUEST     = 0x0001;
        const MULTIPART   = 0x0002;
        const ACKNOWLEDGE = 0x0004;
        const DUMP        = 0x0100 | 0x0200;
    }
}

pub enum MessageMode {
    None,
    Acknowledge,
    Dump,
}

impl Into<MessageFlags> for MessageMode {
    fn into(self) -> MessageFlags {
        let flags = MessageFlags::REQUEST;
        match self {
            MessageMode::None => flags,
            MessageMode::Acknowledge => flags | MessageFlags::ACKNOWLEDGE,
            MessageMode::Dump => flags | MessageFlags::DUMP,
        }
    }
}

#[inline]
pub(crate) fn align_to(len: usize, align_to: usize) -> usize
{
    (len + align_to - 1) & !(align_to - 1)
}

#[inline]
pub(crate) fn netlink_align(len: usize) -> usize
{
    align_to(len, 4usize)
}

#[inline]
pub(crate) fn netlink_padding(len: usize) -> usize
{
    netlink_align(len) - len
}

/// Netlink message header
#[repr(C)]
pub struct Header {
    pub length: u32,
    pub identifier: u16,
    pub flags: u16,
    pub sequence: u32,
    pub pid: u32,
}

impl Header {
    const HEADER_SIZE: usize = 16;

    pub fn parse(data: &[u8]) -> Result<(usize, Header)> {
        if data.len() < Header::HEADER_SIZE {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        let length = u32::unpack_unchecked(&data[..]);
        let identifier = u16::unpack_unchecked(&data[4..]);
        let flags = u16::unpack_unchecked(&data[6..]);
        let sequence = u32::unpack_unchecked(&data[8..]);
        let pid = u32::unpack_unchecked(&data[12..]);
        Ok((Header::HEADER_SIZE, Header {
            length: length,
            identifier: identifier,
            flags: flags,
            sequence: sequence,
            pid: pid,
            }))
    }

    pub fn length(&self) -> usize {
        self.length as usize
    }

    pub fn data_length(&self) -> usize {
        self.length() - size_of::<Header>()
    }

    pub fn padding(&self) -> usize {
        netlink_padding(self.length())
    }

    pub fn aligned_length(&self) -> usize {
        netlink_align(self.length())
    }

    pub fn aligned_data_length(&self) -> usize {
        netlink_align(self.data_length())
    }

    pub fn check_pid(&self, pid: u32) -> bool {
        self.pid == 0 || self.pid == pid
    }

    pub fn check_sequence(&self, sequence: u32) -> bool {
        self.pid == 0 || self.sequence == sequence
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "Length: {0:08x} {0}\nIdentifier: {1:04x}\nFlags: {2:04x}\n\
            Sequence: {3:08x} {3}\nPID: {4:08x} {4}",
            self.length,
            self.identifier,
            self.flags,
            self.sequence,
            self.pid,
        )
    }
}

pub struct DataMessage {
    pub header: Header,
    pub data: Vec<u8>,
}

/// Netlink data message
impl DataMessage {
    pub fn parse(data: &[u8], header: Header) -> Result<(usize, DataMessage)>
    {
        let size = header.data_length();
        let aligned_size = netlink_align(size);
        if data.len() < aligned_size {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        Ok((aligned_size,
            DataMessage { header: header, data: (&data[..size]).to_vec() }
        ))
    }
}

/// Netlink error message
pub struct ErrorMessage {
    pub header: Header,
    pub code: i32,
    pub original_header: Header,
}

impl ErrorMessage {
    pub fn parse(data: &[u8], header: Header) -> Result<(usize, ErrorMessage)>
    {
        let size = 4 + Header::HEADER_SIZE;
        if data.len() < size {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        let code = i32::unpack_unchecked(data);
        let (_, original) = Header::parse(&data[4..])?;
        Ok((size,
            ErrorMessage { header: header, code: code,
                original_header: original }))
    }
}

pub enum Message {
    Data(DataMessage),
    Acknowledge,
    Done,
}

/// Netlink attribute
///
/// Consists of a 2 octet length, an 2 octet identifier and the data.
/// The data is aligned to 4 octets.
#[derive(Clone)]
pub struct Attribute {
    pub identifier: u16,
    data: Vec<u8>,
}

impl Attribute {
    const HEADER_SIZE: usize = 4;

    pub fn parse(data: &[u8]) -> Result<(usize, Attribute)> {
        if data.len() < Attribute::HEADER_SIZE {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        let length = u16::unpack_unchecked(data) as usize;
        let identifier = u16::unpack_unchecked(&data[2..]);

        let padding = netlink_padding(length);
        if data.len() < (length + padding) {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        let attr_data = (&data[4..length]).to_vec();
        Ok((length + padding,
                Attribute { identifier: identifier, data: attr_data }))
    }

    pub fn parse_all(data: &[u8]) -> (usize, Vec<Attribute>)
    {
        let mut pos = 0usize;
        let mut attrs = vec![];
        loop {
            match Attribute::parse(&data[pos..]) {
                Ok(r) => { attrs.push(r.1); pos += r.0; },
                Err(_) => { break; },
            }
        }
        (pos, attrs)
    }

    pub fn new_string<ID: Into<u16>>(identifier: ID, value: &str) -> Attribute
    {
        let c_string = CString::new(value).unwrap();
        Attribute { identifier: identifier.into(),
            data: c_string.into_bytes_with_nul() }
    }

    pub fn new<ID: Into<u16>, V: NativeWrite>(identifier: ID, value: V)
        -> Attribute
    {
        let mut writer = io::Cursor::new(Vec::new());
        value.write(&mut writer).unwrap();
        Attribute { identifier: identifier.into(), data: writer.into_inner() }
    }

    pub fn len(&self) -> u16 {
        self.data.len() as u16
    }
    pub fn total_len(&self) -> usize {
        netlink_align(self.data.len() + Attribute::HEADER_SIZE)
    }

    pub fn as_u8(&self) -> Result<u8> {
        u8::unpack(&self.data)
    }
    pub fn as_u16(&self) -> Result<u16> {
        u16::unpack(&self.data)
    }
    pub fn as_u32(&self) -> Result<u32> {
        u32::unpack(&self.data)
    }
    pub fn as_u64(&self) -> Result<u64> {
        u64::unpack(&self.data)
    }
    pub fn as_i8(&self) -> Result<i8> {
        i8::unpack(&self.data)
    }
    pub fn as_i16(&self) -> Result<i16> {
        i16::unpack(&self.data)
    }
    pub fn as_i32(&self) -> Result<i32> {
        i32::unpack(&self.data)
    }
    pub fn as_i64(&self) -> Result<i64> {
        i64::unpack(&self.data)
    }
    pub fn as_string(&self) -> Result<String> {
        match CStr::from_bytes_with_nul(&self.data) {
            Ok(bytes) => {
                let s = bytes.to_str()?;
                Ok(String::from(s))
            },
            Err(_) => {
                let s = str::from_utf8(&self.data)?;
                Ok(String::from(s))
            }
        }
    }
    pub fn as_hardware_address(&self) -> Result<HardwareAddress> {
        HardwareAddress::unpack(&self.data)
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let length = self.total_len() as u16;
        length.write(writer)?;
        self.identifier.write(writer)?;
        writer.write_all(&self.data)?;
        Ok(())
    }

    pub fn pack<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
        let length = self.total_len() as u16;
        let slice = length.pack(buffer)?;
        let slice = self.identifier.pack(slice)?;
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_header()
    {
        let data = [
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // sequence
            0x04, 0x00, 0x00, 0x00]; // pid
        let (used, header) = Header::parse(&data).unwrap();
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
    fn parse_data_message()
    {
        let data = [
            0x12, 0x00, 0x00, 0x00, // size
            0x00, 0x10, // identifier
            0x10, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // sequence
            0x04, 0x00, 0x00, 0x00, // pid
            0xaa, 0x55, 0x00, 0x00]; // data with padding
        let (used, header) = Header::parse(&data).unwrap();
        assert_eq!(used, Header::HEADER_SIZE);
        assert_eq!(header.length, 18u32);
        assert_eq!(header.length(), 18usize);
        assert_eq!(header.data_length(), 2usize);
        assert_eq!(header.aligned_data_length(), 4usize);
        assert_eq!(header.identifier, 0x1000u16);
        assert_eq!(header.flags, 0x0010u16);
        assert_eq!(header.sequence, 0x00000001u32);
        assert_eq!(header.pid, 0x00000004u32);
        let (used, msg) = DataMessage::parse(&data[used..], header).unwrap();
        assert_eq!(used, 4usize);
        assert_eq!(msg.data.len(), 2usize);
        assert_eq!(msg.data[0], 0xaau8);
        assert_eq!(msg.data[1], 0x55u8);
    }

    #[test]
    fn parse_error_message()
    {
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
        let (used, header) = Header::parse(&data).unwrap();
        assert_eq!(used, Header::HEADER_SIZE);
        assert_eq!(header.length, 36u32);
        assert_eq!(header.length(), 36usize);
        assert_eq!(header.data_length(), 20usize);
        assert_eq!(header.aligned_data_length(), 20usize);
        assert_eq!(header.identifier, 0x1000u16);
        assert_eq!(header.flags, 0x0010u16);
        assert_eq!(header.sequence, 0x00000001u32);
        assert_eq!(header.pid, 0x00000004u32);
        let (used, msg) = ErrorMessage::parse(&data[used..], header).unwrap();
        assert_eq!(used, 20usize);
        assert_eq!(msg.code, -1);
        assert_eq!(msg.original_header.length, 18u32);
        assert_eq!(msg.original_header.identifier, 0x1100u16);
        assert_eq!(msg.original_header.flags, 0x0011u16);
        assert_eq!(msg.original_header.sequence, u32::max_value());
        assert_eq!(msg.original_header.pid, 5u32);
    }

    #[test]
    fn parse_attribute()
    {
        let data = [
            0x07, 0x00, // size
            0x00, 0x10, // identifier
            0x11, 0xaa, 0x55, // data
            0xee, // padding
            ];
        let (used, attr) = Attribute::parse(&data).unwrap();
        assert_eq!(used, 8);
        assert_eq!(attr.data.len(), 3usize);
        assert_eq!(attr.identifier, 0x1000u16);
        assert_eq!(attr.data[0], 0x11);
        assert_eq!(attr.data[1], 0xaa);
        assert_eq!(attr.data[2], 0x55);

        let data = [
            0x08, 0x00, // size
            0x00, 0x10, // identifier
            0x11, 0xaa, 0x55, 0xee,// data
            ];
        let (used, attr) = Attribute::parse(&data).unwrap();
        assert_eq!(used, 8);
        assert_eq!(attr.data.len(), 4usize);
        assert_eq!(attr.identifier, 0x1000u16);
        assert_eq!(attr.data[0], 0x11);
        assert_eq!(attr.data[1], 0xaa);
        assert_eq!(attr.data[2], 0x55);
        assert_eq!(attr.data[3], 0xee);
    }

    #[test]
    fn parse_attributes()
    {
        let data = [
            0x07, 0x00, // size
            0x00, 0x10, // identifier
            0x11, 0xaa, 0x55, // data
            0xee, // padding
            0x08, 0x00, // size
            0x00, 0x10, // identifier
            0x11, 0xaa, 0x55, 0xee,// data
            ];
        let (used, attrs) = Attribute::parse_all(&data);
        assert_eq!(used, 16usize);
        assert_eq!(attrs.len(), 2usize);

        assert_eq!(attrs[0].data.len(), 3usize);
        assert_eq!(attrs[0].identifier, 0x1000u16);
        assert_eq!(attrs[0].data[0], 0x11);
        assert_eq!(attrs[0].data[1], 0xaa);
        assert_eq!(attrs[0].data[2], 0x55);

        assert_eq!(attrs[1].data.len(), 4usize);
        assert_eq!(attrs[1].identifier, 0x1000u16);
        assert_eq!(attrs[1].data[0], 0x11);
        assert_eq!(attrs[1].data[1], 0xaa);
        assert_eq!(attrs[1].data[2], 0x55);
        assert_eq!(attrs[1].data[3], 0xee);
    }
}