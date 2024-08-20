#![no_std]
#![feature(strict_overflow_ops, const_strict_overflow_ops)]

extern crate alloc;

#[macro_use]
extern crate dandelion_wire;

#[macro_use]
mod macros;

pub mod attestation;
pub mod block;
pub mod claim;
pub mod constants;
pub mod entity;
pub mod envelope;
pub mod message;
pub mod priority;
pub mod time;

pub use attestation::Attestation;
pub use block::{Block, BlockID};
pub use claim::{Claim, Claims};
pub use entity::{Entity, EntityType};
pub use envelope::Envelope;
pub use message::{Message, Messages};
pub use priority::Priority;
pub use time::{Duration, Instant};
