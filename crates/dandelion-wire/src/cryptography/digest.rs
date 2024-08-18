use blake2s_simd::{Hash as BlakeDigest, Params as BlakeParams, State as BlakeState};
use bytes::Buf;

use super::*;

public_bytes!(Digest, raw RawDigest, size DIGEST_SIZE = 32);

impl Digest {
    pub fn from_blake(blake: BlakeDigest) -> Self {
        Self::from_exact(*blake.as_array())
    }

    pub fn as_blake(&self) -> BlakeDigest {
        BlakeDigest::from(self.as_exact())
    }

    pub fn into_blake(self) -> BlakeDigest {
        self.as_blake()
    }

    pub fn new_blake_params(key: UUID) -> BlakeParams {
        let mut params = BlakeParams::new();
        params.key(key.as_slice());
        params
    }

    pub fn new_blake_state(key: UUID) -> BlakeState {
        Self::new_blake_params(key).to_state()
    }

    pub fn compute(key: UUID, mut data: impl Buf) -> Self {
        let mut state = Self::new_blake_state(key);
        while data.has_remaining() {
            let chunk = data.chunk();
            state.update(&chunk);
            data.advance(chunk.len());
        }
        Self::from_blake(state.finalize())
    }
}

impl From<Digest> for BlakeDigest {
    fn from(value: Digest) -> Self {
        value.into_blake()
    }
}
impl From<&Digest> for BlakeDigest {
    fn from(value: &Digest) -> Self {
        value.as_blake()
    }
}
impl From<BlakeDigest> for Digest {
    fn from(value: BlakeDigest) -> Self {
        Self::from_blake(value)
    }
}
