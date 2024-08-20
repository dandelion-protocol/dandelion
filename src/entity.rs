use dandelion_wire::cryptography::sig::PublicKey;

impl_enum!(EntityType repr u16 {
    Endpoint = [ENDPOINT, "Endpoint", 0],
    Node = [NODE, "Node", 1],
    Zone = [ZONE, "Zone", 2],
});

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Entity {
    pub entity_type: EntityType,
    pub public_key: PublicKey,
}

impl_serializable_for_struct!(Entity { entity_type: EntityType, public_key: PublicKey }, fixed size);
impl_printable_for_struct!(Entity { entity_type, public_key });
impl_debug_for_printable!(Entity);
impl_display_for_printable!(Entity);
