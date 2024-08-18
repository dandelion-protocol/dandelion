use alloc::fmt;

use bytes::{Bytes, BytesMut};

use super::{
    cryptography::cipher::{Key, Nonce, Tag},
    *,
};

pub trait Encryptable: Typed + Serializable {
    fn encrypt(&self, key: &Key, nonce: Nonce, extra: impl Serializable) -> Encrypted {
        let associated = prepare_associated(Self::TYPE_UUID, nonce, extra);
        let mut buffer = util::serialize(self);
        let tag = key.encrypt_in_place(nonce, Some(associated.as_ref()), &mut buffer);
        let ciphertext = Bytes::from(buffer);
        Encrypted { nonce, ciphertext, tag }
    }

    fn decrypt(encrypted: &Encrypted, key: &Key, extra: impl Serializable) -> Result<Self> {
        let associated = prepare_associated(Self::TYPE_UUID, encrypted.nonce, extra);
        let mut buffer = BytesMut::from(encrypted.ciphertext.clone());
        key.decrypt_in_place(
            encrypted.nonce,
            Some(associated.as_ref()),
            &mut buffer,
            encrypted.tag,
        )?;
        util::deserialize::<Self>(buffer.into())
    }
}

fn prepare_associated(type_uuid: UUID, nonce: Nonce, extra: impl Serializable) -> Bytes {
    let capacity = UUID::WIRE_SIZE.strict_add(Nonce::WIRE_SIZE).strict_add(extra.wire_size());
    let mut buffer = BytesMut::with_capacity(capacity);
    type_uuid.wire_write(&mut buffer);
    nonce.wire_write(&mut buffer);
    extra.wire_write(&mut buffer);
    Bytes::from(buffer)
}

#[derive(Clone)]
pub struct Encrypted {
    pub nonce: Nonce,
    pub ciphertext: Bytes,
    pub tag: Tag,
}

impl Printable for Encrypted {
    fn print(&self, writer: &mut dyn fmt::Write) -> fmt::Result {
        write!(writer, "<{} bytes of encrypted data>", self.ciphertext.len())
    }
}

impl_serializable_for_struct!(Encrypted { nonce: Nonce, ciphertext: Bytes, tag: Tag });
impl_debug_for_printable!(Encrypted);
impl_display_for_printable!(Encrypted);
