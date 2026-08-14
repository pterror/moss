//! Shared helpers for the per-language query fixture tests.
//!
//! Split out of the former monolithic `query_fixtures.rs`. Each
//! `tests/query_<lang>.rs` binary pulls this in via `mod common;`, so any given
//! binary uses only a subset of these helpers — hence the crate-level
//! `allow(dead_code)`.

#![allow(dead_code)]

use normalize_languages::GrammarLoader;
use std::path::PathBuf;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the grammar search path if `target/grammars/` exists relative to the
/// workspace root, otherwise return `None` to signal the test should be skipped.
pub fn grammar_dir() -> Option<PathBuf> {
    // Integration tests run with cwd = crate root; grammars live at workspace root.
    let crate_root = std::env::current_dir().unwrap();
    let workspace_root = crate_root
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())?;
    let dir = workspace_root.join("target/grammars");
    if dir.exists() { Some(dir) } else { None }
}

/// Like [`grammar_dir`], but panics when `NORMALIZE_REQUIRE_GRAMMARS` is set and
/// the grammar directory is missing.  Use in decoration tests (and other new tests)
/// so that CI — which sets the env var — catches silent skips.
pub fn require_grammar_dir() -> Option<PathBuf> {
    let dir = grammar_dir();
    if dir.is_none() && std::env::var("NORMALIZE_REQUIRE_GRAMMARS").is_ok() {
        panic!(
            "NORMALIZE_REQUIRE_GRAMMARS is set but target/grammars/ does not exist \
             — run `cargo xtask build-grammars` first"
        );
    }
    dir
}

/// Parse `source` with `lang`, run `query_str` against it, and collect all
/// captures whose name starts with `capture_prefix` into a `Vec<String>`.
pub fn collect_captures(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
    capture_prefix: &str,
) -> Vec<String> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut results = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            if cap_name.starts_with(capture_prefix) {
                let text = cap.node.utf8_text(source_bytes).unwrap_or("").to_string();
                results.push(text);
            }
        }
    }
    results
}

/// Like [`collect_captures`], but returns `(capture_name, node_kind, text, line)`
/// for every capture (regardless of prefix). Use this when a test needs to
/// assert on capture *kind* (extraction depth), not just capture text — the
/// same text can legitimately come from different node kinds (e.g. a
/// `type_identifier` named "new" vs an `identifier` named "new"), and a test
/// that only checks text can't tell them apart.
pub fn collect_captures_full(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
) -> Vec<(String, String, String, usize)> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut results = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize].to_string();
            let kind = cap.node.kind().to_string();
            let text = cap.node.utf8_text(source_bytes).unwrap_or("").to_string();
            let line = cap.node.start_position().row + 1;
            results.push((cap_name, kind, text, line));
        }
    }
    results
}

/// Collect `(tag_kind, name_text)` pairs from a tags-style query: `tag_kind`
/// is whichever `@definition.*`/`@reference.*` capture co-occurs with `@name`
/// in the same match. Use this instead of [`collect_captures_full`] when the
/// container capture (e.g. `@reference.class`) spans a much larger node (the
/// whole `new Foo()`/`extends Foo` expression) than the `@name` capture that
/// actually holds the identifier text.
pub fn collect_tag_pairs(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
) -> Vec<(String, String)> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut pairs = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut name = None;
        let mut tag_kind = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(source_bytes).unwrap_or("");
            if cap_name == "name" {
                name = Some(text.to_string());
            } else if cap_name.starts_with("definition.") || cap_name.starts_with("reference.") {
                tag_kind = Some(cap_name.to_string());
            }
        }
        if let (Some(n), Some(k)) = (name, tag_kind) {
            pairs.push((k, n));
        }
    }
    pairs
}

/// Run `query_str` against `source` and, for every match that carries a
/// capture named `anchor_capture_name` (e.g. `"reference.class"`,
/// `"definition.module"`), return the `(kind, text)` of that match's `@name`
/// capture. Use this instead of naively filtering `collect_captures_full` by
/// the anchor capture's own name when the anchor is attached to a *container*
/// node (e.g. `new_expression`, `extends_clause`, `module`/`internal_module`)
/// rather than to the field-variant node itself — the anchor's own `kind`
/// would otherwise always report the container type, never the variant.
pub fn tags_matches_by_kind(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
    anchor_capture_name: &str,
) -> Vec<(String, String)> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut results = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut has_anchor = false;
        let mut name: Option<(String, String)> = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            if cap_name == anchor_capture_name {
                has_anchor = true;
            } else if cap_name == "name" {
                let kind = cap.node.kind().to_string();
                let text = cap.node.utf8_text(source_bytes).unwrap_or("").to_string();
                name = Some((kind, text));
            }
        }
        if has_anchor && let Some(n) = name {
            results.push(n);
        }
    }
    results
}

// ---------------------------------------------------------------------------
// Decorations tests
// ---------------------------------------------------------------------------

pub fn assert_decorations_contains(
    loader: &GrammarLoader,
    grammar: &str,
    sample: &str,
    expected: &[&str],
) {
    let Some(lang) = loader.get(grammar).ok() else {
        if std::env::var("NORMALIZE_REQUIRE_GRAMMARS").is_ok() {
            panic!(
                "{grammar}_decorations: grammar .so not found \
                 — set NORMALIZE_REQUIRE_GRAMMARS only when grammars are built"
            );
        }
        eprintln!("Skipping {grammar}_decorations: grammar .so not found");
        return;
    };
    let query_str = loader
        .get_decorations(grammar)
        .unwrap_or_else(|| panic!("{grammar} decorations query missing"));
    let captures = collect_captures(&lang, sample, &query_str, "decoration");
    assert!(
        !captures.is_empty(),
        "expected at least one @decoration capture for {grammar}, got none"
    );
    for exp in expected {
        assert!(
            captures.iter().any(|c| c.contains(exp)),
            "expected capture containing {exp:?} for {grammar}, got: {captures:?}"
        );
    }
}
