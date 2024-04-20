#![deny(
    clippy::undocumented_unsafe_blocks,
    clippy::multiple_unsafe_ops_per_block,
    clippy::missing_safety_doc,
    clippy::default_union_representation
)]

mod cell;
mod defs;
mod mem;
mod parse;
mod unify;

fn main() {}
