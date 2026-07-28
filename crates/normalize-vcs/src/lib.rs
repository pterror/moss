//! VCS-agnostic trait boundary over version-control backends.
//!
//! `normalize` today only has a git backend ([`GitBackend`], wrapping `normalize-git`,
//! itself pure-Rust `gix` with no shell-out except worktree add/remove). This crate exists
//! so every consumer that needs commit/blame/diff/history data depends on normalize's own
//! domain types ([`CommitId`], [`BlameLine`], [`CommitWalkEntry`], ...) instead of reaching
//! into `gix` directly. Per TODO.md's "VCS abstraction layer" entry: VCS-agnosticism is a
//! settled product principle ("normalize should Just Work no matter what the user's tooling
//! is"), so this crate exists now even though git is the only backend implemented today. A
//! second backend (jj, Mercurial) is future work, not implemented here.
//!
//! ## Scope
//!
//! As of the "fully subsume normalize-git" migration, every in-workspace crate that touches
//! git goes through [`Vcs`] — no workspace crate other than this one depends on
//! `normalize-git` directly. The trait's surface is exactly the set of operations real
//! callers use today (see each method's doc for its consumer); nothing here is speculative.
//!
//! ## The worktree hard case
//!
//! [`Vcs::run_in_worktree`] is git-specific and does **not** generalize cleanly to a
//! hypothetical jj/Mercurial backend: `gix` has no worktree add/remove (this shells out to
//! the real `git` binary), and jj's working-copy model doesn't map onto git worktrees at
//! all — jj has no equivalent "detached checkout of an arbitrary commit into a scratch
//! directory" primitive; the closest analogue would be a workspace-add plus an edit to the
//! target commit, which is a different operation with different semantics (it moves the
//! *working copy*, not a side directory). Rather than pretend this method is
//! backend-agnostic, it stays on the trait as a named capability whose doc is explicit that
//! it is a git-only operation today: a future second backend either implements the same
//! contract (materialize `hash` into a temporary directory for the duration of the
//! callback) some other way, or the call site needs a capability check. No attempt is made
//! here to design that fallback speculatively — it's future work for whoever adds the
//! second backend.
//!
//! Scope is deliberately narrow — this is a boundary extraction, not a green-field API
//! design. Only operations that actually cross a `gix`-typed boundary in a real consumer are
//! wrapped here.

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

/// Kind of change a file underwent in a diff or commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Deleted,
    Modified,
}

/// A single commit with its unix timestamp (oldest-first history listings).
#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub timestamp: i64,
}

/// One author's total commit count in a repository.
#[derive(Debug, Clone)]
pub struct AuthorCommitCount {
    pub name: String,
    pub email: String,
    pub commits: usize,
}

/// One commit's hash, timestamp, and decoded message (subject/body split at the first
/// blank line, matching `git log`'s convention).
#[derive(Debug, Clone)]
pub struct CommitMessage {
    pub hash: String,
    pub timestamp: i64,
    pub subject: String,
    pub body: String,
}

/// One unique commit found while blaming a line range, with enough metadata to render
/// a `git log -L`-style history entry (no full diff, unlike `git log -L` itself).
#[derive(Debug, Clone)]
pub struct BlameRangeCommit {
    pub commit: CommitId,
    pub author_name: String,
    pub timestamp: i64,
    pub message_summary: String,
}

/// A file changed between two refs, with its content at both ends (for diff-based metrics
/// that need to inspect old/new file content, not just the change kind).
///
/// Content is read eagerly for every changed file — simpler than threading a live repo
/// handle back to the caller for on-demand blob reads, at the cost of reading content for
/// files a caller only inspects by kind (e.g. counting added/deleted files). Diffs are
/// typically dozens of files, not repo-wide, so this trade favors a single non-leaking
/// primitive over an optimization that's rarely load-bearing in practice.
#[derive(Debug, Clone)]
pub struct FileContentChange {
    pub path: String,
    pub kind: ChangeKind,
    /// Raw content in the base ref's tree (`None` for added files).
    pub old_content: Option<Vec<u8>>,
    /// Raw content in HEAD's tree (`None` for deleted files).
    pub new_content: Option<Vec<u8>>,
}

impl FileContentChange {
    /// `old_content` decoded as UTF-8 text, or `None` if absent/not valid UTF-8.
    pub fn old_text(&self) -> Option<String> {
        self.old_content
            .as_ref()
            .and_then(|b| String::from_utf8(b.clone()).ok())
    }

    /// `new_content` decoded as UTF-8 text, or `None` if absent/not valid UTF-8.
    pub fn new_text(&self) -> Option<String> {
        self.new_content
            .as_ref()
            .and_then(|b| String::from_utf8(b.clone()).ok())
    }
}

