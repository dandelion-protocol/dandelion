use alloc::{fmt, vec::Vec};

use dandelion_wire::{
    bytes::{Buf, BufMut},
    util,
    BaseSerializable,
    Encryptable,
    Error,
    FixedSizeSerializable,
    Printable,
    Result,
    Serializable,
    Typed,
    UUID,
};

use super::{Attestation, Block, BlockID, Envelope, Priority};

#[derive(Clone)]
#[repr(transparent)]
pub struct Messages(pub Vec<Message>);

impl Typed for Messages {
    const TYPE_UUID: UUID = crate::constants::MESSAGES_TYPE;
}

impl Encryptable for Messages {}

impl_serializable_for_wrapper!(Messages, wraps Vec<Message>);
impl_printable_for_wrapper!(Messages);
impl_debug_for_printable!(Messages);
impl_display_for_printable!(Messages);

#[derive(Clone)]
pub enum Message {
    Padding(usize),
    Attestation(Attestation),
    Envelope(Envelope),
    HaveBlock(Block),
    WantBlock(DesireBlockID),
    DontWantBlock(BlockID),
}

pub mod codes {
    pub const PADDING: u16 = 0x0000;
    pub const ATTESTATION: u16 = 0x0001;
    pub const ENVELOPE: u16 = 0x0002;
    pub const HAVE_BLOCK: u16 = 0x0100;
    pub const WANT_BLOCK: u16 = 0x0101;
    pub const DONT_WANT_BLOCK: u16 = 0x0102;
}

pub mod names {
    pub const PADDING: &str = "Padding";
    pub const ATTESTATION: &str = "Attestation";
    pub const ENVELOPE: &str = "Envelope";
    pub const HAVE_BLOCK: &str = "HaveBlock";
    pub const WANT_BLOCK: &str = "WantBlock";
    pub const DONT_WANT_BLOCK: &str = "DontWantBlock";
}

impl Message {
    pub fn code(&self) -> u16 {
        match self {
            Self::Padding(_) => codes::PADDING,
            Self::Attestation(_) => codes::ATTESTATION,
            Self::Envelope(_) => codes::ENVELOPE,
            Self::HaveBlock(_) => codes::HAVE_BLOCK,
            Self::WantBlock(_) => codes::WANT_BLOCK,
            Self::DontWantBlock(_) => codes::DONT_WANT_BLOCK,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Padding(_) => names::PADDING,
            Self::Attestation(_) => names::ATTESTATION,
            Self::Envelope(_) => names::ENVELOPE,
            Self::HaveBlock(_) => names::HAVE_BLOCK,
            Self::WantBlock(_) => names::WANT_BLOCK,
            Self::DontWantBlock(_) => names::DONT_WANT_BLOCK,
        }
    }
}

impl BaseSerializable for Message {
    fn wire_write(&self, buffer: &mut impl BufMut) {
        match self {
            Self::Padding(len) => {
                codes::PADDING.wire_write(buffer);
                util::varlen_fill(buffer, 0, *len);
            },
            Self::Attestation(att) => {
                codes::ATTESTATION.wire_write(buffer);
                util::nested_write(buffer, att);
            },
            Self::Envelope(env) => {
                codes::ENVELOPE.wire_write(buffer);
                util::nested_write(buffer, env);
            },
            Self::HaveBlock(block) => {
                codes::HAVE_BLOCK.wire_write(buffer);
                util::nested_write(buffer, block);
            },
            Self::WantBlock(desire) => {
                codes::WANT_BLOCK.wire_write(buffer);
                util::nested_write(buffer, desire);
            },
            Self::DontWantBlock(id) => {
                codes::DONT_WANT_BLOCK.wire_write(buffer);
                util::nested_write(buffer, id);
            },
        }
    }
    fn wire_read(buffer: &mut impl Buf) -> Result<Self> {
        use util::{nested_read, varlen_skip};
        let code = u16::wire_read(buffer)?;
        match code {
            codes::PADDING => Ok(Self::Padding(varlen_skip(buffer)?)),
            codes::ATTESTATION => Ok(Self::Attestation(nested_read::<Attestation>(buffer)?)),
            codes::ENVELOPE => Ok(Self::Envelope(nested_read::<Envelope>(buffer)?)),
            codes::HAVE_BLOCK => Ok(Self::HaveBlock(nested_read::<Block>(buffer)?)),
            codes::WANT_BLOCK => Ok(Self::WantBlock(nested_read::<DesireBlockID>(buffer)?)),
            codes::DONT_WANT_BLOCK => Ok(Self::DontWantBlock(nested_read::<BlockID>(buffer)?)),
            _ => Err(Error),
        }
    }
    fn wire_skip(buffer: &mut impl Buf) -> Result<()> {
        u16::wire_skip(buffer)?;
        let _ = util::varlen_skip(buffer)?;
        Ok(())
    }
}

impl Serializable for Message {
    fn wire_size(&self) -> usize {
        use util::{nested_wire_size, varlen_wire_size};
        u16::WIRE_SIZE.strict_add(match self {
            Self::Padding(len) => varlen_wire_size(*len),
            Self::Attestation(att) => nested_wire_size(att),
            Self::Envelope(env) => nested_wire_size(env),
            Self::HaveBlock(block) => nested_wire_size(block),
            Self::WantBlock(desire) => nested_wire_size(desire),
            Self::DontWantBlock(id) => nested_wire_size(id),
        })
    }
}

impl Printable for Message {
    fn print(&self, writer: &mut dyn fmt::Write) -> fmt::Result {
        match self {
            Self::Padding(len) => write!(writer, "Padding({})", len),
            Self::Attestation(att) => write!(writer, "Attestation({})", att),
            Self::Envelope(env) => write!(writer, "Envelope({})", env),
            Self::HaveBlock(block) => write!(writer, "HaveBlock({})", block),
            Self::WantBlock(desire) => write!(writer, "WantBlock({})", desire),
            Self::DontWantBlock(id) => write!(writer, "DontWantBlock({})", id),
        }
    }
}

impl_debug_for_printable!(Message);
impl_display_for_printable!(Message);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct DesireBlockID {
    pub block_id: BlockID,
    pub priority: Priority,
}

impl_serializable_for_struct!(DesireBlockID { block_id: BlockID, priority: Priority }, fixed size);
impl_printable_for_struct!(DesireBlockID { block_id, priority });
impl_debug_for_printable!(DesireBlockID);
impl_display_for_printable!(DesireBlockID);
