use alloc::{
    boxed::Box,
    fmt::{Debug, Display},
};

use bytes::{Bytes, BytesMut};
use zeroize::{DefaultIsZeroes, Zeroize};

use crate::{dandelion_wire, Printable, Serializable};

pub mod cipher;
pub mod digest;
pub mod ecdh;
pub mod hkdf;
pub mod sig;

pub trait SecretBytes<const N: usize>: Printable + Debug + Display + Zeroize + Sized {
    fn from_box(inner: Box<[u8; N]>) -> Self;
    fn into_box(self) -> Box<[u8; N]>;
    fn expose(&self) -> &[u8; N];

    fn len() -> usize {
        N
    }

    fn from_exposed(raw: [u8; N]) -> Self {
        Self::from_box(Box::new(raw))
    }

    fn from_exposed_slice(raw: &[u8]) -> Self {
        assert_eq!(N, raw.len());
        Self::from_exposed(*raw.first_chunk::<N>().unwrap())
    }
}

pub trait PublicBytes<const N: usize>:
    Serializable
    + Printable
    + Debug
    + Display
    + DefaultIsZeroes
    + AsRef<[u8; N]>
    + AsRef<[u8]>
    + Eq
    + Sized
{
    fn from_exact(raw: [u8; N]) -> Self;
    fn as_exact(&self) -> &[u8; N];
    fn as_exact_mut(&mut self) -> &mut [u8; N];

    fn len() -> usize {
        N
    }

    fn zero() -> Self {
        Self::from_exact([0u8; N])
    }

    fn from_slice(raw: &[u8]) -> Self {
        assert_eq!(N, raw.len());
        Self::from_exact(*raw.first_chunk::<N>().unwrap())
    }

    fn from_bytes(raw: &Bytes) -> Self {
        Self::from_slice(raw.as_ref())
    }

    fn from_bytes_mut(raw: &BytesMut) -> Self {
        Self::from_slice(raw.as_ref())
    }

    fn into_exact(self) -> [u8; N] {
        *self.as_exact()
    }

    fn as_slice(&self) -> &[u8] {
        self.as_exact()
    }

    fn as_slice_mut(&mut self) -> &mut [u8] {
        self.as_exact_mut()
    }

    fn as_static_bytes(&'static self) -> Bytes {
        Bytes::from_static(self.as_slice())
    }

    fn as_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(self.as_slice())
    }

    fn as_bytes_mut(&self) -> BytesMut {
        BytesMut::from(self.as_bytes())
    }
}

secret_bytes!(SharedSecret, raw RawSharedSecret, size SHARED_SECRET_SIZE = 32);

impl SharedSecret {
    pub fn from_cryptoxide(secret: cryptoxide::x25519::SharedSecret) -> Self {
        Self::from_exposed_slice(secret.as_ref())
    }
}

impl From<cryptoxide::x25519::SharedSecret> for SharedSecret {
    fn from(secret: cryptoxide::x25519::SharedSecret) -> Self {
        Self::from_cryptoxide(secret)
    }
}
