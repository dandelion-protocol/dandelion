#![no_std]
#![feature(strict_overflow_ops, const_strict_overflow_ops)]

#[macro_use]
mod macros;

extern crate alloc;
pub extern crate bytes;
pub extern crate zeroize;
use alloc::boxed::Box;
use alloc::fmt;

use crate as dandelion_wire;

pub mod cryptography;
pub mod encryptable;
pub mod error;
pub mod printable;
pub mod serializable;
pub mod signable;
pub mod util;
pub mod uuid;

pub use encryptable::{Encryptable, Encrypted};
pub use error::{Error, Result};
pub use printable::Printable;
pub use serializable::{BaseSerializable, FixedSizeSerializable, Serializable};
pub use signable::{Signable, Signed};
pub use uuid::{Typed, UUID};

pub trait PublicBytes<const N: usize>:
    serializable::Serializable
    + printable::Printable
    + fmt::Debug
    + fmt::Display
    + zeroize::DefaultIsZeroes
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

    fn from_bytes(raw: &bytes::Bytes) -> Self {
        Self::from_slice(raw.as_ref())
    }

    fn from_bytes_mut(raw: &bytes::BytesMut) -> Self {
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

    fn as_static_bytes(&'static self) -> bytes::Bytes {
        bytes::Bytes::from_static(self.as_slice())
    }

    fn as_bytes(&self) -> bytes::Bytes {
        bytes::Bytes::copy_from_slice(self.as_slice())
    }

    fn as_bytes_mut(&self) -> bytes::BytesMut {
        bytes::BytesMut::from(self.as_bytes())
    }
}

pub trait SecretBytes<const N: usize>:
    serializable::Serializable
    + printable::Printable
    + fmt::Debug
    + fmt::Display
    + zeroize::Zeroize
    + Sized
{
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
