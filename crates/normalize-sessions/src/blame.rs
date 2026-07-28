//! `normalize sessions blame` — trace a file's line provenance through AI/coding
//! sessions, not just commits.
//!
//! # Matching algorithm (design notes, kept here so the logic is inspectable)
//!
//! `git blame` already answers "which commit last touched this line." The hard
//! part is the next hop: "which *session edit* produced the content that commit
//! introduced." Per the settled design decision (TODO.md, "sessions blame"),
//! this must be **content matching**, not timestamp proximity — no fuzzy
//! best-effort fallback.
//!
//! Key insight that keeps this tractable: **a single commit routinely bundles
//! several session edits (and possibly manual edits too)**, so the unit of
//! attribution is a *chunk* (a maximal run of consecutive lines in the current
//! file blamed to the same commit), not the whole commit. We never assume
//! edit-to-commit is 1:1.
//!
//! Steps, per file (optionally restricted to a line range):
//!
//! 1. **Blame** the current file at HEAD (`Vcs::blame`). This gives, per
//!    current line, the commit that introduced the content sitting there today
//!    (standard git blame semantics: unchanged since that commit).
//! 2. **Group** consecutive lines blamed to the same commit into chunks. This
//!    is the attribution unit — deliberately *not* the whole commit, since a
//!    commit's diff for this file may contain several independent hunks from
//!    different session edits (or a mix of session and manual edits).
//! 3. For the chunk's commit `C`, read the file content at `C` and at `C^`
//!    (`Vcs::show`). This is the actual before/after text `C` produced for this
//!    file.
//! 4. **Search candidate session edits**: every `Edit`/`Write` tool call
//!    (across all scanned sessions) touching this file, with the recorded
//!    message timestamp no later than `C`'s commit timestamp. Timestamp order
//!    is used only as a **hard validity filter** (an edit cannot postdate the
//!    commit it fed), never as a proximity heuristic for ranking matches.
//! 5. A candidate is a **match** for the chunk when, after whitespace
//!    normalization (trailing-space/CRLF tolerant, not fuzzy):
//!    - the chunk's own text is a substring of the edit's `new_string` (for
//!      `Write`, the whole new file content), **and**
//!    - the edit's `old_string` (if any) is a substring of `C^`'s content for
//!      this file — i.e. the edit's "before" text is consistent with what
//!      actually existed at that point in history, **and**
//!    - the edit's `new_string` is a substring of `C`'s own content for this
//!      file — i.e. what the session wrote is consistent with what actually
//!      landed in the commit.
//! 6. Zero matches → **unattributed** (no session correlation found — likely a
//!    manual edit, a commit predating session logging, or a tool other than
//!    Edit/Write, e.g. a `sed` invocation via Bash). More than one match →
//!    **ambiguous**, listing every candidate — reported honestly rather than
//!    guessed, since two edits with byte-identical before/after text for the
//!    same file are inherently indistinguishable from content alone.
//!
//! ## Known limitations (false-negative / false-positive modes)
//!
//! - **Superseded edits are handled correctly, not by luck.** If a session
//!   edits a region twice before committing (edit1 introduces X, edit2
//!   overwrites X with Y), only edit2 matches — because matching is against
//!   the commit's actual final content, not the session's edit log in
//!   isolation. Edit1 naturally fails to match and is never reported.
//! - **A human touch-up after the last session edit, before commit, is a
//!   known false negative.** If a line's exact text was hand-edited after the
//!   session wrote it, it won't byte-match any recorded `new_string`, and is
//!   correctly reported as unattributed rather than wrongly attributed to the
//!   session. This is the intended trade — reliability over recall.
//! - **Very short/generic chunks are refused, not guessed.** A chunk whose
//!   normalized text has fewer than [`MIN_MEANINGFUL_CHARS`] non-whitespace
//!   characters (e.g. a lone `}` or blank line) is reported unattributed with
//!   an explicit "too generic" reason, even if exactly one candidate
//!   textually matches — such matches are not reliable evidence.
//! - **Path matching uses the same heuristic `normalize_path` as `sessions
//!   heatmap`** (stripping to a `src|lib|crates|tests|docs|packages` marker
//!   for absolute paths recorded by tools). Repos that don't use one of those
//!   directory names may see under-matching; this mirrors an existing,
//!   already-accepted limitation elsewhere in this crate rather than
//!   introducing a new one.
//! - **Renames are not tracked.** If `C^`'s content is read from the *same*
//!   path as `C`'s, a same-commit rename will show as if the file were newly
//!   created at that path (parent content empty), which can turn a real match
//!   into an unattributed chunk. Not attempted in this version.

