//! Query fixture tests for capnp.
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

const CAPNP_SAMPLE: &str = include_str!("fixtures/capnp/sample.capnp");

#[test]
fn capnp_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping capnp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "capnp",
        CAPNP_SAMPLE,
        &["# A point in 2D space"],
    );
}

// =============================================================================
// Batch 9: capnp + wit query-testing sweep
// =============================================================================
//
// capnp.imports.scm bug found: the query matched a bare `(import ...)` node
// type that node-types.json lists but the parser never actually produces —
// every real `using X = import "path";` parses as
// `using_directive > import_using`, which the query never handled. The old
// query matched ZERO imports on real capnp source (verified: 0 matches on
// the existing sample.capnp fixture, which has a real `using Cxx = import
// "/capnp/c++.capnp";` line). Fixed to match `import_using` with its
// `type_identifier`/`import_path` children, adding an `@import.alias`
// capture for the bound name.
//
// wit.imports.scm bugs found:
// - `import foo;` inside a `world` body (`import_item` wrapping a
//   `use_path`) was entirely unhandled — only interface/world-scoped
//   `use x.{...}` (`use_item`) was matched. Verified 0 matches on
//   `import_item` before the fix even though it's a common world-body
//   idiom (see sample.wit's `import types;`).
// - Top-level `use foo:bar/baz;` (`toplevel_use_item`, no braced name list)
//   was entirely unhandled.
// - `include foo:bar/pkg;` (`include_item`, merging another world) was
//   entirely unhandled.
// - The `@import.path` capture on `use_item` captured the *entire*
//   statement text (e.g. `"use types.{resource-handle, status,
//   result-value};"`) instead of just the path, contradicting its own
//   header comment ("the interface path being used") and every other
//   language's imports.scm convention (path capture = just the path node).
//   Fixed to capture the `use_path` child specifically, and added the
//   `@import.name` capture the header comment already documented but never
//   implemented.
// - `export foo;` (`export_item`) is intentionally NOT matched: exporting a
//   locally-defined interface has no external path — it isn't a dependency
//   on anything, so it isn't an import. Verified via `normalize syntax ast`
//   that `export_item` and `import_item` are structurally near-identical
//   (both wrap `use_path` or `extern_type`), so this is a deliberate
//   semantic exclusion, not an oversight — covered by a negative test below.
// - `import name: func(...);` (`import_item` wrapping an inline
//   `extern_type` rather than a `use_path`) is intentionally NOT matched:
//   it declares a local signature, not a reference to external code —
//   covered by a negative test below.
//
// wit.decorations.scm bug found: `block_comment` (`/* ... */` and
// `/** ... */`) was never matched at all. `doc_comment` only ever appears
// nested as the "doc" field of `line_comment`/`block_comment` (verified:
// nothing else in node-types.json references `doc_comment`), so:
// - plain `/* ... */` (no nested doc_comment) was silently dropped
//   entirely — no decoration capture at all.
// - `/** ... */` matched only via the nested `doc_comment`, producing a
//   truncated capture missing the `/**`/`*/` delimiters (inconsistent with
//   every other decoration in the codebase, which captures the whole
//   comment node).
// - `///` doc line comments were DOUBLE-counted: both the outer
//   `line_comment` wrapper and the nested `doc_comment` field matched,
//   producing two @decoration captures for one comment.
// Fixed by querying only the wrapper node types (`line_comment`,
// `block_comment`) and dropping the bare `(doc_comment)` pattern — the
// wrapper capture already includes the nested doc_comment's text.
//
// No new query purposes were authored (imports/decorations already existed
// for both languages; all changes are field-completeness/correctness fixes
// to those two existing files per language, verified against
// node-types.json and real parse output via `normalize syntax ast` /
// `normalize syntax query`).

const CAPNP_VARIANTS: &str = include_str!("fixtures/capnp/variants.capnp");

/// Dimension 4 (real-world): capnp.imports.scm finds the real
/// `using Cxx = import "/capnp/c++.capnp";` import in the existing sample
/// fixture, with both the alias and the path captured as distinct node
/// kinds (not just distinct text).
#[test]
fn capnp_imports_finds_using_import_on_sample() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping capnp_imports_sample: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("capnp").ok() else {
        eprintln!("Skipping capnp_imports_sample: capnp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("capnp")
        .expect("capnp imports query missing");
    let caps = collect_captures_full(&lang, CAPNP_SAMPLE, &query_str);

    let path = caps
        .iter()
        .find(|(cn, kind, _, _)| cn == "import.path" && kind == "import_path")
        .unwrap_or_else(|| {
            panic!("expected an import_path-kind @import.path capture, got: {caps:?}")
        });
    assert!(
        path.2.contains("/capnp/c++.capnp"),
        "expected the c++.capnp import path, got: {path:?}"
    );

    let alias = caps
        .iter()
        .find(|(cn, kind, _, _)| cn == "import.alias" && kind == "type_identifier")
        .unwrap_or_else(|| {
            panic!("expected a type_identifier-kind @import.alias capture, got: {caps:?}")
        });
    assert_eq!(alias.2, "Cxx", "expected alias 'Cxx', got: {caps:?}");
}

/// Dimension 2 (completeness): every `import_using` in the variants fixture
/// produces exactly one @import.path/@import.alias pair, and the
/// near-miss `replace_using` (`using MyAlias = UInt32;` — a type alias, not
/// an import) contributes zero captures.
#[test]
fn capnp_imports_completeness_and_negative_replace_using() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping capnp_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("capnp").ok() else {
        eprintln!("Skipping capnp_imports_completeness: capnp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("capnp")
        .expect("capnp imports query missing");
    let caps = collect_captures_full(&lang, CAPNP_VARIANTS, &query_str);

    let aliases: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.alias")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert_eq!(
        aliases,
        vec!["Cxx", "Other"],
        "expected exactly the two import_using aliases in declaration order, got: {aliases:?}"
    );

    let paths: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.path")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        paths.iter().any(|p| p.contains("/capnp/c++.capnp")),
        "expected the c++.capnp path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("other/thing.capnp")),
        "expected the other/thing.capnp path, got: {paths:?}"
    );

    // NEGATIVE: `using MyAlias = UInt32;` (replace_using) must not appear
    // anywhere in the captures — neither as a path nor as an alias.
    assert!(
        !aliases.contains(&"MyAlias"),
        "type alias 'MyAlias' (replace_using, not an import) must not be captured as \
         @import.alias, got: {aliases:?}"
    );
    assert!(
        caps.iter().all(|(_, _, t, _)| !t.contains("UInt32")),
        "replace_using's target type 'UInt32' must never appear in any import capture, \
         got: {caps:?}"
    );
}

/// capnp.decorations.scm: `comment` is the grammar's sole comment node type
/// (verified via node-types.json — no doc-comment/pragma variant exists in
/// this grammar), so a plain contains-check on the existing sample is
/// sufficient; this test additionally confirms the variants fixture's
/// header comments are picked up too, guarding against a future regression
/// that narrows the pattern.
#[test]
fn capnp_decorations_finds_comment_on_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping capnp_decorations_variants: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "capnp",
        CAPNP_VARIANTS,
        &["# POSITIVE: import_using — basic form."],
    );
}
