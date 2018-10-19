use std::mem;
use std::ptr;
use std::io::{Read, Write, Error, ErrorKind};
use core::hardware_address::HardwareAddress;
use ::errors::Result;

use byteorder::{ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};

pub trait NativeRead: Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self>;
}

impl NativeRead for u8 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u8()?)
    }
}
impl NativeRead for i8 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i8()?)
    }
}
impl NativeRead for u16 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u16::<NativeEndian>()?)
    }
}
impl NativeRead for i16 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i16::<NativeEndian>()?)
    }
}
impl NativeRead for u32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u32::<NativeEndian>()?)
    }
}
impl NativeRead for i32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i32::<NativeEndian>()?)
    }
}
impl NativeRead for u64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u64::<NativeEndian>()?)
    }
}
impl NativeRead for i64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i64::<NativeEndian>()?)
    }
}
impl NativeRead for f32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f32::<NativeEndian>()?)
    }
}
impl NativeRead for f64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f64::<NativeEndian>()?)
    }
}
impl NativeRead for HardwareAddress {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut data = vec![0u8; 6];
        reader.read_exact(&mut data)?;
        Ok(HardwareAddress::from(data.as_slice()))
    }
}

pub trait NativeWrite: Sized {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;
}

impl NativeWrite for u8 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(*self)?;
        Ok(())
    }
}
impl NativeWrite for i8 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i8(*self)?;
        Ok(())
    }
}
impl NativeWrite for u16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for i16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i16::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for u32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for i32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i32::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for u64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for i64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i64::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for f32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for f64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f64::<NativeEndian>(*self)?;
        Ok(())
    }
}
impl NativeWrite for HardwareAddress {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write(&self.bytes())?;
        Ok(())
    }
}

pub trait NativeParse: Sized {
    fn parse(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < mem::size_of::<Self>() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        Ok(Self::parse_unchecked(buffer))
    }
    fn parse_unchecked(buffer: &[u8]) -> Self;
}

impl NativeParse for u8 {
    fn parse_unchecked(buffer: &[u8]) -> Self {
        buffer[0]
    }
}
impl NativeParse for i8 {
    fn parse_unchecked(buffer: &[u8]) -> Self {
        buffer[0] as i8
    }
}
impl NativeParse for u16 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_u16(buffer)
    }
}
impl NativeParse for i16 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_i16(buffer)
    }
}
impl NativeParse for u32 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_u32(buffer)
    }
}
impl NativeParse for i32 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_i32(buffer)
    }
}
impl NativeParse for u64 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_u64(buffer)
    }
}
impl NativeParse for i64 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_i64(buffer)
    }
}
impl NativeParse for f32 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_f32(buffer)
    }
}
impl NativeParse for f64 {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_f64(buffer)
    }
}
impl NativeParse for HardwareAddress {
    fn parse_unchecked(buffer: &[u8]) -> Self
    {
        HardwareAddress::from(&buffer[0..6])
    }
}

pub trait NativePack: Sized {
    fn pack<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
        let type_size = mem::size_of::<Self>();
        if buffer.len() < type_size {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        Self::pack_unchecked(&self, buffer);
        Ok(&mut buffer[type_size..])
    }
    fn pack_unchecked(&self, buffer: &mut [u8]);
}

