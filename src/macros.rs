macro_rules! impl_enum {
    ( $ty:ident repr $repr:ty {
        $( $variant:ident = [$const:ident, $str:literal, $num:literal] ),*
        $(,)?
    } ) => {
        #[derive(::core::clone::Clone, ::core::marker::Copy, ::core::hash::Hash, ::core::cmp::PartialEq, ::core::cmp::Eq)]
        #[repr($repr)]
        pub enum $ty {
            $( $variant = $num, )*
        }

        pub mod codes {
            $( pub const $const: $repr = super::$ty::$variant as $repr; )*
        }

        pub mod names {
            $( pub const $const: &str = $str; )*
        }

        impl $ty {
            pub fn from_code(code: $repr) -> ::dandelion_wire::Result<Self> {
                match code {
                    $( codes::$const => Ok(Self::$variant), )*
                    _ => Err(::dandelion_wire::Error),
                }
            }
            pub fn code(self) -> $repr {
                match self {
                    $( Self::$variant => codes::$const, )*
                }
            }
            pub fn name(self) -> &'static str {
                match self {
                    $( Self::$variant => names::$const, )*
                }
            }
        }

        impl ::dandelion_wire::BaseSerializable for $ty {
            fn wire_write(&self, buffer: &mut dyn ::dandelion_wire::bytes::BufMut) {
                self.code().wire_write(buffer);
            }
            fn wire_read(buffer: &mut dyn ::dandelion_wire::bytes::Buf) -> ::dandelion_wire::Result<Self> {
                Ok(Self::from_code(<$repr>::wire_read(buffer)?)?)
            }
            fn wire_skip(buffer: &mut dyn ::dandelion_wire::bytes::Buf) -> ::dandelion_wire::Result<()> {
                <$repr>::wire_skip(buffer)?;
                Ok(())
            }
        }

        impl ::dandelion_wire::FixedSizeSerializable for $ty {
            const WIRE_SIZE: usize = <$repr>::WIRE_SIZE;
        }

        impl ::dandelion_wire::Printable for $ty {
            fn print(&self, writer: &mut dyn ::alloc::fmt::Write) -> ::alloc::fmt::Result {
                writer.write_str(self.name())
            }
        }

        impl ::core::fmt::Debug for $ty {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                fmt.write_str("EntityType::")?;
                fmt.write_str(self.name())
            }
        }

        impl ::core::fmt::Display for $ty {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                fmt.write_str(self.name())
            }
        }

        impl ::core::convert::TryFrom<$repr> for $ty {
            type Error = ::dandelion_wire::Error;
            fn try_from(code: $repr) -> ::dandelion_wire::Result<Self> {
                Self::from_code(code)
            }
        }

        impl ::core::convert::From<$ty> for $repr {
            fn from(value: $ty) -> Self {
                value.code()
            }
        }

        impl ::core::convert::From<$ty> for &'static str {
            fn from(value: $ty) -> &'static str {
                value.name()
            }
        }
    };
}
