use alloc::vec::Vec;

#[derive(Clone)]
pub struct Claims(pub Vec<Claim>);

impl_serializable_for_wrapper!(Claims, wraps Vec<Claim>);
impl_printable_for_wrapper!(Claims);
impl_debug_for_printable!(Claims);
impl_display_for_printable!(Claims);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Claim;

impl_serializable_todo!(Claim);
impl_printable_todo!(Claim);
impl_debug_for_printable!(Claim);
impl_display_for_printable!(Claim);
