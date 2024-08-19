#[macro_export]
macro_rules! impl_debug_for_printable {
    ( $ty:ty ) => {
        #[allow(unused_imports)]
        impl ::alloc::fmt::Debug for $ty {
            fn fmt(&self, fmt: &mut ::alloc::fmt::Formatter<'_>) -> ::alloc::fmt::Result {
                use ::alloc::fmt::Write;
                use dandelion_wire::Printable;
                fmt.write_str(::core::any::type_name::<Self>())?;
                fmt.write_char('(')?;
                self.print(fmt)?;
                fmt.write_char(')')
            }
        }
    };
}

#[macro_export]
macro_rules! impl_display_for_printable {
    ( $ty:ty ) => {
        #[allow(unused_imports)]
        impl ::alloc::fmt::Display for $ty {
            fn fmt(&self, fmt: &mut ::alloc::fmt::Formatter<'_>) -> ::alloc::fmt::Result {
                use dandelion_wire::Printable;
                self.print(fmt)
            }
        }
    };
}

#[macro_export]
macro_rules! secret_bytes {
    ( $ty:ident, raw $raw:ident, size $const:ident = $size:expr ) => {
        pub const $const: usize = $size;
        pub type $raw = [u8; $const];

        #[derive(::core::clone::Clone)]
        #[repr(transparent)]
        pub struct $ty(::alloc::boxed::Box<$raw>);

        impl dandelion_wire::SecretBytes<$const> for $ty {
            fn from_box(inner: ::alloc::boxed::Box<$raw>) -> Self {
                Self(inner)
            }

            fn into_box(mut self) -> ::alloc::boxed::Box<$raw> {
                let mut dummy = ::alloc::boxed::Box::new([0u8; $const]);
                ::core::mem::swap(&mut dummy, &mut self.0);
                dummy
            }

            fn expose(&self) -> &$raw {
                &self.0
            }
        }

        #[allow(unused_imports)]
        impl ::core::ops::Drop for $ty {
            fn drop(&mut self) {
                use dandelion_wire::zeroize::Zeroize;
                self.zeroize();
            }
        }

        impl dandelion_wire::zeroize::Zeroize for $ty {
            fn zeroize(&mut self) {
                self.0.zeroize();
            }
        }

        #[allow(unused_imports)]
        impl dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut dyn dandelion_wire::bytes::BufMut) {
                use dandelion_wire::{BaseSerializable, SecretBytes};
                self.expose().wire_write(buffer);
            }
            fn wire_read(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<Self> {
                use dandelion_wire::{BaseSerializable, SecretBytes};
                Ok(Self::from_exposed($raw::wire_read(buffer)?))
            }
            fn wire_skip(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<()> {
                use dandelion_wire::BaseSerializable;
                $raw::wire_skip(buffer)
            }
        }

        impl dandelion_wire::FixedSizeSerializable for $ty {
            const WIRE_SIZE: usize = $const;
        }

        impl dandelion_wire::Printable for $ty {
            fn print(&self, writer: &mut dyn ::alloc::fmt::Write) -> ::alloc::fmt::Result {
                dandelion_wire::printable::print_secret_bytes(writer)
            }
        }

        impl_debug_for_printable!($ty);
        impl_display_for_printable!($ty);

        #[allow(unused_imports)]
        impl ::core::convert::From<$raw> for $ty {
            fn from(raw: $raw) -> Self {
                use dandelion_wire::SecretBytes;
                Self::from_exposed(raw)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<::alloc::boxed::Box<$raw>> for $ty {
            fn from(boxed: ::alloc::boxed::Box<$raw>) -> Self {
                use dandelion_wire::SecretBytes;
                Self::from_box(boxed)
            }
        }
    };
}

#[macro_export]
macro_rules! public_bytes {
    ( $ty:ident, raw $raw:ident, size $const:ident = $size:expr ) => {
        pub const $const: usize = $size;
        pub type $raw = [u8; $const];

        #[derive(::core::clone::Clone, ::core::marker::Copy, ::core::hash::Hash)]
        #[repr(transparent)]
        pub struct $ty(pub $raw);

        impl dandelion_wire::PublicBytes<$const> for $ty {
            fn from_exact(raw: $raw) -> Self {
                Self(raw)
            }

            fn as_exact(&self) -> &$raw {
                &self.0
            }

            fn as_exact_mut(&mut self) -> &mut $raw {
                &mut self.0
            }
        }

        #[allow(unused_imports)]
        impl ::core::default::Default for $ty {
            fn default() -> Self {
                use dandelion_wire::PublicBytes;
                Self::zero()
            }
        }

        impl dandelion_wire::zeroize::DefaultIsZeroes for $ty {}

        #[allow(unused_imports)]
        impl dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut dyn dandelion_wire::bytes::BufMut) {
                use dandelion_wire::{BaseSerializable, PublicBytes};
                self.as_exact().wire_write(buffer);
            }
            fn wire_read(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<Self> {
                use dandelion_wire::{BaseSerializable, PublicBytes};
                Ok(Self::from_exact($raw::wire_read(buffer)?))
            }
            fn wire_skip(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<()> {
                use dandelion_wire::BaseSerializable;
                $raw::wire_skip(buffer)
            }
        }

        impl dandelion_wire::FixedSizeSerializable for $ty {
            const WIRE_SIZE: usize = $const;
        }

        #[allow(unused_imports)]
        impl dandelion_wire::Printable for $ty {
            fn print(&self, writer: &mut dyn ::alloc::fmt::Write) -> ::alloc::fmt::Result {
                use dandelion_wire::PublicBytes;
                dandelion_wire::printable::print_public_bytes(writer, self.as_slice())
            }
        }

        impl_debug_for_printable!($ty);
        impl_display_for_printable!($ty);

        impl ::core::cmp::PartialEq for $ty {
            fn eq(&self, rhs: &Self) -> bool {
                use dandelion_wire::PublicBytes;
                dandelion_wire::util::eq(&self.as_exact(), &rhs.as_exact())
            }
        }

        impl ::core::cmp::Eq for $ty {}

        #[allow(unused_imports)]
        impl ::core::convert::AsMut<$raw> for $ty {
            fn as_mut(&mut self) -> &mut $raw {
                use dandelion_wire::PublicBytes;
                self.as_exact_mut()
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::AsMut<[u8]> for $ty {
            fn as_mut(&mut self) -> &mut [u8] {
                use dandelion_wire::PublicBytes;
                self.as_slice_mut()
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::AsRef<$raw> for $ty {
            fn as_ref(&self) -> &$raw {
                use dandelion_wire::PublicBytes;
                self.as_exact()
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::AsRef<[u8]> for $ty {
            fn as_ref(&self) -> &[u8] {
                use dandelion_wire::PublicBytes;
                self.as_slice()
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<$raw> for $ty {
            fn from(raw: $raw) -> Self {
                use dandelion_wire::PublicBytes;
                Self::from_exact(raw)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<&$raw> for $ty {
            fn from(raw: &$raw) -> Self {
                use dandelion_wire::PublicBytes;
                Self::from_exact(*raw)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<&[u8]> for $ty {
            fn from(slice: &[u8]) -> Self {
                use dandelion_wire::PublicBytes;
                Self::from_slice(slice)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<dandelion_wire::bytes::Bytes> for $ty {
            fn from(bytes: dandelion_wire::bytes::Bytes) -> Self {
                use dandelion_wire::PublicBytes;
                Self::from_bytes(&bytes)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<&dandelion_wire::bytes::Bytes> for $ty {
            fn from(bytes: &dandelion_wire::bytes::Bytes) -> Self {
                use dandelion_wire::PublicBytes;
                Self::from_bytes(bytes)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<dandelion_wire::bytes::BytesMut> for $ty {
            fn from(bytes: dandelion_wire::bytes::BytesMut) -> Self {
                use dandelion_wire::PublicBytes;
                Self::from_bytes_mut(&bytes)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<&dandelion_wire::bytes::BytesMut> for $ty {
            fn from(bytes: &dandelion_wire::bytes::BytesMut) -> Self {
                use dandelion_wire::PublicBytes;
                Self::from_bytes_mut(bytes)
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<$ty> for $raw {
            fn from(value: $ty) -> Self {
                use dandelion_wire::PublicBytes;
                value.into_exact()
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<$ty> for dandelion_wire::bytes::Bytes {
            fn from(value: $ty) -> Self {
                use dandelion_wire::PublicBytes;
                value.as_bytes()
            }
        }

        #[allow(unused_imports)]
        impl ::core::convert::From<$ty> for dandelion_wire::bytes::BytesMut {
            fn from(value: $ty) -> Self {
                use dandelion_wire::PublicBytes;
                value.as_bytes_mut()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_serializable_todo {
    ( $ty:ty ) => {
        impl dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, _: &mut dyn dandelion_wire::bytes::BufMut) {
                todo!()
            }
            fn wire_read(_: &mut dyn dandelion_wire::bytes::Buf) -> dandelion_wire::Result<Self> {
                todo!()
            }
            fn wire_skip(_: &mut dyn dandelion_wire::bytes::Buf) -> dandelion_wire::Result<()> {
                todo!()
            }
        }
        impl dandelion_wire::Serializable for $ty {
            fn wire_size(&self) -> usize {
                todo!()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_serializable_for_wrapper {
    ( $ty:ty, wraps $inner:ty, fixed size ) => {
        impl dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut dyn dandelion_wire::bytes::BufMut) {
                self.0.wire_write(buffer)
            }
            fn wire_read(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<Self> {
                Ok(Self(<$inner>::wire_read(buffer)?))
            }
            fn wire_skip(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<()> {
                <$inner>::wire_skip(buffer)?;
                Ok(())
            }
        }
        impl dandelion_wire::FixedSizeSerializable for $ty {
            const WIRE_SIZE: usize = <$inner>::WIRE_SIZE;
        }
    };
    ( $ty:ty, wraps $inner:ty ) => {
        impl dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut dyn dandelion_wire::bytes::BufMut) {
                self.0.wire_write(buffer)
            }
            fn wire_read(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<Self> {
                Ok(Self(<$inner>::wire_read(buffer)?))
            }
            fn wire_skip(
                buffer: &mut dyn dandelion_wire::bytes::Buf,
            ) -> dandelion_wire::Result<()> {
                <$inner>::wire_skip(buffer)?;
                Ok(())
            }
        }
        impl dandelion_wire::Serializable for $ty {
            fn wire_size(&self) -> usize {
                self.0.wire_size()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_serializable_for_struct {
    ( $ty:ty { $( $field:ident : $field_ty:ty ),* $(,)? }, fixed size ) => {
        impl dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut dyn dandelion_wire::bytes::BufMut) {
                $( self.$field.wire_write(buffer); )*
            }
            fn wire_read(buffer: &mut dyn dandelion_wire::bytes::Buf) -> dandelion_wire::Result<Self> {
                Ok(Self {
                    $( $field: <$field_ty>::wire_read(buffer)?, )*
                })
            }
            fn wire_skip(buffer: &mut dyn dandelion_wire::bytes::Buf) -> dandelion_wire::Result<()> {
                $( <$field_ty>::wire_skip(buffer)?; )*
                Ok(())
            }
        }
        impl dandelion_wire::FixedSizeSerializable for $ty {
            const WIRE_SIZE: usize = 0usize $( .strict_add(<$field_ty>::WIRE_SIZE) )*;
        }
    };
    ( $ty:ty { $( $field:ident : $field_ty:ty ),* $(,)? } ) => {
        impl dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut dyn dandelion_wire::bytes::BufMut) {
                $( self.$field.wire_write(buffer); )*
            }
            fn wire_read(buffer: &mut dyn dandelion_wire::bytes::Buf) -> dandelion_wire::Result<Self> {
                Ok(Self {
                    $( $field: <$field_ty>::wire_read(buffer)?, )*
                })
            }
            fn wire_skip(buffer: &mut dyn dandelion_wire::bytes::Buf) -> dandelion_wire::Result<()> {
                $( <$field_ty>::wire_skip(buffer)?; )*
                Ok(())
            }
        }
        impl dandelion_wire::Serializable for $ty {
            fn wire_size(&self) -> usize {
                0usize $( .strict_add(self.$field.wire_size()) )*
            }
        }
    };
}

#[macro_export]
macro_rules! impl_printable_todo {
    ( $ty:ty ) => {
        impl dandelion_wire::Printable for $ty {
            fn print(&self, _: &mut dyn ::alloc::fmt::Write) -> ::alloc::fmt::Result {
                todo!()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_printable_for_wrapper {
    ( $ty:ty ) => {
        impl dandelion_wire::Printable for $ty {
            fn print(&self, writer: &mut dyn ::alloc::fmt::Write) -> ::alloc::fmt::Result {
                self.0.print(writer)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_printable_for_struct {
    ( $ty:ty {} ) => {
        impl dandelion_wire::Printable for $ty {
            fn print(&self, writer: &mut dyn ::alloc::fmt::Write) -> ::alloc::fmt::Result {
                writer.write_str("{}")
            }
        }
    };
    ( $ty:ty { $first:ident $(, $rest:ident )* $(,)? } ) => {
        impl dandelion_wire::Printable for $ty {
            fn print(&self, writer: &mut dyn ::alloc::fmt::Write) -> ::alloc::fmt::Result {
                writer.write_str(concat!("{", stringify!($first), ": "))?;
                self.$first.print(writer)?;
                $(
                    writer.write_str(concat!(", ", stringify!($rest), ": "))?;
                    self.$rest.print(writer)?;
                )*
                writer.write_char('}')
            }
        }
    };
}
