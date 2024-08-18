use alloc::boxed::Box;

use super::*;

type Generator = hkdf::Hkdf<sha2::Sha256>;

secret_bytes!(Seed, raw RawSeed, size SEED_SIZE = 32);

impl Seed {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn as_generator(&self) -> Generator {
        Generator::from_prk(self.expose()).unwrap()
    }

    pub fn into_generator(&self) -> Generator {
        self.as_generator()
    }

    pub fn generate(&self, info: &[u8]) -> SharedSecret {
        let mut scratch = Box::new(RawSharedSecret::default());
        self.as_generator().expand(info, scratch.as_mut()).unwrap();
        SharedSecret::from_box(scratch)
    }

    pub fn generate_into<const N: usize>(&self, info: &[u8], output: &mut [u8; N]) {
        self.as_generator().expand(info, output).unwrap()
    }
}

impl From<SharedSecret> for Seed {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}
impl From<Seed> for Generator {
    fn from(value: Seed) -> Self {
        value.into_generator()
    }
}