/// One file's change within a single commit, as part of a [`Vcs::walk_commit_history`] walk.
#[derive(Debug, Clone)]
pub struct CommitFileChange {
    pub path: String,
    pub kind: ChangeKind,
    /// Lines added, by a simple line-count heuristic (not a full Myers diff).
    pub lines_added: usize,
    /// Lines deleted, by the same heuristic.
    pub lines_deleted: usize,
}

/// One commit visited during a [`Vcs::walk_commit_history`] walk.
#[derive(Debug, Clone)]
pub struct CommitWalkEntry {
    pub commit: CommitId,
    pub timestamp: u64,
    pub author_email: String,
    pub files: Vec<CommitFileChange>,
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

    /// Resolve a ref (branch, tag, short hash, `HEAD~N`, ...) to a full commit hash.
    fn resolve_ref(&self, root: &Path, git_ref: &str) -> Result<String, String>;

    /// Resolve the merge-base between `base` and HEAD, falling back to `base` resolved
    /// as-is if no merge-base can be found.
    fn resolve_merge_base(&self, root: &Path, base: &str) -> Result<String, String>;

    /// Materialize `hash` into a scratch working directory for the duration of `callback`,
    /// then remove it — even if `callback` errors.
    ///
    /// **Git-specific, not backend-agnostic.** See the crate-level docs' "worktree hard
    /// case" section: this shells out to the `git` binary today because `gix` has no
    /// worktree add/remove, and a jj backend would need a different contract entirely.
    fn run_in_worktree<T>(
        &self,
        root: &Path,
        hash: &str,
        callback: impl FnOnce(&Path) -> Result<T, String>,
    ) -> Result<T, String>;

    /// Read the content of `file_path` (repo-relative) at `git_ref`, or `None` if the ref,
    /// path, or content can't be resolved/decoded.
    fn show(&self, root: &Path, git_ref: &str, file_path: &str) -> Option<String>;

    /// Name-and-status diff (no content) between `base_ref` and HEAD.
    fn diff_name_status(
        &self,
        root: &Path,
        base_ref: &str,
    ) -> Result<Vec<(ChangeKind, String)>, String>;

    /// Diff between `base_ref` and HEAD, including old/new content for every changed file.
    fn diff_file_contents(
        &self,
        root: &Path,
        base_ref: &str,
    ) -> anyhow::Result<Vec<FileContentChange>>;

    /// All file paths tracked by the repository (i.e. in the index).
    fn ls_files(&self, root: &Path) -> Vec<String>;

    /// Fetch URL of the `origin` remote, or `None`.
    fn remote_origin_url(&self, root: &Path) -> Option<String>;

    /// All commit timestamps (seconds since epoch) reachable from HEAD, newest first.
    fn commit_timestamps(&self, root: &Path) -> Vec<u64>;

    /// All commits with hash and timestamp, oldest first.
    fn log_timestamps(&self, root: &Path) -> Result<Vec<CommitEntry>, String>;

    /// Per-author commit counts across the whole history.
    fn author_commit_counts(&self, root: &Path) -> Vec<AuthorCommitCount>;

    /// Walk commit history from HEAD (newest first), stopping (exclusive) at
    /// `since_commit` if given, yielding per-commit file changes with line churn.
    ///
    /// Computes line churn eagerly for every file in every commit — more work than a bare
    /// path-listing walk, but avoids maintaining divergent "just paths" vs "paths + churn"
    /// walk primitives. Used by hotspot/activity analyses that need churn anyway, and by
    /// coupling/co-change analyses that only need paths and discard the rest.
    fn walk_commit_history(&self, root: &Path, since_commit: Option<&str>) -> Vec<CommitWalkEntry>;

    /// Read every file (blob) in the tree at `git_ref`, calling `visitor` with its
    /// repo-relative path and decoded text content (`None` if not valid UTF-8).
    fn read_files_at_ref(
        &self,
        root: &Path,
        git_ref: &str,
        visitor: impl FnMut(&str, Option<String>),
    ) -> anyhow::Result<()>;

    /// Count commits (newest-first from HEAD) before the most recent commit that touched
    /// `rel_path`. `None` if no commit ever touched the path, or history can't be walked.
    fn commits_since_last_touch(&self, root: &Path, rel_path: &str) -> Option<usize>;

    /// Walk up to `max_commits` commits from HEAD (newest first) and return their hash,
    /// timestamp, and decoded message. Used for commit-message embedding (semantic search
    /// over commit history).
    fn recent_commit_messages(&self, root: &Path, max_commits: usize) -> Vec<CommitMessage>;

