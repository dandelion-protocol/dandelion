use bytes::BytesMut;
use chacha20poly1305::{
    AeadInPlace,
    Key as ChaChaKey,
    KeyInit,
    Tag as ChaChaTag,
    XChaCha20Poly1305 as ChaChaCipher,
    XNonce as ChaChaNonce,
};

use super::*;

secret_bytes!(Key, raw RawKey, size KEY_SIZE = 32);
public_bytes!(Nonce, raw RawNonce, size NONCE_SIZE = 24);
public_bytes!(Tag, raw RawTag, size TAG_SIZE = 16);

impl Key {
    pub fn from_shared_secret(secret: SharedSecret) -> Self {
        Self::from_box(secret.into_box())
    }

    pub fn from_chacha(chacha: ChaChaKey) -> Self {
        Self::from_exposed(chacha.into())
    }

    pub fn as_chacha(&self) -> ChaChaKey {
        ChaChaKey::from(*self.expose())
    }

    pub fn into_chacha(self) -> ChaChaKey {
        self.as_chacha()
    }

    pub fn as_cipher(&self) -> ChaChaCipher {
        ChaChaCipher::new(&self.as_chacha())
    }

    pub fn into_cipher(self) -> ChaChaCipher {
        ChaChaCipher::new(&self.as_chacha())
    }

    pub fn encrypt_in_place(
        &self,
        nonce: Nonce,
        associated: Option<&[u8]>,
        buffer: &mut [u8],
    ) -> Tag {
        Tag::from_chacha(
            self.as_cipher()
                .encrypt_in_place_detached(&nonce.as_chacha(), aead_ref(associated), buffer)
                .unwrap(),
        )
    }

    pub fn encrypt(
        &self,
        nonce: Nonce,
        associated: Option<&[u8]>,
        plaintext: &[u8],
    ) -> (BytesMut, Tag) {
        let mut buffer = BytesMut::from(plaintext);
        let tag = self.encrypt_in_place(nonce, associated, &mut buffer);
        (buffer, tag)
    }

    pub fn decrypt_in_place(
        &self,
        nonce: Nonce,
        associated: Option<&[u8]>,
        buffer: &mut [u8],
        tag: Tag,
    ) -> Result<()> {
        Ok(self.as_cipher().decrypt_in_place_detached(
            &nonce.as_chacha(),
            aead_ref(associated),
            buffer,
            &tag.as_chacha(),
        )?)
    }

    pub fn decrypt(
        &self,
        nonce: Nonce,
        associated: Option<&[u8]>,
        ciphertext: &[u8],
        tag: Tag,
    ) -> Result<BytesMut> {
        let mut buffer = BytesMut::from(ciphertext);
        self.decrypt_in_place(nonce, associated, &mut buffer, tag)?;
        Ok(buffer)
    }
}

impl Nonce {
    pub fn from_chacha(chacha: ChaChaNonce) -> Self {
        Self::from_exact(chacha.into())
    }

    pub fn as_chacha(&self) -> ChaChaNonce {
        ChaChaNonce::from(*self.as_exact())
    }

    pub fn into_chacha(self) -> ChaChaNonce {
        self.as_chacha()
    }
}

impl Tag {
    pub fn from_chacha(chacha: ChaChaTag) -> Self {
        Self::from_exact(chacha.into())
    }

    pub fn as_chacha(&self) -> ChaChaTag {
        ChaChaTag::from(*self.as_exact())
    }

    pub fn into_chacha(self) -> ChaChaTag {
        self.as_chacha()
    }
}

impl core::convert::From<SharedSecret> for Key {
    fn from(value: SharedSecret) -> Self {
        Self::from_shared_secret(value)
    }
}
impl core::convert::From<Key> for ChaChaKey {
    fn from(value: Key) -> Self {
        value.into_chacha()
    }
}
impl core::convert::From<Nonce> for ChaChaNonce {
    fn from(value: Nonce) -> Self {
        value.into_chacha()
    }
}
impl core::convert::From<&Nonce> for ChaChaNonce {
    fn from(value: &Nonce) -> Self {
        value.as_chacha()
    }
}
impl core::convert::From<Tag> for ChaChaTag {
    fn from(value: Tag) -> Self {
        value.into_chacha()
    }
}
impl core::convert::From<&Tag> for ChaChaTag {
    fn from(value: &Tag) -> Self {
        value.as_chacha()
    }
}
impl core::convert::From<ChaChaKey> for Key {
    fn from(value: ChaChaKey) -> Self {
        Self::from_chacha(value)
    }
}
impl core::convert::From<ChaChaNonce> for Nonce {
    fn from(value: ChaChaNonce) -> Self {
        Self::from_chacha(value)
    }
}
impl core::convert::From<ChaChaTag> for Tag {
    fn from(value: ChaChaTag) -> Self {
        Self::from_chacha(value)
    }
}

fn aead_ref(associated: Option<&[u8]>) -> &[u8] {
    match associated {
        Some(slice) => slice,
        None => &[],
    }
}
