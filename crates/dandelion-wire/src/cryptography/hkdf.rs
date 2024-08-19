use alloc::boxed::Box;

use cryptoxide::{blake2s, hkdf};

use super::{RawSharedSecret, SharedSecret};
use crate::{dandelion_wire, SecretBytes};
type Context = blake2s::Blake2s;

secret_bytes!(Seed, raw RawSeed, size SEED_SIZE = 32);

impl Seed {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn into_shared_secret(self) -> SharedSecret {
        SharedSecret::from_box(self.into_box())
    }

    pub fn from_key_material(salt: &[u8], input: &[u8]) -> Self {
        let context = Context::new(SEED_SIZE);
        let mut seed = Box::new(RawSeed::default());
        hkdf::hkdf_extract(context, salt, input, seed.as_mut());
        Seed::from_box(seed)
    }

    pub fn generate_into(&self, info: &[u8], output: &mut [u8]) {
        let context = Context::new(SEED_SIZE);
        hkdf::hkdf_expand(context, self.expose(), info, output);
    }

    pub fn generate(&self, info: &[u8]) -> SharedSecret {
        let context = Context::new(SEED_SIZE);
        let mut secret = Box::new(RawSharedSecret::default());
        hkdf::hkdf_expand(context, self.expose(), info, secret.as_mut());
        SharedSecret::from_box(secret)
    }
}

impl From<SharedSecret> for Seed {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}
