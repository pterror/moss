//! Tool adapters.
//!
//! Each adapter wraps an external tool and provides:
//! - Availability detection
//! - Project relevance detection
//! - Output parsing to diagnostics

mod biome;
mod clippy;
mod deno;
mod eslint;
mod gofmt;
mod mypy;
mod oxfmt;
mod oxlint;
mod prettier;
mod pyright;
mod ruff;
mod rustfmt;
mod tsc;
mod tsgo;

pub use biome::{BiomeFormat, BiomeLint};
pub use clippy::Clippy;
pub use deno::Deno;
pub use eslint::Eslint;
pub use gofmt::{Gofmt, Govet};
pub use mypy::Mypy;
pub use oxfmt::Oxfmt;
pub use oxlint::Oxlint;
pub use prettier::Prettier;
pub use pyright::Pyright;
pub use ruff::Ruff;
pub use rustfmt::Rustfmt;
pub use tsc::Tsc;
pub use tsgo::Tsgo;

use crate::Tool;

/// Create a registry with all built-in adapters.
pub fn all_adapters() -> Vec<Box<dyn Tool>> {
    vec![
        // Python
        Box::new(Ruff::new()),
        Box::new(Mypy::new()),
        Box::new(Pyright::new()),
        // JavaScript/TypeScript (oxc toolchain preferred over eslint/prettier)
        Box::new(Oxlint::new()),
        Box::new(Oxfmt::new()),
        Box::new(Eslint::new()),
        Box::new(BiomeLint::new()),
        Box::new(BiomeFormat::new()),
        Box::new(Prettier::new()),
        Box::new(Tsgo::new()), // Native TypeScript (faster than tsc)
        Box::new(Tsc::new()),
        Box::new(Deno::new()),
        // Rust
        Box::new(Clippy::new()),
        Box::new(Rustfmt::new()),
        // Go
        Box::new(Gofmt::new()),
        Box::new(Govet::new()),
    ]
}
