use dandelion_wire::{cryptography::sig::PublicKey, Encrypted, Signable, Typed, UUID};

use super::Entity;

#[derive(Clone)]
pub struct Envelope {
    pub sender: Entity,
    pub recipient: Entity,
    pub payload: Encrypted,
}

impl Typed for Envelope {
    const TYPE_UUID: UUID = crate::constants::ENVELOPE_TYPE;
}

impl Signable for Envelope {
    fn signer(&self) -> PublicKey {
        self.sender.public_key
    }
}

impl_serializable_for_struct!(Envelope { sender: Entity, recipient: Entity, payload: Encrypted });
impl_printable_for_struct!(Envelope { sender, recipient, payload });
impl_debug_for_printable!(Envelope);
impl_display_for_printable!(Envelope);
