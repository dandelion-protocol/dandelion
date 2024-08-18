use cryptoxide::chacha20poly1305::{self, DecryptionResult};

use super::{PublicBytes, SecretBytes, SharedSecret};
use crate::{
    bytes::{Buf, BufMut, BytesMut},
    dandelion_wire,
    Error,
    Result,
};

const ROUNDS: usize = 20;

pub type Context = chacha20poly1305::Context<ROUNDS>;
pub type ContextEncryption = chacha20poly1305::ContextEncryption<ROUNDS>;
pub type ContextDecryption = chacha20poly1305::ContextDecryption<ROUNDS>;

secret_bytes!(Key, raw RawKey, size KEY_SIZE = 32);
public_bytes!(Nonce, raw RawNonce, size NONCE_SIZE = 24);
public_bytes!(Tag, raw RawTag, size TAG_SIZE = 16);

impl Key {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn as_context(&self, nonce: Nonce) -> Context {
        Context::new(self.expose(), nonce.as_slice())
    }

    pub fn encrypt_in_place(
        &self,
        nonce: Nonce,
        associated: Option<&[u8]>,
        buffer: &mut [u8],
    ) -> Tag {
        let mut context = self.as_context(nonce);
        context.add_data(associated.unwrap_or(&[]));
        let mut context = context.to_encryption();
        context.encrypt_mut(buffer);
        Tag::from_cryptoxide(context.finalize())
    }

    pub fn decrypt_in_place(
        &self,
        nonce: Nonce,
        associated: Option<&[u8]>,
        buffer: &mut [u8],
        tag: Tag,
    ) -> Result<()> {
        let mut context = self.as_context(nonce);
        context.add_data(associated.unwrap_or(&[]));
        let mut context = context.to_decryption();
        context.decrypt_mut(buffer);
        match context.finalize(&tag.as_cryptoxide()) {
            DecryptionResult::Match => Ok(()),
            DecryptionResult::MisMatch => Err(Error),
        }
    }

    pub fn encrypt(
        &self,
        nonce: Nonce,
        associated: &mut dyn Buf,
        plaintext: &mut dyn Buf,
        ciphertext: &mut impl BufMut,
    ) -> Tag {
        let mut context = self.as_context(nonce);
        consume_chunks(associated, |x| context.add_data(x));
        let mut context = context.to_encryption();
        transform_chunks(plaintext, ciphertext, |a, b| context.encrypt(a, b));
        Tag::from_cryptoxide(context.finalize())
    }

    pub fn decrypt(
        &self,
        nonce: Nonce,
        associated: &mut dyn Buf,
        ciphertext: &mut dyn Buf,
        plaintext: &mut impl BufMut,
        tag: Tag,
    ) -> Result<()> {
        let mut context = self.as_context(nonce);
        consume_chunks(associated, |x| context.add_data(x));
        let mut context = context.to_decryption();
        transform_chunks(ciphertext, plaintext, |a, b| context.decrypt(a, b));
        match context.finalize(&tag.as_cryptoxide()) {
            DecryptionResult::Match => Ok(()),
            DecryptionResult::MisMatch => Err(Error),
        }
    }
}

impl Tag {
    pub fn from_cryptoxide(tag: chacha20poly1305::Tag) -> Self {
        Self::from_exact(tag.0)
    }
    pub fn as_cryptoxide(&self) -> chacha20poly1305::Tag {
        chacha20poly1305::Tag(*self.as_exact())
    }
    pub fn into_cryptoxide(self) -> chacha20poly1305::Tag {
        chacha20poly1305::Tag(self.into_exact())
    }
}

impl From<SharedSecret> for Key {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}

fn consume_chunks(input: &mut dyn Buf, mut callback: impl FnMut(&[u8])) {
    while input.has_remaining() {
        let chunk = input.chunk();
        callback(chunk);
        input.advance(chunk.len());
    }
}

fn transform_chunks(
    input: &mut dyn Buf,
    output: &mut impl BufMut,
    mut callback: impl FnMut(&[u8], &mut [u8]),
) {
    while input.has_remaining() {
        let in_chunk = input.chunk();
        let mut out_chunk = BytesMut::zeroed(in_chunk.len());
        callback(in_chunk, &mut out_chunk);
        output.put(out_chunk);
        input.advance(in_chunk.len());
    }
}
