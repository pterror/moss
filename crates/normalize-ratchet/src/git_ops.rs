//! Git operations for the ratchet crate — thin wrappers over `normalize_vcs::Vcs`.
//!
//! Goes through the VCS trait boundary (no direct `gix` types) so this crate works
//! unchanged if a non-git backend is ever added.
use normalize_vcs::{GitBackend, Vcs};
use std::path::Path;

/// Read every file (blob) in the tree at `git_ref`, calling `visitor` with its
/// repo-relative path and decoded text content (`None` if not valid UTF-8).
pub fn read_files_at_ref(
    root: &Path,
    git_ref: &str,
    visitor: impl FnMut(&str, Option<String>),
) -> anyhow::Result<()> {
    GitBackend.read_files_at_ref(root, git_ref, visitor)
}
