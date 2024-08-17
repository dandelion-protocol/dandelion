use alloc::fmt;

use dandelion_wire::{
    bytes::{Buf, BufMut},
    cryptography::sig::PublicKey,
    BaseSerializable,
    Error,
    FixedSizeSerializable,
    Printable,
    Result,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Entity {
    pub entity_type: EntityType,
    pub public_key: PublicKey,
}

impl_serializable_for_struct!(Entity { entity_type: EntityType, public_key: PublicKey }, fixed size);
impl_printable_for_struct!(Entity { entity_type, public_key });
impl_debug_for_printable!(Entity);
impl_display_for_printable!(Entity);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum EntityType {
    Endpoint = 0,
    Node = 1,
}

pub mod codes {
    pub const ENDPOINT: u16 = super::EntityType::Endpoint as u16;
    pub const NODE: u16 = super::EntityType::Node as u16;
}

pub mod names {
    pub const ENDPOINT: &str = "Endpoint";
    pub const NODE: &str = "Node";
}

impl EntityType {
    pub fn from_code(code: u16) -> Result<Self> {
        match code {
            codes::ENDPOINT => Ok(Self::Endpoint),
            codes::NODE => Ok(Self::Node),
            _ => Err(Error),
        }
    }
    pub fn code(self) -> u16 {
        match self {
            Self::Endpoint => codes::ENDPOINT,
            Self::Node => codes::NODE,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Endpoint => names::ENDPOINT,
            Self::Node => names::NODE,
        }
    }
}

impl BaseSerializable for EntityType {
    fn wire_write(&self, buffer: &mut impl BufMut) {
        self.code().wire_write(buffer);
    }
    fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
        Ok(Self::from_code(u16::wire_read(buffer)?)?)
    }
    fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
        u16::wire_skip(buffer)?;
        Ok(())
    }
}

impl FixedSizeSerializable for EntityType {
    const WIRE_SIZE: usize = u16::WIRE_SIZE;
}

impl Printable for EntityType {
    fn print(&self, writer: &mut dyn fmt::Write) -> fmt::Result {
        writer.write_str(self.name())
    }
}

impl_debug_for_printable!(EntityType);
impl_display_for_printable!(EntityType);
