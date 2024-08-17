use dandelion_wire::{cryptography::sig::PublicKey, Signable, Typed, UUID};

use super::{Claims, Entity, Instant};

#[derive(Clone)]
pub struct Attestation {
    pub attestor: Entity,
    pub time: Instant,
    pub claims: Claims,
}

impl Typed for Attestation {
    const TYPE_UUID: UUID = crate::constants::ATTESTATION_TYPE;
}

impl Signable for Attestation {
    fn signer(&self) -> PublicKey {
        self.attestor.public_key
    }
}

impl_serializable_for_struct!(Attestation { attestor: Entity, time: Instant, claims: Claims });
impl_printable_for_struct!(Attestation { attestor, time, claims });
impl_debug_for_printable!(Attestation);
impl_display_for_printable!(Attestation);
