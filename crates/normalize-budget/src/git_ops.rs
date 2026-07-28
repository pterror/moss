//! Git operations for the budget crate — thin wrappers over `normalize_vcs::Vcs`.
//!
//! All metric collection goes through the VCS trait boundary (no direct `gix` types)
//! so this crate works unchanged if a non-git backend is ever added.
pub use normalize_vcs::{ChangeKind, FileContentChange};
use normalize_vcs::{GitBackend, Vcs};
use std::path::Path;

/// Diff between `base_ref` and HEAD, including old/new content for every changed file.
pub fn diff_base_to_head(root: &Path, base_ref: &str) -> anyhow::Result<Vec<FileContentChange>> {
    GitBackend.diff_file_contents(root, base_ref)
}

/// Read every file (blob) in the tree at `git_ref`, calling `visitor` with its
/// repo-relative path and decoded text content (`None` if not valid UTF-8).
pub fn read_files_at_ref(
    root: &Path,
    git_ref: &str,
    visitor: impl FnMut(&str, Option<String>),
) -> anyhow::Result<()> {
    GitBackend.read_files_at_ref(root, git_ref, visitor)
}
