use super::{
    bytes::{Buf, BufMut, Bytes, BytesMut},
    BaseSerializable,
    Error,
    FixedSizeSerializable,
    Result,
    Serializable,
};

pub fn serialize(value: &impl Serializable) -> BytesMut {
    let mut buffer = BytesMut::with_capacity(value.wire_size());
    value.wire_write(&mut buffer);
    buffer
}

pub fn deserialize<T: Serializable>(mut buffer: Bytes) -> Result<T> {
    let value = T::wire_read(&mut buffer)?;
    if !buffer.is_empty() {
        return Err(Error);
    }
    Ok(value)
}

pub fn nested_wire_size(inner: &impl Serializable) -> usize {
    varlen_wire_size(inner.wire_size())
}

pub fn nested_write(buffer: &mut impl BufMut, inner: &impl Serializable) {
    let payload = serialize(inner);
    varlen_write(buffer, payload.as_ref());
}

pub fn nested_read<T: Serializable>(buffer: &mut impl Buf) -> Result<T> {
    let payload = varlen_read(buffer)?;
    deserialize::<T>(Bytes::from(payload))
}

pub const fn varlen_wire_size(len: usize) -> usize {
    usize::WIRE_SIZE.strict_add(len)
}

pub fn varlen_write(buffer: &mut impl BufMut, value: &[u8]) {
    value.len().wire_write(buffer);
    buffer.put_slice(value);
}

pub fn varlen_fill(buffer: &mut impl BufMut, value: u8, count: usize) {
    count.wire_write(buffer);
    buffer.put_bytes(value, count);
}

pub fn varlen_read(buffer: &mut impl Buf) -> Result<BytesMut> {
    let len = usize::wire_read(buffer)?;
    if buffer.remaining() < len {
        return Err(Error);
    }
    let mut value = BytesMut::zeroed(len);
    buffer.copy_to_slice(value.as_mut());
    Ok(value)
}

pub fn varlen_skip(buffer: &mut impl Buf) -> Result<usize> {
    let len = usize::wire_read(buffer)?;
    generic_skip(buffer, len)?;
    Ok(len)
}

pub fn fixed_write<const N: usize>(buffer: &mut impl BufMut, value: &[u8; N]) {
    buffer.put_slice(value);
}

pub fn fixed_read<const N: usize>(buffer: &mut impl Buf) -> Result<[u8; N]> {
    if buffer.remaining() < N {
        return Err(Error);
    }
    let mut value = [0u8; N];
    buffer.copy_to_slice(&mut value);
    Ok(value)
}

pub fn fixed_skip<const N: usize>(buffer: &mut impl Buf) -> Result<()> {
    generic_skip(buffer, N)
}

pub fn generic_skip(buffer: &mut impl Buf, len: usize) -> Result<()> {
    if buffer.remaining() < len {
        return Err(Error);
    }
    buffer.advance(len);
    Ok(())
}

pub fn generic_skip_many<T: BaseSerializable>(buffer: &mut impl Buf, count: usize) -> Result<()> {
    for _ in 0..count {
        T::wire_skip(buffer)?;
    }
    Ok(())
}

pub fn eq<const N: usize>(lhs: &[u8; N], rhs: &[u8; N]) -> bool {
    constant_time_eq::constant_time_eq_n::<N>(lhs, rhs)
}