impl NativePack for u8 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        buffer[0] = *self;
    }
}
impl NativePack for i8 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        buffer[0] = *self as u8;
    }
}
impl NativePack for u16 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_u16(buffer, *self);
    }
}
impl NativePack for i16 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_i16(buffer, *self);
    }
}
impl NativePack for u32 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_u32(buffer, *self);
    }
}
impl NativePack for i32 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_i32(buffer, *self);
    }
}
impl NativePack for u64 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_u64(buffer, *self);
    }
}
impl NativePack for i64 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_i64(buffer, *self);
    }
}
impl NativePack for f32 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_f32(buffer, *self);
    }
}
impl NativePack for f64 {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        NativeEndian::write_f64(buffer, *self);
    }
}
impl NativePack for HardwareAddress {
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        unsafe {
            ptr::copy_nonoverlapping(self.as_ptr(), buffer.as_mut_ptr(), 6);
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io;
    use super::*;
    use std::fmt;
    use std::cmp;

    fn read_write_test<T>(bytes: &[u8], value: T) 
        where T: NativeRead + NativeWrite + fmt::Debug + cmp::PartialEq 
    {
        let value_size = mem::size_of::<T>();
        assert_eq!(bytes.len(), mem::size_of::<T>());
        let mut reader = io::Cursor::new(bytes);
        assert_eq!(T::read(&mut reader).unwrap(), value);
        let mut writer = io::Cursor::new(vec![0u8; value_size]);
        value.write(&mut writer).unwrap();
        assert_eq!(writer.into_inner(), Vec::from(bytes));
    }

    fn pack_unpack_test<T>(bytes: &[u8], value: T)
        where T: NativePack + NativeParse + fmt::Debug + cmp::PartialEq + Sized
    {
        let value_size = mem::size_of::<T>();
        assert_eq!(bytes.len(), value_size);
        assert_eq!(T::parse(bytes).unwrap(), value);
        let mut buffer = vec![0u8; mem::size_of::<T>()];
        {
            let left = T::pack(&value, &mut buffer).unwrap();
            assert_eq!(left.len(), 0);
        }
        assert_eq!(buffer, bytes);
    }

    #[test]
    fn read_write_u8() {
        read_write_test(&[0x5a], 0x5au8);
    }

    #[test]
    fn pack_unpack_u8() {
        pack_unpack_test(&[0x5a], 0x5au8);
    }

    #[test]
    fn read_write_i8() {
        read_write_test(&[0xa5], -91i8);
    }

    #[test]
    fn pack_unpack_i8() {
        pack_unpack_test(&[0xa5], -91i8);
    }

    #[test]
    fn read_write_u16() {
        read_write_test(&[0x22, 0xaa], 0xaa22u16.to_le());
    }

    #[test]
    fn pack_unpack_u16() {
        pack_unpack_test(&[0x22, 0xaa], 0xaa22u16.to_le());
    }

    #[test]
    fn read_write_i16() {
        read_write_test(&[0x55, 0xaa], (-21931i16).to_le());
    }

    #[test]
    fn pack_unpack_i16() {
        pack_unpack_test(&[0x55, 0xaa], (-21931i16).to_le());
    }

    #[test]
    fn read_write_u32() {
        read_write_test(&[0x44, 0x33, 0x22, 0x11], 0x11223344u32.to_le());
    }

    #[test]
    fn pack_unpack_u32() {
        pack_unpack_test(&[0x44, 0x33, 0x22, 0x11], 0x11223344u32.to_le());
    }

    #[test]
    fn read_write_i32() {
        read_write_test(&[0x11, 0x22, 0x33, 0xa4], (-1540152815i32).to_le());
    }

    #[test]
    fn pack_unpack_i32() {
        pack_unpack_test(&[0x11, 0x22, 0x33, 0xa4], (-1540152815i32).to_le());
    }

    #[test]
    fn read_write_u64() {
        read_write_test(&[0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11],
            0x1122334455667788u64.to_le());
    }

    #[test]
    fn pack_unpack_u64() {
        pack_unpack_test(&[0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11],
            0x1122334455667788u64.to_le());
    }

    #[test]
    fn read_write_i64() {
        read_write_test(&[0x11, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x88],
            (-8637284766759618799i64).to_le());
    }

    #[test]
    fn pack_unpack_i64() {
        pack_unpack_test(&[0x11, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x88],
            (-8637284766759618799i64).to_le());
    }

    #[test]
    fn read_write_hardware_address() {
        let bytes = vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        let hwa = HardwareAddress::from(bytes.as_slice());
        read_write_test(bytes.as_slice(), hwa);
    }

    #[test]
    fn pack_unpack_hardware_address() {
        let bytes = vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        let hwa = HardwareAddress::from(bytes.as_slice());
        pack_unpack_test(bytes.as_slice(), hwa);
    }
}