use crate::output::OutputFormatter;
use crate::sessions::{
    ContentBlock, Role, normalize_path, parse_session, parse_session_with_format,
};
use normalize_vcs::{GitBackend, Vcs};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::stats::parse_date;
use super::{SessionMode, list_sessions_by_mode, session_matches_grep};

/// Below this many non-whitespace characters, a chunk's content is considered
/// too generic to confidently attribute even if exactly one candidate
/// textually matches (see module docs).
const MIN_MEANINGFUL_CHARS: usize = 6;

/// One candidate session edit touching the target file, extracted from a
/// scanned session's `Edit`/`Write` tool calls.
#[derive(Debug, Clone)]
struct EditCandidate {
    session_id: String,
    session_path: PathBuf,
    agent_id: Option<String>,
    subagent_type: Option<String>,
    /// Parsed message timestamp (unix seconds), if the session recorded one.
    timestamp: Option<i64>,
    tool: &'static str,
    old_string: Option<String>,
    new_string: String,
}

/// A single matched or unattributed session edit, as reported to the caller.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SessionEditRef {
    pub session_id: String,
    pub session_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_timestamp: Option<i64>,
    pub tool: String,
}

/// Attribution outcome for one blame chunk.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Attribution {
    /// Exactly one session edit's content matches this chunk.
    Matched { edit: SessionEditRef },
    /// More than one session edit matches — reported rather than guessed.
    Ambiguous { candidates: Vec<SessionEditRef> },
    /// No session edit matches (manual commit, pre-instrumentation history,
    /// a non-Edit/Write tool, a human touch-up after the last matching edit,
    /// or content too generic to trust — see `reason`).
    Unattributed { reason: String },
}

/// One contiguous run of current lines blamed to the same commit.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct BlameChunk {
    /// 1-based, inclusive.
    pub start_line: usize,
    /// 1-based, inclusive.
    pub end_line: usize,
    pub commit: String,
    pub commit_short: String,
    pub author_name: String,
    pub author_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_timestamp: Option<i64>,
    pub attribution: Attribution,
}

/// Report for `normalize sessions blame`.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SessionsBlameReport {
    pub path: String,
    pub chunks: Vec<BlameChunk>,
    pub total_lines: usize,
    pub matched_lines: usize,
    pub ambiguous_lines: usize,
    pub unattributed_lines: usize,
}

impl OutputFormatter for SessionsBlameReport {
    fn format_text(&self) -> String {
        let mut out = String::new();
        if self.chunks.is_empty() {
            writeln!(out, "No blame data for {}", self.path).unwrap();
            return out;
        }
        writeln!(
            out,
            "Session blame — {} ({} lines: {} matched, {} ambiguous, {} unattributed)",
            self.path,
            self.total_lines,
            self.matched_lines,
            self.ambiguous_lines,
            self.unattributed_lines
        )
        .unwrap();
        writeln!(out).unwrap();
        for chunk in &self.chunks {
            let range = if chunk.start_line == chunk.end_line {
                format!("{}", chunk.start_line)
            } else {
                format!("{}-{}", chunk.start_line, chunk.end_line)
            };
            let commit_line = format!("commit {} ({})", chunk.commit_short, chunk.author_name);
            match &chunk.attribution {
                Attribution::Matched { edit } => {
                    let agent = edit
                        .agent_id
                        .as_deref()
                        .or(edit.subagent_type.as_deref())
                        .map(|a| format!(" agent={a}"))
                        .unwrap_or_default();
                    writeln!(
                        out,
                        "  {:<8} {}  -> session {} ({}{})",
                        range, commit_line, edit.session_id, edit.tool, agent
                    )
                    .unwrap();
                }
                Attribution::Ambiguous { candidates } => {
                    let ids: Vec<&str> = candidates.iter().map(|c| c.session_id.as_str()).collect();
                    writeln!(
                        out,
                        "  {:<8} {}  -> ambiguous ({} candidates: {})",
                        range,
                        commit_line,
                        candidates.len(),
                        ids.join(", ")
                    )
                    .unwrap();
                }
                Attribution::Unattributed { reason } => {
                    writeln!(
                        out,
                        "  {:<8} {}  -> no session correlation ({})",
                        range, commit_line, reason
                    )
                    .unwrap();
                }
            }
        }
        out
    }
}

