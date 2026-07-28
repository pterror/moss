//! VCS-agnostic trait boundary over version-control backends.
//!
//! `normalize` today only has a git backend (`normalize-git`, itself pure-Rust `gix`,
//! no shell-out except worktree add/remove). This crate exists so consumers that only
//! need blame/history primitives depend on normalize's own domain types (`CommitId`,
//! `BlameLine`, ...) instead of reaching into `gix` directly. Per TODO.md's "VCS
//! abstraction layer" entry: VCS-agnosticism is a settled product principle
//! ("normalize should Just Work no matter what the user's tooling is"), so this crate
//! exists now even though git is the only backend implemented today. A second backend
//! (jj, Mercurial) is future work, not implemented here.
//!
//! Scope is deliberately narrow — this is a boundary extraction, not a green-field API
//! design. Only the operations that actually cross a `gix`-typed boundary in current
//! consumers are wrapped here (blame, repo-exists, commit-count). Most of
//! `normalize-git`'s API (churn stats, per-commit file lists, author counts, ref
//! resolution, diff name-status) already returns plain `String`/`HashMap`/custom
//! structs and doesn't need wrapping — those call sites are left as direct
//! `normalize-git` consumers.

use std::path::Path;

/// Opaque commit identifier (a full hex object id in the git backend).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitId(pub String);

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One blamed line: which commit/author last touched it.
#[derive(Debug, Clone)]
pub struct BlameLine {
    /// 1-based line number in the file at the blamed revision.
    pub line: usize,
    pub commit: CommitId,
    pub author_name: String,
    pub author_email: String,
}

/// VCS-agnostic operations. Implemented today by [`GitBackend`]; a `JjBackend` or
/// `HgBackend` would implement the same trait once a concrete second-backend consumer
/// exists (see `TODO.md`'s "VCS abstraction layer" entry).
pub trait Vcs {
    /// Whether `root` is (inside) a repository this backend recognizes.
    fn repo_exists(&self, root: &Path) -> bool;

    /// Total number of commits reachable from the current checkout, or `None` if the
    /// repository can't be opened or walked.
    fn commit_count(&self, root: &Path) -> Option<usize>;

    /// Per-line blame for `path` (repo-relative) at the current checkout. Returns
    /// `None` if the repo or file can't be opened or blamed.
    fn blame(&self, root: &Path, path: &str) -> Option<Vec<BlameLine>>;
}

/// Git-backed implementation of [`Vcs`], wrapping `normalize-git` (pure-Rust `gix`)
/// and converting its `gix`-typed results to this crate's domain types at the
/// boundary. The only backend today.
#[derive(Debug, Default, Clone, Copy)]
pub struct GitBackend;

impl Vcs for GitBackend {
    fn repo_exists(&self, root: &Path) -> bool {
        normalize_git::open_repo(root).is_some()
    }

    fn commit_count(&self, root: &Path) -> Option<usize> {
        let repo = normalize_git::open_repo(root)?;
        let head_id = repo.head_id().ok()?;
        let walk = head_id.ancestors().all().ok()?;
        Some(walk.filter(|r| r.is_ok()).count())
    }

    fn blame(&self, root: &Path, path: &str) -> Option<Vec<BlameLine>> {
        let repo = normalize_git::open_repo(root)?;
        let head_id = repo.head_id().ok()?;
        let path_bstr: &gix::bstr::BStr = path.as_bytes().into();
        let outcome = repo
            .blame_file(
                path_bstr,
                head_id.detach(),
                gix::repository::blame_file::Options::default(),
            )
            .ok()?;

        let mut lines = Vec::new();
        let mut line_no = 0usize;
        for entry in &outcome.entries {
            let commit_id = entry.commit_id;
            let (author_name, author_email) = repo
                .find_object(commit_id)
                .ok()
                .and_then(|obj| {
                    obj.into_commit().author().ok().map(|a| {
                        (
                            String::from_utf8_lossy(a.name).into_owned(),
                            String::from_utf8_lossy(a.email).into_owned(),
                        )
                    })
                })
                .unwrap_or_else(|| ("Unknown".to_string(), String::new()));
            let commit = CommitId(commit_id.to_hex().to_string());
            for _ in 0..entry.len.get() {
                line_no += 1;
                lines.push(BlameLine {
                    line: line_no,
                    commit: commit.clone(),
                    author_name: author_name.clone(),
                    author_email: author_email.clone(),
                });
            }
        }
        if lines.is_empty() { None } else { Some(lines) }
    }
}
