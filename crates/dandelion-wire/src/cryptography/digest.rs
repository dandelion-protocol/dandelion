use cryptoxide::blake2s;

use super::PublicBytes;
use crate::{bytes::Buf, dandelion_wire, UUID};

pub type Context = blake2s::Blake2s;

public_bytes!(Digest, raw RawDigest, size DIGEST_SIZE = 32);

impl Digest {
    pub fn new_context(key: UUID) -> Context {
        Context::new_keyed(DIGEST_SIZE, key.as_slice())
    }

    pub fn compute(key: UUID, mut data: impl Buf) -> Self {
        use cryptoxide::digest::Digest;
        let mut context = Self::new_context(key);
        while data.has_remaining() {
            let chunk = data.chunk();
            context.input(chunk);
            data.advance(chunk.len());
        }
        let mut digest = Self::default();
        context.result(digest.as_slice_mut());
        digest
    }
}