/// Whitespace-tolerant normalization: trims trailing whitespace per line and
/// normalizes CRLF/CR to LF, then joins with `\n`. Deliberately *not* fuzzy —
/// this tolerates only trivial formatting differences, not near-matches.
fn normalize_ws(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

fn count_meaningful_chars(s: &str) -> usize {
    s.chars().filter(|c| !c.is_whitespace()).count()
}

/// Extract `(old_string, new_string)`-shaped edits from a tool call.
fn edit_strings(
    tool_name: &str,
    input: &serde_json::Value,
) -> Option<(&'static str, Option<String>, String)> {
    match tool_name {
        "Edit" => {
            let old = input
                .get("old_string")
                .and_then(|v| v.as_str())?
                .to_string();
            let new = input
                .get("new_string")
                .and_then(|v| v.as_str())?
                .to_string();
            Some(("Edit", Some(old), new))
        }
        "Write" => {
            let content = input.get("content").and_then(|v| v.as_str())?.to_string();
            Some(("Write", None, content))
        }
        _ => None,
    }
}

/// Parse an RFC3339-ish session timestamp into unix seconds.
fn parse_session_timestamp(ts: &str) -> Option<i64> {
    // Sessions store timestamps as RFC3339 strings (e.g.
    // "2026-07-28T12:34:56.789Z"). `chrono` parses this directly.
    chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.timestamp())
}

/// Scan one session file, collecting `Edit`/`Write` candidates for `target_path`
/// (already normalized).
fn collect_candidates_from_session(
    path: &Path,
    format_name: Option<&str>,
    target_path: &str,
    out: &mut Vec<EditCandidate>,
) {
    let session = if let Some(fmt) = format_name {
        match parse_session_with_format(path, fmt) {
            Ok(s) => s,
            Err(_) => return,
        }
    } else {
        match parse_session(path) {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    let session_id = session.metadata.session_id.clone().unwrap_or_else(|| {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into()
    });

    for turn in &session.turns {
        for msg in &turn.messages {
            if msg.role != Role::Assistant {
                continue;
            }
            let ts = msg.timestamp.as_deref().and_then(parse_session_timestamp);
            for block in &msg.content {
                if let ContentBlock::ToolUse { name, input, .. } = block {
                    let Some(file_path) = input.get("file_path").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    if normalize_path(file_path) != target_path {
                        continue;
                    }
                    if let Some((tool, old_string, new_string)) = edit_strings(name, input) {
                        out.push(EditCandidate {
                            session_id: session_id.clone(),
                            session_path: path.to_path_buf(),
                            agent_id: session.agent_id.clone(),
                            subagent_type: session.subagent_type.clone(),
                            timestamp: ts,
                            tool,
                            old_string,
                            new_string,
                        });
                    }
                }
            }
        }
    }
}

/// Options controlling which sessions are scanned for candidate edits.
#[derive(Default)]
pub struct BlameSessionFilter<'a> {
    pub format_name: Option<&'a str>,
    pub grep: Option<&'a str>,
    pub days: Option<u32>,
    pub since: Option<&'a str>,
    pub until: Option<&'a str>,
    pub project: Option<&'a Path>,
    pub all_projects: bool,
    pub mode: SessionMode,
    pub agent_type: Option<&'a str>,
    pub session_limit: usize,
}

fn collect_all_candidates(
    root: Option<&Path>,
    target_path: &str,
    filter: &BlameSessionFilter,
) -> Result<Vec<EditCandidate>, String> {
    use super::stats::list_all_project_sessions_by_mode;
    use crate::sessions::{FormatRegistry, SessionSource};

    let registry = FormatRegistry::new();
    let source: &dyn SessionSource = match filter.format_name {
        Some(name) => registry
            .get(name)
            .ok_or_else(|| format!("Unknown format: {}", name))?,
        None => registry.get("claude").ok_or_else(|| {
            "Claude format not available (compile with feature = format-claude)".to_string()
        })?,
    };

    let grep_re = filter
        .grep
        .map(|p| regex::Regex::new(p).map_err(|_| format!("Invalid grep pattern: {}", p)))
        .transpose()?;

    let mut sessions = if filter.all_projects {
        list_all_project_sessions_by_mode(source, &filter.mode)
    } else {
        let project = filter.project.or(root);
        list_sessions_by_mode(source, project, &filter.mode)
    };

    let now = std::time::SystemTime::now();
    let since_time = if let Some(d) = filter.days {
        Some(now - std::time::Duration::from_secs(d as u64 * 86400))
    } else if let Some(s) = filter.since {
        Some(parse_date(s).ok_or_else(|| format!("Invalid date format: {} (use YYYY-MM-DD)", s))?)
    } else {
        None
    };
    let until_time = if let Some(u) = filter.until {
        Some(
            parse_date(u).ok_or_else(|| format!("Invalid date format: {} (use YYYY-MM-DD)", u))?
                + std::time::Duration::from_secs(86400),
        )
    } else {
        None
    };
    if let Some(since) = since_time {
        sessions.retain(|s| s.mtime >= since);
    }
    if let Some(until) = until_time {
        sessions.retain(|s| s.mtime <= until);
    }
    if let Some(ref re) = grep_re {
        sessions.retain(|s| session_matches_grep(&s.path, re));
    }
    if let Some(at) = filter.agent_type {
        let at_lower = at.to_lowercase();
        sessions.retain(|s| {
            s.subagent_type
                .as_deref()
                .is_some_and(|t| t.to_lowercase() == at_lower)
        });
    }

    sessions.sort_by_key(|s| std::cmp::Reverse(s.mtime));
    if filter.session_limit > 0 {
        sessions.truncate(filter.session_limit);
    }

    let mut candidates = Vec::new();
    for sf in &sessions {
        collect_candidates_from_session(&sf.path, filter.format_name, target_path, &mut candidates);
    }
    Ok(candidates)
}

/// A commit's file content at itself and at its parent (`C^`).
#[derive(Clone)]
struct CommitContents {
    parent_content: Option<String>,
    commit_content: Option<String>,
}

/// Cache of per-commit file content for a single file, so a commit spanning
/// multiple chunks is only fetched once.
struct CommitContentCache<'a> {
    root: &'a Path,
    path: &'a str,
    cache: HashMap<String, CommitContents>,
}

