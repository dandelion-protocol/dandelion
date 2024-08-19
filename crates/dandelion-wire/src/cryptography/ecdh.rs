use cryptoxide::x25519;

use super::SharedSecret;
use crate::{dandelion_wire, PublicBytes, Result, SecretBytes};

secret_bytes!(PrivateKey, raw RawPrivateKey, size PRIVATE_KEY_SIZE = 32);
public_bytes!(PublicKey, raw RawPublicKey, size PUBLIC_KEY_SIZE = 32);

impl PrivateKey {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn from_cryptoxide(cryptoxide: x25519::SecretKey) -> Self {
        Self::from_exposed_slice(cryptoxide.as_ref())
    }

    pub fn as_cryptoxide(&self) -> x25519::SecretKey {
        x25519::SecretKey::from(*self.expose())
    }

    pub fn into_cryptoxide(self) -> x25519::SecretKey {
        self.as_cryptoxide()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_cryptoxide(x25519::base(&self.as_cryptoxide()))
    }

    pub fn diffie_hellman(&self, partner: PublicKey) -> Result<SharedSecret> {
        Ok(SharedSecret::from_cryptoxide(x25519::dh(
            &self.as_cryptoxide(),
            &partner.as_cryptoxide(),
        )))
    }
}

impl PublicKey {
    pub fn from_cryptoxide(cryptoxide: x25519::PublicKey) -> Self {
        Self::from_slice(cryptoxide.as_ref())
    }

    pub fn as_cryptoxide(&self) -> x25519::PublicKey {
        x25519::PublicKey::from(*self.as_exact())
    }

    pub fn into_cryptoxide(self) -> x25519::PublicKey {
        self.as_cryptoxide()
    }
}

impl From<SharedSecret> for PrivateKey {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}

impl From<x25519::SecretKey> for PrivateKey {
    fn from(value: x25519::SecretKey) -> Self {
        Self::from_cryptoxide(value)
    }
}

impl From<PrivateKey> for x25519::SecretKey {
    fn from(value: PrivateKey) -> Self {
        value.into_cryptoxide()
    }
}

impl From<x25519::PublicKey> for PublicKey {
    fn from(value: x25519::PublicKey) -> Self {
        Self::from_cryptoxide(value)
    }
}

impl From<PublicKey> for x25519::PublicKey {
    fn from(value: PublicKey) -> Self {
        value.into_cryptoxide()
    }
}

impl From<&PublicKey> for x25519::PublicKey {
    fn from(value: &PublicKey) -> Self {
        value.as_cryptoxide()
    }
}
