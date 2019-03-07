use std::mem;
use std::str;
use std::ffi::{CStr, CString};

use errors::{Result, NetlinkError, NetlinkErrorKind};
use core::pack::{NativeUnpack, NativePack};
use core::hardware_address::HardwareAddress;
use core::message::{netlink_padding};

/// Parsing an array of nested attributes
///
/// Each chunk of attributes has a size and an index, the size is the size of
/// the chunk including the header
///
/// ```text
/// ---------------------------------------------------------------
/// | size | index | attributes ... | size | index | attributes ...
/// ---------------------------------------------------------------
///    u16    u16    u8 * (size - 4)
/// ```
pub fn nested_attribute_array(data: &[u8]) -> Vec<Vec<Attribute>>
{
    let vs = mem::size_of::<u16>();
    let mut attrs = vec![];
    let mut d = &data[..];
    while d.len() > (vs * 2) {
        let size = u16::unpack(&d).unwrap();
        let _index = u16::unpack(&d[vs..]).unwrap();
        if d.len() > size as usize {
            let (_, attributes) = Attribute::unpack_all(
                &d[(vs * 2)..size as usize]);
            attrs.push(attributes);
        }
        else {
            break;
        }
        d = &d[size as usize..];
    }
    attrs
}


/// Netlink attribute
///
/// ```text
/// | length | identifier |        data        | padding |
/// |--------|------------|--------------------|---------|
/// |   u16  |     u16    |  u8 * (length - 4) |         |
/// ```
/// 
/// The data is 4 byte aligned.
#[derive(Clone)]
pub struct Attribute {
    /// Attribute identifier
    pub identifier: u16,
    /// Attribute data
    data: Vec<u8>,
}

impl Attribute {
    const HEADER_SIZE: usize = 4;

    /// Unpack all attributes in the byte slice
    pub fn unpack_all(data: &[u8]) -> (usize, Vec<Attribute>) {
        let mut pos = 0usize;
        let mut attrs = vec![];
        loop {
            match Attribute::unpack_with_size(&data[pos..]) {
                Ok(r) => { attrs.push(r.1); pos += r.0; },
                Err(_) => { break; },
            }
        }
        (pos, attrs)
    }

    /// Create a new string attribute with provided identifier
    pub fn new_bytes<ID: Into<u16>>(identifier: ID, value: &[u8]) -> Attribute
    {
        Attribute { identifier: identifier.into(), data: value.to_vec() }
    }

    /// Create a new string attribute with provided identifier
    pub fn new_string_with_nul<ID: Into<u16>>(identifier: ID, value: &str)
        -> Attribute {
        let c_string = CString::new(value).unwrap();
        Attribute { identifier: identifier.into(),
            data: c_string.into_bytes_with_nul() }
    }

    /// Create a new string attribute with provided identifier
    pub fn new_string<ID: Into<u16>>(identifier: ID, value: &str) -> Attribute
    {
        let string = CString::new(value).unwrap();
        Attribute { identifier: identifier.into(),
            data: string.into_bytes() }
    }

    /// Create a new attribute from a type that can be packed into a byte slice
    pub fn new<ID: Into<u16>, V: NativePack>(identifier: ID, value: V)
        -> Attribute
    {
        let mut data = vec![0u8; mem::size_of::<V>()];
        value.pack_unchecked(&mut data);
        Attribute { identifier: identifier.into(), data: data }
    }

