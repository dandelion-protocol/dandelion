use alloc::vec::Vec;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::*;

pub trait BaseSerializable: Sized {
    fn wire_write(&self, buffer: &mut impl BufMut);
    fn wire_read(buffer: &mut impl Buf) -> Result<Self>;
    fn wire_skip(buffer: &mut impl Buf) -> Result<()>;

    fn wire_skip_many(buffer: &mut impl Buf, count: usize) -> Result<()> {
        util::generic_skip_many::<Self>(buffer, count)
    }
}

pub trait FixedSizeSerializable: BaseSerializable {
    const WIRE_SIZE: usize;

    fn wire_skip_many(buffer: &mut impl Buf, count: usize) -> Result<()> {
        if let Some(len) = Self::WIRE_SIZE.checked_mul(count) {
            util::generic_skip(buffer, len)
        } else {
            util::generic_skip_many::<Self>(buffer, count)
        }
    }
}

pub trait Serializable: BaseSerializable {
    fn wire_size(&self) -> usize;
}

impl<T: FixedSizeSerializable> Serializable for T {
    fn wire_size(&self) -> usize {
        T::WIRE_SIZE
    }
}

impl BaseSerializable for BytesMut {
    fn wire_write(&self, buffer: &mut impl BufMut) {
        util::varlen_write(buffer, self.as_ref());
    }
    fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
        util::varlen_read(buffer)
    }
    fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
        util::varlen_skip(buffer)?;
        Ok(())
    }
}

impl Serializable for BytesMut {
    fn wire_size(&self) -> usize {
        util::varlen_wire_size(self.len())
    }
}

impl BaseSerializable for Bytes {
    fn wire_write(&self, buffer: &mut impl BufMut) {
        util::varlen_write(buffer, self.as_ref());
    }
    fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
        Ok(Bytes::from(util::varlen_read(buffer)?))
    }
    fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
        util::varlen_skip(buffer)?;
        Ok(())
    }
}

impl Serializable for Bytes {
    fn wire_size(&self) -> usize {
        util::varlen_wire_size(self.len())
    }
}

impl<T: Serializable> BaseSerializable for Vec<T> {
    fn wire_write(&self, buffer: &mut impl BufMut) {
        self.len().wire_write(buffer);
        for item in self {
            item.wire_write(buffer);
        }
    }
    fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
        let count = usize::wire_read(buffer)?;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(T::wire_read(buffer)?);
        }
        Ok(vec)
    }
    fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
        let count = usize::wire_read(buffer)?;
        T::wire_skip_many(buffer, count)
    }
}

impl<T: Serializable> Serializable for Vec<T> {
    fn wire_size(&self) -> usize {
        let mut sum = usize::WIRE_SIZE;
        for item in self {
            sum = sum.strict_add(item.wire_size());
        }
        sum
    }
}

impl<const N: usize> BaseSerializable for [u8; N] {
    fn wire_write(&self, buffer: &mut impl BufMut) {
        util::fixed_write::<N>(buffer, self)
    }
    fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
        util::fixed_read::<N>(buffer)
    }
    fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
        util::fixed_skip::<N>(buffer)
    }
}

impl<const N: usize> FixedSizeSerializable for [u8; N] {
    const WIRE_SIZE: usize = N;
}

macro_rules! numtype {
    ( $ty:ty, $size:expr, $put:ident, $get:ident ) => {
        impl BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut impl BufMut) {
                buffer.$put(*self);
            }
            fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
                if buffer.remaining() < Self::WIRE_SIZE {
                    return Err(Error);
                }
                Ok(buffer.$get())
            }
            fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
                const WIRE_SIZE: usize = $size;
                util::fixed_skip::<WIRE_SIZE>(buffer)
            }
        }
        impl FixedSizeSerializable for $ty {
            const WIRE_SIZE: usize = $size;
        }
    };
}

numtype!(i8, 1, put_i8, get_i8);
numtype!(i16, 2, put_i16, get_i16);
numtype!(i32, 4, put_i32, get_i32);
numtype!(i64, 8, put_i64, get_i64);
numtype!(i128, 16, put_i128, get_i128);

numtype!(u8, 1, put_u8, get_u8);
numtype!(u16, 2, put_u16, get_u16);
numtype!(u32, 4, put_u32, get_u32);
numtype!(u64, 8, put_u64, get_u64);
numtype!(u128, 16, put_u128, get_u128);

numtype!(f32, 4, put_f32, get_f32);
numtype!(f64, 8, put_f64, get_f64);

macro_rules! thunktype {
    ( $ty:ty => $repr:ty, $to_repr:path, $from_repr:path ) => {
        impl BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut impl BufMut) {
                $to_repr(*self).wire_write(buffer);
            }
            fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
                Ok($from_repr(<$repr>::wire_read(buffer)?)?)
            }
            fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
                <$repr>::wire_skip(buffer)
            }
        }
        impl FixedSizeSerializable for $ty {
            const WIRE_SIZE: usize = <$repr>::WIRE_SIZE;
        }
    };
}

thunktype!(bool => u8, bool_to_u8, u8_to_bool);
thunktype!(usize => u32, usize_to_u32, u32_to_usize);

fn bool_to_u8(value: bool) -> u8 {
    if value {
        1
    } else {
        0
    }
}

fn u8_to_bool(value: u8) -> Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Error),
    }
}

fn usize_to_u32(value: usize) -> u32 {
    value.try_into().unwrap()
}

fn u32_to_usize(value: u32) -> Result<usize> {
    Ok(value.try_into()?)
}
