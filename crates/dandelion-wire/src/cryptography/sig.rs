use cryptoxide::ed25519;

use super::SharedSecret;
use crate::{dandelion_wire, Error, PublicBytes, Result, SecretBytes};

secret_bytes!(PrivateKey, raw RawPrivateKey, size PRIVATE_KEY_SIZE = 32);
public_bytes!(PublicKey, raw RawPublicKey, size PUBLIC_KEY_SIZE = 32);
public_bytes!(Signature, raw RawSignature, size SIGNATURE_SIZE = 64);

impl PrivateKey {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn public_key(&self) -> PublicKey {
        let (_, pk) = ed25519::keypair(self.expose());
        PublicKey::from_exact(pk)
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        let (kp, _) = ed25519::keypair(self.expose());
        Signature::from_exact(ed25519::signature(data, &kp))
    }
}

impl PublicKey {
    pub fn verify(&self, data: &[u8], sig: Signature) -> Result<()> {
        if ed25519::verify(data, self.as_exact(), sig.as_exact()) {
            Ok(())
        } else {
            Err(Error)
        }
    }
}

impl From<SharedSecret> for PrivateKey {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}