    /// Get the length of the data
    pub fn len(&self) -> u16 {
        self.data.len() as u16
    }
    /// Get the length of the data and header
    pub fn total_len(&self) -> usize {
        self.data.len() + Attribute::HEADER_SIZE
    }
    /// Unpack the underlying data into a u8
    pub fn as_u8(&self) -> Result<u8> {
        u8::unpack(&self.data)
    }
    /// Unpack the underlying data into a u16
    pub fn as_u16(&self) -> Result<u16> {
        u16::unpack(&self.data)
    }
    /// Unpack the underlying data into a u32
    pub fn as_u32(&self) -> Result<u32> {
        u32::unpack(&self.data)
    }
    /// Unpack the underlying data into a u64
    pub fn as_u64(&self) -> Result<u64> {
        u64::unpack(&self.data)
    }
    /// Unpack the underlying data into a i8
    pub fn as_i8(&self) -> Result<i8> {
        i8::unpack(&self.data)
    }
    /// Unpack the underlying data into a i16
    pub fn as_i16(&self) -> Result<i16> {
        i16::unpack(&self.data)
    }
    /// Unpack the underlying data into a i32
    pub fn as_i32(&self) -> Result<i32> {
        i32::unpack(&self.data)
    }
    /// Unpack the underlying data into a i64
    pub fn as_i64(&self) -> Result<i64> {
        i64::unpack(&self.data)
    }
    /// Unpack the underlying data into a String
    pub fn as_string(&self) -> Result<String> {
        match CStr::from_bytes_with_nul(&self.data) {
            Ok(bytes) => {
                let s = bytes.to_str()?;
                Ok(String::from(s))
            },
            Err(_) => {
                let s = String::from_utf8(self.data.clone())?;
                Ok(s)
            }
        }
    }
    /// Unpack the underlying data into a HardwareAddress
    pub fn as_hardware_address(&self) -> Result<HardwareAddress> {
        HardwareAddress::unpack(&self.data)
    }
    /// Get a clone of the underlying data
    pub fn as_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl NativePack for Attribute {
    fn pack<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
        let length = self.total_len() as u16;
        let slice = length.pack(buffer)?;
        let slice = self.identifier.pack(slice)?;
        let slice = self.data.pack(slice)?;
        let padding = netlink_padding(self.data.len());
        Ok(&mut slice[padding..])
    }
    fn pack_unchecked(&self, buffer: &mut [u8]) {
        let length = self.total_len() as u16;
        length.pack_unchecked(buffer);
        self.identifier.pack_unchecked(&mut buffer[2..]);
        self.data.pack_unchecked(&mut buffer[4..]);
    }
}

impl NativeUnpack for Attribute {
    fn unpack_with_size(buffer: &[u8]) -> Result<(usize, Self)>
    {
        if buffer.len() < Attribute::HEADER_SIZE {
            return Err(NetlinkError::new(NetlinkErrorKind::NotEnoughData).into());
        }
        let length = u16::unpack_unchecked(buffer) as usize;
        let identifier = u16::unpack_unchecked(&buffer[2..]);

        let padding = netlink_padding(length);
        if buffer.len() < (length + padding) {
            return Err(NetlinkError::new(NetlinkErrorKind::NotEnoughData).into());
        }
        let attr_data = (&buffer[4..length]).to_vec();
        Ok((length + padding,
                Attribute { identifier: identifier, data: attr_data }))
    }
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        let length = u16::unpack_unchecked(buffer) as usize;
        let identifier = u16::unpack_unchecked(&buffer[2..]);
        let attr_data = (&buffer[4..length]).to_vec();
        Attribute { identifier: identifier, data: attr_data }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_attribute()
    {
        let data = [
            0x07, 0x00, // size
            0x00, 0x10, // identifier
            0x11, 0xaa, 0x55, // data
            0xee, // padding
            ];
        let (used, attr) = Attribute::unpack_with_size(&data).unwrap();
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
        let (used, attr) = Attribute::unpack_with_size(&data).unwrap();
        assert_eq!(used, 8);
        assert_eq!(attr.data.len(), 4usize);
        assert_eq!(attr.identifier, 0x1000u16);
        assert_eq!(attr.data[0], 0x11);
        assert_eq!(attr.data[1], 0xaa);
        assert_eq!(attr.data[2], 0x55);
        assert_eq!(attr.data[3], 0xee);
    }

    #[test]
    fn unpack_attributes()
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
        let (used, attrs) = Attribute::unpack_all(&data);
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

    #[test]
    fn pack_attribute()
    {
        let data = [
            0x07, 0x00, // size
            0x55, 0x81, // identifier
            0x11, 0xaa, 0x55, // data
            0x00, // padding
            ];
        let attr = Attribute {
            identifier: 0x8155,
            data: vec![0x11, 0xaa, 0x55],
        };
        let mut buffer = [0u8; 32];
        {
            let slice = attr.pack(&mut buffer).unwrap();
            assert_eq!(slice.len(), 24);
        }
        assert_eq!(&buffer[0..8], data);
    }
}