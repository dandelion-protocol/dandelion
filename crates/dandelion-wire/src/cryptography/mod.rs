use crate::dandelion_wire;

pub mod cipher;
pub mod digest;
pub mod ecdh;
pub mod hkdf;
pub mod sig;

secret_bytes!(SharedSecret, raw RawSharedSecret, size SHARED_SECRET_SIZE = 32);

impl SharedSecret {
    pub fn from_cryptoxide(secret: cryptoxide::x25519::SharedSecret) -> Self {
        use dandelion_wire::SecretBytes;
        Self::from_exposed_slice(secret.as_ref())
    }
}

impl From<cryptoxide::x25519::SharedSecret> for SharedSecret {
    fn from(secret: cryptoxide::x25519::SharedSecret) -> Self {
        Self::from_cryptoxide(secret)
    }
}