impl<'a> CommitContentCache<'a> {
    fn new(root: &'a Path, path: &'a str) -> Self {
        Self {
            root,
            path,
            cache: HashMap::new(),
        }
    }

    fn get(&mut self, commit: &str) -> CommitContents {
        if let Some(v) = self.cache.get(commit) {
            return v.clone();
        }
        let parent_ref = format!("{commit}^");
        let v = CommitContents {
            parent_content: GitBackend.show(self.root, &parent_ref, self.path),
            commit_content: GitBackend.show(self.root, commit, self.path),
        };
        self.cache.insert(commit.to_string(), v.clone());
        v
    }
}

/// Attribute one blame chunk to a session edit (or report why it can't be).
fn attribute_chunk(
    chunk_text: &str,
    commit: &str,
    commit_timestamp: Option<i64>,
    candidates: &[EditCandidate],
    contents: &mut CommitContentCache,
) -> Attribution {
    let normalized_chunk = normalize_ws(chunk_text);
    if count_meaningful_chars(&normalized_chunk) < MIN_MEANINGFUL_CHARS {
        return Attribution::Unattributed {
            reason: format!(
                "chunk content too short/generic to confidently attribute (<{MIN_MEANINGFUL_CHARS} meaningful chars)"
            ),
        };
    }

    let Some(commit_ts) = commit_timestamp else {
        return Attribution::Unattributed {
            reason: "commit timestamp unknown (commit not found in walked history)".to_string(),
        };
    };

    let CommitContents {
        parent_content,
        commit_content,
    } = contents.get(commit);
    let Some(commit_content) = commit_content else {
        return Attribution::Unattributed {
            reason:
                "commit content for this file could not be read (binary, deleted, or unreadable)"
                    .to_string(),
        };
    };
    let normalized_commit_content = normalize_ws(&commit_content);
    let normalized_parent_content = parent_content
        .as_deref()
        .map(normalize_ws)
        .unwrap_or_default();

    let mut matches: Vec<&EditCandidate> = Vec::new();
    for cand in candidates {
        let Some(cand_ts) = cand.timestamp else {
            continue; // Unknown-timestamp edits are excluded from matching (reliability > recall).
        };
        if cand_ts > commit_ts {
            continue; // An edit cannot postdate the commit it fed.
        }
        let normalized_new = normalize_ws(&cand.new_string);
        if !normalized_new.contains(&normalized_chunk) {
            continue;
        }
        if !normalized_commit_content.contains(&normalized_new) {
            continue;
        }
        match &cand.old_string {
            Some(old) if !old.is_empty() => {
                if !normalized_parent_content.contains(&normalize_ws(old)) {
                    continue;
                }
            }
            _ => {
                // Write tool (no old_string): require the new content to be
                // (close to) the entire file, not merely a substring of it.
                if normalized_new != normalized_commit_content {
                    continue;
                }
            }
        }
        matches.push(cand);
    }

    match matches.len() {
        0 => Attribution::Unattributed {
            reason: "no session edit's recorded content matches this commit's change to this file"
                .to_string(),
        },
        1 => Attribution::Matched {
            edit: to_edit_ref(matches[0]),
        },
        _ => Attribution::Ambiguous {
            candidates: matches.into_iter().map(to_edit_ref).collect(),
        },
    }
}

