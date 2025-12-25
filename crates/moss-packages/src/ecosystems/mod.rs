//! Ecosystem implementations.

mod cargo;
mod npm;
mod python;

use crate::Ecosystem;
use std::path::Path;

pub use cargo::Cargo;
pub use npm::Npm;
pub use python::Python;

/// All registered ecosystems.
static ECOSYSTEMS: &[&dyn Ecosystem] = &[&Cargo, &Npm, &Python];

/// Detect ecosystem from project files.
pub fn detect(project_root: &Path) -> Option<&'static dyn Ecosystem> {
    for ecosystem in ECOSYSTEMS {
        for manifest in ecosystem.manifest_files() {
            if project_root.join(manifest).exists() {
                return Some(*ecosystem);
            }
        }
    }
    None
}

/// Get all registered ecosystems.
pub fn all() -> &'static [&'static dyn Ecosystem] {
    ECOSYSTEMS
}
