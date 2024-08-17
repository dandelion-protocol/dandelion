pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Error;

macro_rules! error_from {
    ($source:ty) => {
        impl From<$source> for Error {
            fn from(_: $source) -> Self {
                Self
            }
        }
    };
}

error_from!(core::convert::Infallible);
error_from!(core::num::TryFromIntError);
error_from!(chacha20poly1305::Error);
error_from!(ed25519_dalek::SignatureError);
