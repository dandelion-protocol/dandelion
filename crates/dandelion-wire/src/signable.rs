use alloc::fmt;

use bytes::{Bytes, BytesMut};

use super::{
    cryptography::sig::{PrivateKey, PublicKey, Signature},
    *,
};

pub trait Signable: Typed + Serializable {
    fn signer(&self) -> PublicKey;

    fn seal(&self, key: &PrivateKey) -> Signed {
        let signer = key.public_key();
        assert_eq!(signer, self.signer());
        let payload = Bytes::from(util::serialize(self));
        let prepared = prepare_payload(Self::TYPE_UUID, signer, payload.as_ref());
        let signature = key.sign(prepared.as_ref());
        Signed { signer, payload, signature }
    }

    fn unseal(signed: &Signed) -> Result<Self> {
        let signer = signed.signer;
        let prepared = prepare_payload(Self::TYPE_UUID, signer, signed.payload.as_ref());
        signer.verify(prepared.as_ref(), signed.signature)?;
        let value = util::deserialize::<Self>(signed.payload.clone())?;
        if value.signer() == signer {
            Ok(value)
        } else {
            Err(Error)
        }
    }
}

fn prepare_payload(type_uuid: UUID, signer: PublicKey, payload: &[u8]) -> Bytes {
    let cap = payload.len().strict_add(UUID::WIRE_SIZE).strict_add(PublicKey::WIRE_SIZE);
    let mut buf = BytesMut::with_capacity(cap);
    type_uuid.wire_write(&mut buf);
    signer.wire_write(&mut buf);
    buf.extend_from_slice(payload);
    Bytes::from(buf)
}

#[derive(Clone)]
pub struct Signed {
    pub signer: PublicKey,
    pub payload: Bytes,
    pub signature: Signature,
}

impl Printable for Signed {
    fn print(&self, writer: &mut dyn fmt::Write) -> fmt::Result {
        write!(writer, "<{} bytes of data, signed by {}>", self.payload.len(), self.signer)
    }
}

impl_serializable_for_struct!(Signed { signer: PublicKey, payload: Bytes, signature: Signature });
impl_debug_for_printable!(Signed);
impl_display_for_printable!(Signed);
