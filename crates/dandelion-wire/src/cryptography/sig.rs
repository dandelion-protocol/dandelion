use ed25519_dalek::{
    Signature as DalekSignature,
    Signer,
    SigningKey as DalekPrivateKey,
    Verifier,
    VerifyingKey as DalekPublicKey,
};

use super::*;

secret_bytes!(PrivateKey, raw RawPrivateKey, size PRIVATE_KEY_SIZE = 32);
public_bytes!(PublicKey, raw RawPublicKey, size PUBLIC_KEY_SIZE = 32);
public_bytes!(Signature, raw RawSignature, size SIGNATURE_SIZE = 64);

impl PrivateKey {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn from_dalek(dalek: DalekPrivateKey) -> Self {
        Self::from_exposed(dalek.to_bytes())
    }

    pub fn as_dalek(&self) -> DalekPrivateKey {
        DalekPrivateKey::from_bytes(self.expose())
    }

    pub fn into_dalek(self) -> DalekPrivateKey {
        self.as_dalek()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_dalek(self.as_dalek().verifying_key())
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        Signature::from_dalek(self.as_dalek().sign(data))
    }
}

impl PublicKey {
    pub fn from_dalek(dalek: DalekPublicKey) -> Self {
        Self::from_exact(*dalek.as_bytes())
    }

    pub fn try_as_dalek(&self) -> Result<DalekPublicKey> {
        Ok(DalekPublicKey::from_bytes(self.as_exact())?)
    }

    pub fn try_into_dalek(self) -> Result<DalekPublicKey> {
        self.try_as_dalek()
    }

    pub fn verify(&self, data: &[u8], sig: Signature) -> Result<()> {
        Ok(self.try_as_dalek()?.verify(data, &sig.into_dalek())?)
    }
}

impl Signature {
    pub fn from_dalek(dalek: DalekSignature) -> Self {
        Self::from_exact(dalek.to_bytes())
    }

    pub fn as_dalek(&self) -> DalekSignature {
        DalekSignature::from_bytes(self.as_exact())
    }

    pub fn into_dalek(self) -> DalekSignature {
        self.as_dalek()
    }
}

impl core::convert::From<SharedSecret> for PrivateKey {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}
impl core::convert::From<PrivateKey> for DalekPrivateKey {
    fn from(value: PrivateKey) -> Self {
        value.into_dalek()
    }
}
impl core::convert::TryFrom<PublicKey> for DalekPublicKey {
    type Error = Error;
    fn try_from(value: PublicKey) -> Result<Self> {
        value.try_into_dalek()
    }
}
impl core::convert::TryFrom<&PublicKey> for DalekPublicKey {
    type Error = Error;
    fn try_from(value: &PublicKey) -> Result<Self> {
        value.try_as_dalek()
    }
}
impl core::convert::From<Signature> for DalekSignature {
    fn from(value: Signature) -> Self {
        value.into_dalek()
    }
}
impl core::convert::From<&Signature> for DalekSignature {
    fn from(value: &Signature) -> Self {
        value.as_dalek()
    }
}

impl core::convert::From<DalekPrivateKey> for PrivateKey {
    fn from(value: DalekPrivateKey) -> Self {
        Self::from_dalek(value)
    }
}
impl core::convert::From<DalekPublicKey> for PublicKey {
    fn from(value: DalekPublicKey) -> Self {
        Self::from_dalek(value)
    }
}
impl core::convert::From<DalekSignature> for Signature {
    fn from(value: DalekSignature) -> Self {
        Self::from_dalek(value)
    }
}