fn to_edit_ref(c: &EditCandidate) -> SessionEditRef {
    SessionEditRef {
        session_id: c.session_id.clone(),
        session_path: c.session_path.clone(),
        agent_id: c.agent_id.clone(),
        subagent_type: c.subagent_type.clone(),
        edit_timestamp: c.timestamp,
        tool: c.tool.to_string(),
    }
}

/// Build a `sessions blame` report for `path` (repo-relative), optionally
/// restricted to `[start_line, end_line]` (1-based, inclusive).
pub fn build_blame_report(
    root: &Path,
    path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
    filter: &BlameSessionFilter,
) -> Result<SessionsBlameReport, String> {
    if !GitBackend.repo_exists(root) {
        return Err(format!("Not a git repository: {}", root.display()));
    }

    let target_path = normalize_path(path);

    let file_content = std::fs::read_to_string(root.join(path))
        .map_err(|e| format!("Failed to read {}: {e}", path))?;
    let lines: Vec<&str> = file_content.lines().collect();

    let blame_lines = GitBackend.blame(root, path).ok_or_else(|| {
        format!(
            "git blame failed for {} (not tracked, or repo unreadable)",
            path
        )
    })?;

    let start = start_line.unwrap_or(1);
    let end = end_line.unwrap_or(blame_lines.len());
    let scoped: Vec<_> = blame_lines
        .into_iter()
        .filter(|l| l.line >= start && l.line <= end)
        .collect();

    if scoped.is_empty() {
        return Ok(SessionsBlameReport {
            path: target_path,
            chunks: Vec::new(),
            total_lines: 0,
            matched_lines: 0,
            ambiguous_lines: 0,
            unattributed_lines: 0,
        });
    }

    // Group consecutive same-commit lines into chunks.
    let mut raw_chunks: Vec<(usize, usize, String)> = Vec::new(); // (start, end, commit)
    for bl in &scoped {
        if let Some(last) = raw_chunks.last_mut()
            && last.1 + 1 == bl.line
            && last.2 == bl.commit.0
        {
            last.1 = bl.line;
            continue;
        }
        raw_chunks.push((bl.line, bl.line, bl.commit.0.clone()));
    }
    // Keep author info per line for the report (first line of each chunk's author).
    let author_by_line: HashMap<usize, (String, String)> = scoped
        .iter()
        .map(|l| (l.line, (l.author_name.clone(), l.author_email.clone())))
        .collect();

    let commit_timestamps: HashMap<String, i64> = GitBackend
        .log_timestamps(root)
        .map(|entries| entries.into_iter().map(|e| (e.hash, e.timestamp)).collect())
        .unwrap_or_default();

    let candidates = collect_all_candidates(Some(root), &target_path, filter)?;

    let mut contents = CommitContentCache::new(root, path);
    let mut chunks = Vec::new();
    let mut matched_lines = 0usize;
    let mut ambiguous_lines = 0usize;
    let mut unattributed_lines = 0usize;
    let total_lines = scoped.len();

    for (start_line, end_line, commit) in raw_chunks {
        let chunk_text = lines
            .get(start_line.saturating_sub(1)..end_line.min(lines.len()))
            .map(|s| s.join("\n"))
            .unwrap_or_default();
        let commit_timestamp = commit_timestamps.get(&commit).copied();
        let attribution = attribute_chunk(
            &chunk_text,
            &commit,
            commit_timestamp,
            &candidates,
            &mut contents,
        );

        let line_count = end_line - start_line + 1;
        match &attribution {
            Attribution::Matched { .. } => matched_lines += line_count,
            Attribution::Ambiguous { .. } => ambiguous_lines += line_count,
            Attribution::Unattributed { .. } => unattributed_lines += line_count,
        }

        let (author_name, author_email) = author_by_line
            .get(&start_line)
            .cloned()
            .unwrap_or_else(|| ("Unknown".to_string(), String::new()));
        let commit_short = commit.chars().take(7).collect();
        chunks.push(BlameChunk {
            start_line,
            end_line,
            commit: commit.clone(),
            commit_short,
            author_name,
            author_email,
            commit_timestamp,
            attribution,
        });
    }

    Ok(SessionsBlameReport {
        path: target_path,
        chunks,
        total_lines,
        matched_lines,
        ambiguous_lines,
        unattributed_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ws_tolerates_trailing_whitespace_and_crlf() {
        assert_eq!(normalize_ws("a  \r\nb\t\n"), "a\nb");
        assert_eq!(normalize_ws("a\nb"), "a\nb");
    }

    #[test]
    fn count_meaningful_chars_ignores_whitespace() {
        assert_eq!(count_meaningful_chars("  a b \n c  "), 3);
        assert_eq!(count_meaningful_chars("   \n\t  "), 0);
    }

    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .status()
            .expect("failed to run git");
        assert!(status.success(), "git {:?} failed", args);
    }

    fn commit_at(dir: &Path, message: &str, iso_date: &str) {
        let status = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .env("GIT_AUTHOR_DATE", iso_date)
            .env("GIT_COMMITTER_DATE", iso_date)
            .status()
            .expect("failed to run git commit");
        assert!(status.success(), "git commit failed");
    }

    /// Replicates `ClaudeCodeFormat`'s primary path encoding (`normalize-chat-sessions`'s
    /// `claude_code.rs::path_to_claude_dir`), so the fabricated session directory is where
    /// `sessions_root(Some(repo_path))` will actually look, given `CLAUDE_SESSIONS_DIR`.
    fn claude_project_dir(sessions_dir: &Path, repo_path: &Path) -> PathBuf {
        let raw = repo_path.to_string_lossy();
        let path_str = raw.trim_end_matches('/').replace('/', "-");
        sessions_dir.join(format!("-{}", path_str.trim_start_matches('-')))
    }

    fn write_session(path: &Path, session_id: &str, timestamp: &str, tool_json: &str) {
        let content = format!(
            concat!(
                "{{\"type\":\"summary\",\"sessionId\":\"{session_id}\",\"timestamp\":\"{timestamp}\"}}\n",
                "{{\"type\":\"assistant\",\"timestamp\":\"{timestamp}\",\"message\":{{\"model\":\"test\",\"content\":[{tool_json}]}}}}\n",
            ),
            session_id = session_id,
            timestamp = timestamp,
            tool_json = tool_json,
        );
        std::fs::write(path, content).unwrap();
    }

    fn edit_tool_json(id: &str, file_path: &str, old_string: &str, new_string: &str) -> String {
        format!(
            r#"{{"type":"tool_use","id":"{id}","name":"Edit","input":{{"file_path":{file_path:?},"old_string":{old_string:?},"new_string":{new_string:?}}}}}"#,
        )
    }

    /// End-to-end fixture: a real git repo with three commits, and fabricated
    /// Claude Code session logs whose Edit tool calls' recorded old/new text is
    /// checked against ground truth we control. Exercises all three outcomes:
    /// a confident match, an ambiguous match (two sessions with byte-identical
    /// edits), and an unattributed chunk (content predates any session).
    #[test]
    fn build_blame_report_attributes_by_content() {
        let repo = tempfile::tempdir().unwrap();
        let repo_path = repo.path();
        let sessions_dir = tempfile::tempdir().unwrap();

        git(repo_path, &["init", "-q"]);
        git(repo_path, &["config", "commit.gpgsign", "false"]);

        // Commit 1 (T0): no session involved — "manual" content.
        std::fs::write(repo_path.join("file.txt"), "one\ntwo\nthreewordline\n").unwrap();
        git(repo_path, &["add", "file.txt"]);
        commit_at(repo_path, "initial", "2024-01-01T00:00:00+00:00");

        // Session S1 (T1, before commit 2): two -> two + TWOPOINTFIVE.
        let proj_dir = claude_project_dir(sessions_dir.path(), repo_path);
        std::fs::create_dir_all(&proj_dir).unwrap();
        write_session(
            &proj_dir.join("s1.jsonl"),
            "s1-session",
            "2024-01-02T00:00:00Z",
            &edit_tool_json("t1", "file.txt", "two\n", "two\nTWOPOINTFIVE\n"),
        );

        // Commit 2 (T2): applies S1's edit.
        std::fs::write(
            repo_path.join("file.txt"),
            "one\ntwo\nTWOPOINTFIVE\nthreewordline\n",
        )
        .unwrap();
        git(repo_path, &["add", "file.txt"]);
        commit_at(repo_path, "apply s1", "2024-01-03T00:00:00+00:00");

        // Two sessions (S2, S3) both recording the *same* edit before commit 3 —
        // deliberately indistinguishable from content alone.
        write_session(
            &proj_dir.join("s2.jsonl"),
            "s2-session",
            "2024-01-04T00:00:00Z",
            &edit_tool_json(
                "t2",
                "file.txt",
                "threewordline\n",
                "threewordline\nFOURFIVE\n",
            ),
        );
        write_session(
            &proj_dir.join("s3.jsonl"),
            "s3-session",
            "2024-01-04T00:10:00Z",
            &edit_tool_json(
                "t3",
                "file.txt",
                "threewordline\n",
                "threewordline\nFOURFIVE\n",
            ),
        );

        // Commit 3 (T3): applies the (ambiguous) edit.
        std::fs::write(
            repo_path.join("file.txt"),
            "one\ntwo\nTWOPOINTFIVE\nthreewordline\nFOURFIVE\n",
        )
        .unwrap();
        git(repo_path, &["add", "file.txt"]);
        commit_at(repo_path, "apply s2/s3", "2024-01-05T00:00:00+00:00");

        // SAFETY: single-threaded within this test; no other test in this
        // process reads/writes CLAUDE_SESSIONS_DIR.
        unsafe {
            std::env::set_var("CLAUDE_SESSIONS_DIR", sessions_dir.path());
        }
        let filter = BlameSessionFilter::default();
        let report = build_blame_report(repo_path, "file.txt", None, None, &filter)
            .expect("build_blame_report failed");
        unsafe {
            std::env::remove_var("CLAUDE_SESSIONS_DIR");
        }

        assert_eq!(
            report.chunks.len(),
            4,
            "expected 4 blame chunks: {:#?}",
            report.chunks
        );

        // Chunk 1: lines 1-2 ("one", "two") — commit 1, no session existed yet.
        assert_eq!(report.chunks[0].start_line, 1);
        assert_eq!(report.chunks[0].end_line, 2);
        assert!(
            matches!(
                report.chunks[0].attribution,
                Attribution::Unattributed { .. }
            ),
            "expected unattributed, got {:?}",
            report.chunks[0].attribution
        );

        // Chunk 2: line 3 ("TWOPOINTFIVE") — commit 2, confidently matched to S1.
        assert_eq!(report.chunks[1].start_line, 3);
        assert_eq!(report.chunks[1].end_line, 3);
        match &report.chunks[1].attribution {
            Attribution::Matched { edit } => assert_eq!(edit.session_id, "s1-session"),
            other => panic!("expected matched to s1-session, got {other:?}"),
        }

        // Chunk 3: line 4 ("threewordline") — commit 1 again, still no session.
        assert_eq!(report.chunks[2].start_line, 4);
        assert_eq!(report.chunks[2].end_line, 4);
        assert!(matches!(
            report.chunks[2].attribution,
            Attribution::Unattributed { .. }
        ));

        // Chunk 4: line 5 ("FOURFIVE") — commit 3, ambiguous between S2 and S3.
        assert_eq!(report.chunks[3].start_line, 5);
        assert_eq!(report.chunks[3].end_line, 5);
        match &report.chunks[3].attribution {
            Attribution::Ambiguous { candidates } => {
                let mut ids: Vec<&str> = candidates.iter().map(|c| c.session_id.as_str()).collect();
                ids.sort();
                assert_eq!(ids, vec!["s2-session", "s3-session"]);
            }
            other => panic!("expected ambiguous between s2/s3, got {other:?}"),
        }

        assert_eq!(report.matched_lines, 1);
        assert_eq!(report.ambiguous_lines, 1);
        assert_eq!(report.unattributed_lines, 3);
        assert_eq!(report.total_lines, 5);
    }
}
