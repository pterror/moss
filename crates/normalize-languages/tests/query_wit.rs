//! Query fixture tests for wit.
//!
//! Split out of the former monolithic `query_fixtures.rs`; see
//! `tests/common/mod.rs` for the shared helpers.
//!
//! These tests require compiled grammar `.so` files in `target/grammars/`.
//! Build them with `cargo xtask build-grammars`. Without grammars present the
//! tests skip gracefully.

mod common;

#[allow(unused_imports)]
use common::*;
use normalize_languages::GrammarLoader;

const WIT_SAMPLE: &str = include_str!("fixtures/wit/sample.wit");

#[test]
fn wit_decorations_finds_doc_comment_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping wit_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "wit",
        WIT_SAMPLE,
        &[
            "/// Types and functions for working with text",
            "/// A handle to an open resource",
        ],
    );
}

const WIT_VARIANTS: &str = include_str!("fixtures/wit/variants.wit");

/// Dimension 4 (real-world): wit.imports.scm finds all three import-like
/// statement shapes present in the existing sample fixture — the
/// interface-scoped `use types.{...}` (`use_item`) and the world-body
/// `import types;` (`import_item`) — with @import.path holding just the
/// path (not the whole statement).
#[test]
fn wit_imports_finds_use_and_import_on_sample() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping wit_imports_sample: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("wit").ok() else {
        eprintln!("Skipping wit_imports_sample: wit grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("wit")
        .expect("wit imports query missing");
    let caps = collect_captures_full(&lang, WIT_SAMPLE, &query_str);

    let paths: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.path")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // 3 matches from use_item (one per imported name) + 1 from import_item
    // (`import types;`) — all resolve to the same path text "types", but
    // crucially each is just "types", never the whole statement text.
    assert_eq!(
        paths.len(),
        4,
        "expected 4 @import.path captures (3 use_item names + 1 import_item), got: {paths:?}"
    );
    assert!(
        paths.iter().all(|p| *p == "types"),
        "expected every @import.path to be just 'types' (not the whole 'use \
         types.{{...}};' statement), got: {paths:?}"
    );

    let names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    for expected in ["resource-handle", "status", "result-value"] {
        assert!(
            names.contains(&expected),
            "expected @import.name to include '{expected}', got: {names:?}"
        );
    }

    // world body `import types;` (import_item) must also be captured.
    let import_anchors: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        import_anchors.iter().any(|t| t.contains("import types;")),
        "expected the world-body 'import types;' statement to be captured, got: {import_anchors:?}"
    );
    // world body `export operations;` must NOT be captured as an import.
    assert!(
        import_anchors.iter().all(|t| !t.contains("export")),
        "'export operations;' must never be captured as an import, got: {import_anchors:?}"
    );
}

/// Dimension 2 (completeness): every import-like node kind
/// (`use_item`/`toplevel_use_item`/`import_item`/`include_item`, with and
/// without a `with { ... }` rename clause) produces the expected
/// @import.path/@import.name/@import.alias captures.
#[test]
fn wit_imports_completeness_all_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping wit_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("wit").ok() else {
        eprintln!("Skipping wit_imports_completeness: wit grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("wit")
        .expect("wit imports query missing");
    let caps = collect_captures_full(&lang, WIT_VARIANTS, &query_str);

    let paths: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.path")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // toplevel_use_item (bare + aliased)
    assert!(
        paths.contains(&"pkg:dep/iface-a"),
        "expected bare toplevel_use_item path, got: {paths:?}"
    );
    assert!(
        paths.contains(&"pkg:dep/iface-b"),
        "expected aliased toplevel_use_item path, got: {paths:?}"
    );
    // use_item (interface-scoped)
    assert!(
        paths.contains(&"types"),
        "expected use_item path 'types', got: {paths:?}"
    );
    // import_item (world-body, real reference)
    assert!(
        paths.contains(&"consumer"),
        "expected import_item path 'consumer', got: {paths:?}"
    );
    // include_item, bare and with-clause forms
    assert!(
        paths
            .iter()
            .filter(|p| **p == "pkg:dep/other-world")
            .count()
            >= 2,
        "expected 'pkg:dep/other-world' from both the bare and with-clause include_item, \
         got: {paths:?}"
    );

    let aliases: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.alias")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert_eq!(
        aliases,
        vec!["renamed-b"],
        "expected exactly one @import.alias ('renamed-b' from the aliased toplevel_use_item), \
         got: {aliases:?}"
    );

    let names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        names.contains(&"resource-handle"),
        "expected use_item name 'resource-handle', got: {names:?}"
    );
    assert!(
        names.contains(&"status"),
        "expected use_item name 'status', got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("thing as renamed-thing")),
        "expected include_names_item 'thing as renamed-thing' from the with-clause \
         include_item, got: {names:?}"
    );
}

