use dandelion_wire::{cryptography::digest::Digest, PublicBytes};

public_bytes!(Block, raw RawBlock, size BLOCK_SIZE = 1 << 20);

impl Block {
    pub fn compute_id(&self) -> BlockID {
        BlockID::compute_from(self)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct BlockID(pub Digest);

impl BlockID {
    pub fn compute_from(block: &Block) -> Self {
        Self(Digest::compute(crate::constants::BLOCK_TYPE, block.as_slice()))
    }
}

impl_serializable_for_wrapper!(BlockID, wraps Digest, fixed size);
impl_printable_for_wrapper!(BlockID);
impl_debug_for_printable!(BlockID);
impl_display_for_printable!(BlockID);
