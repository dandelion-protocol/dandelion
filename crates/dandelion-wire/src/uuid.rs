use super::{bytes::Buf, dandelion_wire, BaseSerializable, Error, Result};

public_bytes!(UUID, raw RawUUID, size UUID_SIZE = 16);

impl UUID {
    pub fn wire_verify(self, buffer: &mut impl Buf) -> Result<()> {
        let actual = Self::wire_read(buffer)?;
        if actual == self {
            Ok(())
        } else {
            Err(Error)
        }
    }
}

pub trait Typed {
    const TYPE_UUID: UUID;
}
