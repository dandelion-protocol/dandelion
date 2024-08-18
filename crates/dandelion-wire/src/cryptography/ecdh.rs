use x25519_dalek::{PublicKey as DalekPublicKey, StaticSecret as DalekPrivateKey};

use super::*;

secret_bytes!(PrivateKey, raw RawPrivateKey, size PRIVATE_KEY_SIZE = 32);
public_bytes!(PublicKey, raw RawPublicKey, size PUBLIC_KEY_SIZE = 32);

impl PrivateKey {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn from_dalek(dalek: DalekPrivateKey) -> Self {
        Self::from_exposed(dalek.to_bytes())
    }

    pub fn as_dalek(&self) -> DalekPrivateKey {
        DalekPrivateKey::from(*self.expose())
    }

    pub fn into_dalek(self) -> DalekPrivateKey {
        self.as_dalek()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_dalek(DalekPublicKey::from(&self.as_dalek()))
    }

    pub fn diffie_hellman(&self, partner: PublicKey) -> Result<SharedSecret> {
        let dalek_shared_secret = self.as_dalek().diffie_hellman(&partner.as_dalek());
        match dalek_shared_secret.was_contributory() {
            true => Ok(SharedSecret::from_dalek(dalek_shared_secret)),
            false => Err(Error),
        }
    }
}

impl PublicKey {
    pub fn from_dalek(dalek: DalekPublicKey) -> Self {
        Self::from_exact(*dalek.as_bytes())
    }

    pub fn as_dalek(&self) -> DalekPublicKey {
        DalekPublicKey::from(*self.as_exact())
    }

    pub fn into_dalek(self) -> DalekPublicKey {
        self.as_dalek()
    }
}

impl From<SharedSecret> for PrivateKey {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}
impl From<PrivateKey> for DalekPrivateKey {
    fn from(value: PrivateKey) -> Self {
        value.into_dalek()
    }
}
impl From<PublicKey> for DalekPublicKey {
    fn from(value: PublicKey) -> Self {
        value.into_dalek()
    }
}
impl From<&PublicKey> for DalekPublicKey {
    fn from(value: &PublicKey) -> Self {
        value.as_dalek()
    }
}
impl From<DalekPrivateKey> for PrivateKey {
    fn from(value: DalekPrivateKey) -> Self {
        Self::from_dalek(value)
    }
}
impl From<DalekPublicKey> for PublicKey {
    fn from(value: DalekPublicKey) -> Self {
        Self::from_dalek(value)
    }
}
