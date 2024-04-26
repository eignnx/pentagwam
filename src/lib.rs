#![deny(
    clippy::undocumented_unsafe_blocks,
    clippy::multiple_unsafe_ops_per_block,
    clippy::missing_safety_doc,
    clippy::default_union_representation
)]

pub mod cell;
pub mod defs;
pub mod mem;
pub mod syntax;
pub mod unify;