/// Negative cases: `export foo;` (export_item) and the inline-signature
/// form of `import name: func(...);` (import_item wrapping an extern_type,
/// not a use_path) must never contribute an @import/@import.path capture —
/// neither is a reference to external code.
#[test]
fn wit_imports_negative_export_and_inline_extern_type() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping wit_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("wit").ok() else {
        eprintln!("Skipping wit_imports_negative: wit grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("wit")
        .expect("wit imports query missing");
    let caps = collect_captures_full(&lang, WIT_VARIANTS, &query_str);

    assert!(
        caps.iter()
            .all(|(_, _, t, _)| !t.contains("export consumer")),
        "'export consumer;' must never be captured, got: {caps:?}"
    );
    assert!(
        caps.iter()
            .all(|(_, _, t, _)| !t.contains("direct-fn: func")),
        "'import direct-fn: func(...)' (inline extern_type, no use_path) must never be \
         captured, got: {caps:?}"
    );
    // Sanity: the negative cases share a file with real positives, so an
    // empty-captures false pass is ruled out.
    assert!(
        !caps.is_empty(),
        "expected non-empty captures overall on the variants fixture (positives exist \
         alongside the negatives)"
    );
}

/// wit.decorations.scm dimension 2/3 (completeness + extraction depth):
/// - plain line comments (`//`) and doc line comments (`///`) each produce
///   exactly one @decoration capture, of kind `line_comment` (not a
///   duplicate via the nested `doc_comment` field).
/// - plain block comments (`/* */`) and doc block comments (`/** */`) each
///   produce exactly one @decoration capture, of kind `block_comment`,
///   including the delimiters (not truncated to the inner doc_comment
///   text).
#[test]
fn wit_decorations_completeness_line_and_block_comments() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping wit_decorations_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("wit").ok() else {
        eprintln!("Skipping wit_decorations_completeness: wit grammar .so not found");
        return;
    };
    let query_str = loader
        .get_decorations("wit")
        .expect("wit decorations query missing");
    let caps = collect_captures_full(&lang, WIT_VARIANTS, &query_str);

    // Exactly one capture for the doc line comment, and it must be the
    // line_comment wrapper (not also a separate nested doc_comment capture).
    let doc_line_hits: Vec<&(String, String, String, usize)> = caps
        .iter()
        .filter(|(_, _, t, _)| t.contains("doc line comment"))
        .collect();
    assert_eq!(
        doc_line_hits.len(),
        1,
        "expected exactly 1 capture for the '///' doc line comment (was double-counted \
         via the nested doc_comment field before the fix), got: {doc_line_hits:?}"
    );
    assert_eq!(
        doc_line_hits[0].1, "line_comment",
        "expected the doc line comment capture to be kind 'line_comment', got: {doc_line_hits:?}"
    );

    // Plain block comment: must be present (was dropped entirely before the
    // fix) and must include the delimiters.
    let plain_block = caps
        .iter()
        .find(|(_, _, t, _)| t.contains("plain block comment"))
        .unwrap_or_else(|| {
            panic!("expected a capture for the plain '/* */' block comment, got: {caps:?}")
        });
    assert_eq!(plain_block.1, "block_comment");
    assert!(
        plain_block.2.starts_with("/*") && plain_block.2.ends_with("*/"),
        "expected the block comment capture to include its delimiters, got: {plain_block:?}"
    );

    // Doc block comment: exactly one capture, kind block_comment, including
    // delimiters (not truncated to the inner doc_comment text).
    let doc_block_hits: Vec<&(String, String, String, usize)> = caps
        .iter()
        .filter(|(_, _, t, _)| t.contains("doc block comment"))
        .collect();
    assert_eq!(
        doc_block_hits.len(),
        1,
        "expected exactly 1 capture for the '/** */' doc block comment, got: {doc_block_hits:?}"
    );
    assert_eq!(doc_block_hits[0].1, "block_comment");
    assert!(
        doc_block_hits[0].2.starts_with("/**"),
        "expected the doc block comment capture to include its '/**' delimiter (not be \
         truncated to just the inner doc_comment text), got: {doc_block_hits:?}"
    );
}