    /// Blame `path` restricted to `[start_line, end_line]` (1-based, inclusive), returning
    /// up to `limit` unique commits that touched those lines, newest first — analogous to
    /// `git log -L` but implemented via the blame algorithm.
    fn blame_range_commits(
        &self,
        root: &Path,
        path: &str,
        start_line: usize,
        end_line: usize,
        limit: usize,
    ) -> Result<Vec<BlameRangeCommit>, String>;
}

/// Format a Unix timestamp as `YYYY-MM-DD`. Pure date math — not backend-specific — kept
/// here (rather than as a `Vcs` method) because every backend would compute it identically.
pub fn format_unix_date(ts: i64) -> String {
    normalize_git::format_unix_date(ts)
}

/// Git-backed implementation of [`Vcs`], wrapping `normalize-git` (pure-Rust `gix`)
/// and converting its `gix`-typed results to this crate's domain types at the
/// boundary. The only backend today.
#[derive(Debug, Default, Clone, Copy)]
pub struct GitBackend;

impl GitBackend {
    fn line_diff(
        repo: &gix::Repository,
        old_id: Option<gix::hash::ObjectId>,
        new_id: Option<gix::hash::ObjectId>,
    ) -> normalize_git::LineDiff {
        normalize_git::count_diff_lines(repo, old_id, new_id)
    }
}

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

    fn resolve_ref(&self, root: &Path, git_ref: &str) -> Result<String, String> {
        normalize_git::resolve_ref(root, git_ref)
    }

    fn resolve_merge_base(&self, root: &Path, base: &str) -> Result<String, String> {
        normalize_git::resolve_merge_base(root, base)
    }

    fn run_in_worktree<T>(
        &self,
        root: &Path,
        hash: &str,
        callback: impl FnOnce(&Path) -> Result<T, String>,
    ) -> Result<T, String> {
        normalize_git::run_in_worktree(root, hash, callback)
    }

    fn show(&self, root: &Path, git_ref: &str, file_path: &str) -> Option<String> {
        normalize_git::git_show(root, git_ref, file_path)
    }

    fn diff_name_status(
        &self,
        root: &Path,
        base_ref: &str,
    ) -> Result<Vec<(ChangeKind, String)>, String> {
        let raw = normalize_git::git_diff_name_status(root, base_ref)?;
        Ok(raw
            .into_iter()
            .map(|(status, path)| {
                let kind = match status {
                    normalize_git::DiffFileStatus::Added => ChangeKind::Added,
                    normalize_git::DiffFileStatus::Deleted => ChangeKind::Deleted,
                    normalize_git::DiffFileStatus::Modified => ChangeKind::Modified,
                };
                (kind, path)
            })
            .collect())
    }

    fn diff_file_contents(
        &self,
        root: &Path,
        base_ref: &str,
    ) -> anyhow::Result<Vec<FileContentChange>> {
        let repo = normalize_git::open_repo(root)
            .ok_or_else(|| anyhow::anyhow!("not a git repository: {}", root.display()))?;
        let changes = normalize_git::diff_base_to_head(root, base_ref)?;
        Ok(changes
            .into_iter()
            .map(|c| {
                let kind = match c.kind {
                    normalize_git::FileChangeKind::Added => ChangeKind::Added,
                    normalize_git::FileChangeKind::Deleted => ChangeKind::Deleted,
                    normalize_git::FileChangeKind::Modified => ChangeKind::Modified,
                };
                FileContentChange {
                    path: c.path,
                    kind,
                    old_content: c
                        .old_id
                        .and_then(|id| normalize_git::read_blob_bytes(&repo, id)),
                    new_content: c
                        .new_id
                        .and_then(|id| normalize_git::read_blob_bytes(&repo, id)),
                }
            })
            .collect())
    }

    fn ls_files(&self, root: &Path) -> Vec<String> {
        normalize_git::git_ls_files(root)
    }

    fn remote_origin_url(&self, root: &Path) -> Option<String> {
        normalize_git::git_remote_origin_url(root)
    }

    fn commit_timestamps(&self, root: &Path) -> Vec<u64> {
        normalize_git::git_commit_timestamps(root)
    }

    fn log_timestamps(&self, root: &Path) -> Result<Vec<CommitEntry>, String> {
        let raw = normalize_git::git_log_timestamps(root)?;
        Ok(raw
            .into_iter()
            .map(|e| CommitEntry {
                hash: e.hash,
                timestamp: e.timestamp,
            })
            .collect())
    }

    fn author_commit_counts(&self, root: &Path) -> Vec<AuthorCommitCount> {
        normalize_git::git_author_commit_counts(root)
            .into_iter()
            .map(|e| AuthorCommitCount {
                name: e.name,
                email: e.email,
                commits: e.commits,
            })
            .collect()
    }

    fn walk_commit_history(&self, root: &Path, since_commit: Option<&str>) -> Vec<CommitWalkEntry> {
        let Some(repo) = normalize_git::open_repo(root) else {
            return Vec::new();
        };
        normalize_git::walk_commits(root, since_commit)
            .into_iter()
            .map(|wc| {
                let files = wc
                    .changes
                    .into_iter()
                    .map(|c| {
                        let kind = match (c.old_id.is_some(), c.new_id.is_some()) {
                            (false, true) => ChangeKind::Added,
                            (true, false) => ChangeKind::Deleted,
                            _ => ChangeKind::Modified,
                        };
                        let diff = Self::line_diff(&repo, c.old_id, c.new_id);
                        CommitFileChange {
                            path: c.path,
                            kind,
                            lines_added: diff.added,
                            lines_deleted: diff.deleted,
                        }
                    })
                    .collect();
                CommitWalkEntry {
                    commit: CommitId(wc.id),
                    timestamp: wc.timestamp,
                    author_email: wc.author_email,
                    files,
                }
            })
            .collect()
    }

    fn read_files_at_ref(
        &self,
        root: &Path,
        git_ref: &str,
        mut visitor: impl FnMut(&str, Option<String>),
    ) -> anyhow::Result<()> {
        let repo = normalize_git::open_repo(root)
            .ok_or_else(|| anyhow::anyhow!("not a git repository: {}", root.display()))?;
        normalize_git::walk_tree_at_ref(root, git_ref, |rel_path, blob_id| {
            let content = normalize_git::read_blob_text(&repo, blob_id);
            visitor(rel_path, content);
        })
    }

    fn commits_since_last_touch(&self, root: &Path, rel_path: &str) -> Option<usize> {
        normalize_git::commits_since_last_touch(root, rel_path)
    }

    fn recent_commit_messages(&self, root: &Path, max_commits: usize) -> Vec<CommitMessage> {
        normalize_git::recent_commit_messages(root, max_commits)
            .into_iter()
            .map(|c| CommitMessage {
                hash: c.hash,
                timestamp: c.timestamp,
                subject: c.subject,
                body: c.body,
            })
            .collect()
    }

    fn blame_range_commits(
        &self,
        root: &Path,
        path: &str,
        start_line: usize,
        end_line: usize,
        limit: usize,
    ) -> Result<Vec<BlameRangeCommit>, String> {
        use gix::bstr::ByteSlice;

        let repo = gix::discover(root).map_err(|e| format!("Not a git repository: {e}"))?;
        let head_id = repo
            .head_id()
            .map_err(|e| format!("Failed to resolve HEAD: {e}"))?;

        let path_bstr: &gix::bstr::BStr = path.as_bytes().into();
        let ranges = gix::blame::BlameRanges::from_one_based_inclusive_range(
            (start_line as u32)..=(end_line as u32),
        )
        .map_err(|e| format!("Invalid line range {start_line}..={end_line}: {e}"))?;

        let outcome = repo
            .blame_file(
                path_bstr,
                head_id.detach(),
                gix::repository::blame_file::Options {
                    ranges,
                    ..Default::default()
                },
            )
            .map_err(|e| format!("git blame failed for {path}: {e}"))?;

        // Collect unique commit ids preserving order (newest first from blame output).
        let mut seen = std::collections::HashSet::new();
        let mut unique_commit_ids: Vec<gix::hash::ObjectId> = Vec::new();
        for entry in &outcome.entries {
            if seen.insert(entry.commit_id) {
                unique_commit_ids.push(entry.commit_id);
            }
            if unique_commit_ids.len() >= limit {
                break;
            }
        }

        // Resolve each commit id to author/date/message.
        let mut commits = Vec::new();
        for commit_id in unique_commit_ids {
            let Ok(obj) = repo.find_object(commit_id) else {
                continue;
            };
            let commit_obj = obj.into_commit();
            let author_name = commit_obj
                .author()
                .ok()
                .map(|a| a.name.to_str_lossy().into_owned())
                .unwrap_or_default();
            let timestamp = commit_obj.time().ok().map(|t| t.seconds).unwrap_or(0);
            let message_summary = commit_obj
                .message()
                .ok()
                .map(|m| m.summary().to_str_lossy().into_owned())
                .unwrap_or_default();
            commits.push(BlameRangeCommit {
                commit: CommitId(commit_id.to_hex().to_string()),
                author_name,
                timestamp,
                message_summary,
            });
        }

        Ok(commits)
    }
}
