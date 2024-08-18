use alloc::fmt;

use dandelion_wire::{
    bytes::{Buf, BufMut},
    BaseSerializable,
    Error,
    FixedSizeSerializable,
    Printable,
    Result,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Priority {
    Least = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

pub mod codes {
    use super::Priority;
    pub const LEAST: u8 = Priority::Least as u8;
    pub const LOW: u8 = Priority::Low as u8;
    pub const MEDIUM: u8 = Priority::Medium as u8;
    pub const HIGH: u8 = Priority::High as u8;
}

pub mod names {
    pub const LEAST: &str = "Least";
    pub const LOW: &str = "Low";
    pub const MEDIUM: &str = "Medium";
    pub const HIGH: &str = "High";
}

impl Priority {
    pub fn from_code(code: u8) -> Result<Self> {
        match code {
            codes::LEAST => Ok(Self::Least),
            codes::LOW => Ok(Self::Low),
            codes::MEDIUM => Ok(Self::Medium),
            codes::HIGH => Ok(Self::High),
            _ => Err(Error),
        }
    }
    pub fn code(self) -> u8 {
        match self {
            Self::Least => codes::LEAST,
            Self::Low => codes::LOW,
            Self::Medium => codes::MEDIUM,
            Self::High => codes::HIGH,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Least => names::LEAST,
            Self::Low => names::LOW,
            Self::Medium => names::MEDIUM,
            Self::High => names::HIGH,
        }
    }
}

impl BaseSerializable for Priority {
    fn wire_write(&self, buffer: &mut dyn BufMut) {
        self.code().wire_write(buffer);
    }
    fn wire_read(buffer: &mut dyn Buf) -> Result<Self> {
        Ok(Self::from_code(u8::wire_read(buffer)?)?)
    }
    fn wire_skip(buffer: &mut dyn Buf) -> Result<()> {
        u8::wire_skip(buffer)
    }
}

impl FixedSizeSerializable for Priority {
    const WIRE_SIZE: usize = u8::WIRE_SIZE;
}

impl Printable for Priority {
    fn print(&self, writer: &mut dyn fmt::Write) -> fmt::Result {
        writer.write_str(self.name())
    }
}

impl_debug_for_printable!(Priority);
impl_display_for_printable!(Priority);

impl TryFrom<u8> for Priority {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_code(code)
    }
}

impl From<Priority> for u8 {
    fn from(value: Priority) -> Self {
        value.code()
    }
}
