//! Git utility functions — thin wrappers over `normalize_vcs::Vcs`.
//!
//! Preserves the existing call-site names (`git_utils::resolve_ref`, etc.) so
//! `git_history.rs`, `skeleton_diff.rs`, `provenance.rs`, and `view/history.rs` didn't
//! need to change, while routing every operation through the VCS trait boundary instead
//! of depending on `normalize-git` (and its `gix` types) directly.
use normalize_vcs::{GitBackend, Vcs};
use std::path::Path;

/// A single commit with its unix timestamp.
pub struct CommitEntry {
    pub hash: String,
    pub timestamp: i64,
}

/// Get all commits with hash and timestamp, oldest first.
pub fn git_log_timestamps(root: &Path) -> Result<Vec<CommitEntry>, String> {
    let entries = GitBackend.log_timestamps(root)?;
    Ok(entries
        .into_iter()
        .map(|e| CommitEntry {
            hash: e.hash,
            timestamp: e.timestamp,
        })
        .collect())
}

/// Resolve a git ref (branch name, tag, short hash, HEAD~N, etc.) to a full commit hash.
pub fn resolve_ref(root: &Path, git_ref: &str) -> Result<String, String> {
    GitBackend.resolve_ref(root, git_ref)
}

/// Resolve base ref to merge-base with HEAD.
pub fn resolve_merge_base(root: &Path, base: &str) -> Result<String, String> {
    GitBackend.resolve_merge_base(root, base)
}

/// Create a detached worktree at `hash`, run `callback`, then remove the worktree.
pub fn run_in_worktree<T, F>(root: &Path, hash: &str, callback: F) -> Result<T, String>
where
    F: FnOnce(&Path) -> Result<T, String>,
{
    GitBackend.run_in_worktree(root, hash, callback)
}

/// Format a unix timestamp as YYYY-MM-DD.
pub fn format_unix_date(ts: i64) -> String {
    normalize_vcs::format_unix_date(ts)
}

/// Read the content of `file_path` (repo-relative) at git ref `git_ref`.
pub fn git_show(root: &Path, git_ref: &str, file_path: &str) -> Option<String> {
    GitBackend.show(root, git_ref, file_path)
}

/// Status of a file in a diff (name-status level: no blob OIDs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffFileStatus {
    Added,
    Deleted,
    Modified,
}

/// Return a list of (status, path) pairs for files changed between `base_ref` and HEAD.
pub fn git_diff_name_status(
    root: &Path,
    base_ref: &str,
) -> Result<Vec<(DiffFileStatus, String)>, String> {
    let raw = GitBackend.diff_name_status(root, base_ref)?;
    Ok(raw
        .into_iter()
        .map(|(kind, path)| {
            let status = match kind {
                normalize_vcs::ChangeKind::Added => DiffFileStatus::Added,
                normalize_vcs::ChangeKind::Deleted => DiffFileStatus::Deleted,
                normalize_vcs::ChangeKind::Modified => DiffFileStatus::Modified,
            };
            (status, path)
        })
        .collect())
}

/// Return all file paths tracked by git (i.e. in the index).
pub fn git_ls_files(root: &Path) -> Vec<String> {
    GitBackend.ls_files(root)
}

/// Return a list of per-commit changed file paths, for temporal coupling analysis.
pub fn git_per_commit_files(root: &Path) -> Vec<Vec<String>> {
    GitBackend
        .walk_commit_history(root, None)
        .into_iter()
        .map(|entry| entry.files.into_iter().map(|f| f.path).collect())
        .filter(|files: &Vec<String>| !files.is_empty())
        .collect()
}

/// Per-line blame for `path` (repo-relative) at the current checkout, aggregated by
/// commit hash into a line count per commit — matches `git blame --line-porcelain`'s
/// per-commit tally.
pub fn blame_line_counts_by_commit(
    root: &Path,
    path: &str,
) -> Option<std::collections::HashMap<String, usize>> {
    let lines = GitBackend.blame(root, path)?;
    let mut commit_lines: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for line in lines {
        *commit_lines.entry(line.commit.0).or_default() += 1;
    }
    if commit_lines.is_empty() {
        None
    } else {
        Some(commit_lines)
    }
}
