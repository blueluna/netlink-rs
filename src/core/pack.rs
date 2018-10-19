use std::mem;
use std::ptr;
use std::io::{Error, ErrorKind};

use byteorder::{ByteOrder, NativeEndian};

use core::hardware_address::HardwareAddress;
use ::errors::Result;

pub trait NativeUnpack: Sized {
    fn unpack(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < mem::size_of::<Self>() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        Ok(Self::unpack_unchecked(buffer))
    }
    fn unpack_unchecked(buffer: &[u8]) -> Self;
}

impl NativeUnpack for u8 {
    fn unpack_unchecked(buffer: &[u8]) -> Self {
        buffer[0]
    }
}
impl NativeUnpack for i8 {
    fn unpack_unchecked(buffer: &[u8]) -> Self {
        buffer[0] as i8
    }
}
impl NativeUnpack for u16 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_u16(buffer)
    }
}
impl NativeUnpack for i16 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_i16(buffer)
    }
}
impl NativeUnpack for u32 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_u32(buffer)
    }
}
impl NativeUnpack for i32 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_i32(buffer)
    }
}
impl NativeUnpack for u64 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_u64(buffer)
    }
}
impl NativeUnpack for i64 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_i64(buffer)
    }
}
impl NativeUnpack for f32 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_f32(buffer)
    }
}
impl NativeUnpack for f64 {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        NativeEndian::read_f64(buffer)
    }
}
impl NativeUnpack for HardwareAddress {
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        HardwareAddress::from(&buffer[0..6])
    }
}
impl NativeUnpack for Vec<u8> {
    fn unpack(buffer: &[u8]) -> Result<Self> {
        Ok(Self::unpack_unchecked(buffer))
    }
    fn unpack_unchecked(buffer: &[u8]) -> Self
    {
        buffer.to_vec()
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
impl NativePack for Vec<u8> {
    fn pack<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
        let size = self.len();
        if buffer.len() < size {
            return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
        }
        Self::pack_unchecked(&self, buffer);
        Ok(&mut buffer[size..])
    }
    fn pack_unchecked(&self, buffer: &mut [u8])
    {
        unsafe {
            ptr::copy_nonoverlapping(self.as_ptr(), buffer.as_mut_ptr(),
                self.len());
        }
    }
}

pub fn pack_vec<T: NativePack>(buffer: &mut [u8], v: &Vec<T>)
    -> Result<usize>
{
    let type_size = mem::size_of::<T>();
    let size = type_size * v.len();
    if buffer.len() < size {
        return Err(Error::new(ErrorKind::UnexpectedEof, "").into());
    }
    let mut pos = 0usize;
    for i in v {
        i.pack_unchecked(&mut buffer[pos..]);
        pos += type_size;
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use std::cmp;

    fn pack_unpack_test<T>(bytes: &[u8], value: T)
        where T: NativePack + NativeUnpack + fmt::Debug + cmp::PartialEq + Sized
    {
        let value_size = mem::size_of::<T>();
        assert_eq!(bytes.len(), value_size);
        assert_eq!(T::unpack(bytes).unwrap(), value);
        let mut buffer = vec![0u8; mem::size_of::<T>()];
        {
            let left = T::pack(&value, &mut buffer).unwrap();
            assert_eq!(left.len(), 0);
        }
        assert_eq!(buffer, bytes);
    }


    #[test]
    fn pack_unpack_u8() {
        pack_unpack_test(&[0x5a], 0x5au8);
    }

    #[test]
    fn pack_unpack_i8() {
        pack_unpack_test(&[0xa5], -91i8);
    }

    #[test]
    fn pack_unpack_u16() {
        pack_unpack_test(&[0x22, 0xaa], 0xaa22u16.to_le());
    }

    #[test]
    fn pack_unpack_i16() {
        pack_unpack_test(&[0x55, 0xaa], (-21931i16).to_le());
    }

    #[test]
    fn pack_unpack_u32() {
        pack_unpack_test(&[0x44, 0x33, 0x22, 0x11], 0x11223344u32.to_le());
    }

    #[test]
    fn pack_unpack_i32() {
        pack_unpack_test(&[0x11, 0x22, 0x33, 0xa4], (-1540152815i32).to_le());
    }

    #[test]
    fn pack_unpack_u64() {
        pack_unpack_test(&[0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11],
            0x1122334455667788u64.to_le());
    }

    #[test]
    fn pack_unpack_i64() {
        pack_unpack_test(&[0x11, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x88],
            (-8637284766759618799i64).to_le());
    }

    #[test]
    fn pack_unpack_hardware_address() {
        let bytes = vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        let hwa = HardwareAddress::from(bytes.as_slice());
        pack_unpack_test(bytes.as_slice(), hwa);
    }

    #[test]
    fn pack_unpack_any_vec() {
        let v = vec![1u16, 2u16];
        let mut buffer = vec![0u8; mem::size_of::<u16>() * v.len()];
        let size = pack_vec(&mut buffer, &v).unwrap();
        assert_eq!(size, 4usize);
        assert_eq!(buffer, &[0x01, 0x00, 0x02, 0x00]);

        let v = vec![1u32, 2u32];
        let mut buffer = vec![0u8; mem::size_of::<u32>() * v.len()];
        let size = pack_vec(&mut buffer, &v).unwrap();
        assert_eq!(size, 8usize);
        assert_eq!(buffer, &[0x01, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00]);
    }
}
