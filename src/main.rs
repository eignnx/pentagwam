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

fn main() {
    // use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    // // Set up tracing_subscriber to log to stdout.
    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .with(EnvFilter::from_env("LOG_LEVEL"))
    //     .init();
}
