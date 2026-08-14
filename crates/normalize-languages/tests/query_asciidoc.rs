//! Query fixture tests for asciidoc.
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

// ==== BATCH 10: asciidoc / batch ====
//
// asciidoc.imports.scm: fixed a real bug — the query required a
// `block_macro_attr` child to match at all, so bare `include::path[]`
// (no `[attrs]`) produced zero captures, and when attributes WERE present
// it captured the attribute text (e.g. "lines=1..10") as @import.path
// instead of the actual path. The path lives in the `target` node
// (node-types.json: required, exactly one child of `block_macro`),
// unconditionally present regardless of attributes. Verified via
// `normalize syntax ast`/`normalize syntax query` against a probe file.
//
// batch.{cfg,complexity}.scm: no capture-output bug fix (the grammar is too
// minimal to do better), but the doc comments were inaccurate/incomplete —
// they didn't disclose that `goto :label` / `call :label` each emit a
// spurious extra `function_definition` sibling for the target,
// indistinguishable at the query level from a genuine label definition
// (confirmed via `normalize syntax ast`: `call` isn't even a recognized
// keyword in this grammar, it parses inside an ERROR node). Comments
// updated to document this precisely; @nesting output unchanged (already
// the best available approximation).

const ASCIIDOC_SAMPLE: &str = include_str!("fixtures/asciidoc/sample.adoc");

const ASCIIDOC_VARIANTS: &str = include_str!("fixtures/asciidoc/variants.adoc");

// --- asciidoc imports: dimension 4 (real-world sample) ---------------------

#[test]
fn asciidoc_imports_finds_sample_includes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping asciidoc_imports_finds_sample_includes: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("asciidoc").ok() else {
        eprintln!(
            "Skipping asciidoc_imports_finds_sample_includes: asciidoc grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("asciidoc")
        .expect("asciidoc imports query missing");

    let captures = collect_captures_full(&lang, ASCIIDOC_SAMPLE, &query_str);
    let paths: Vec<&str> = captures
        .iter()
        .filter(|(name, ..)| name == "import.path")
        .map(|(_, _, text, _)| text.as_str())
        .collect();

    // Bare include (no attributes) — the case the original query dropped
    // entirely.
    assert!(
        paths.contains(&"chapters/chapter1.adoc"),
        "expected bare include path in asciidoc imports, got: {paths:?}"
    );
    // Include with an attribute list — the case the original query
    // mis-captured (attribute text instead of path).
    assert!(
        paths.contains(&"chapters/chapter2.adoc"),
        "expected attributed include path in asciidoc imports, got: {paths:?}"
    );
    // Include whose target contains an unexpanded document-attribute
    // reference.
    assert!(
        paths.contains(&"{docdir}/appendix/notes.adoc"),
        "expected {{docdir}}-prefixed include path in asciidoc imports, got: {paths:?}"
    );
    assert_eq!(
        paths.len(),
        3,
        "expected exactly 3 import.path captures (the image:: macro must not \
         match), got {}: {paths:?}",
        paths.len()
    );

    // Every @import.path capture must be the `target` node kind, not
    // `block_macro_attr` (extraction-depth check: kind, not just text).
    for (name, kind, text, _line) in &captures {
        if name == "import.path" {
            assert_eq!(
                kind, "target",
                "expected @import.path capture to be a `target` node, got kind \
                 '{kind}' for text '{text}'"
            );
        }
    }
}

// --- asciidoc imports: dimension 2 + 3 (completeness matrix) ---------------

#[test]
fn asciidoc_imports_completeness_target_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping asciidoc_imports_completeness_target_variants: run \
             `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("asciidoc").ok() else {
        eprintln!(
            "Skipping asciidoc_imports_completeness_target_variants: asciidoc \
             grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("asciidoc")
        .expect("asciidoc imports query missing");

    let negative_start = ASCIIDOC_VARIANTS
        .find("==== NEGATIVE")
        .expect("fixture must contain a NEGATIVE section marker");
    let positive_source = &ASCIIDOC_VARIANTS[..negative_start];

    let paths = collect_captures(&lang, positive_source, &query_str, "import.path");

    // bare include, no attributes
    assert!(paths.contains(&"bare.adoc".to_string()), "{paths:?}");
    // single attribute
    assert!(paths.contains(&"single-attr.adoc".to_string()), "{paths:?}");
    // multiple comma-separated attributes (multiple block_macro_attr
    // siblings under one block_macro) must still yield exactly one path
    assert!(paths.contains(&"multi-attr.adoc".to_string()), "{paths:?}");
    // relative path with parent-directory segments
    assert!(
        paths.contains(&"../shared/common.adoc".to_string()),
        "{paths:?}"
    );
    // path containing a document-attribute reference
    assert!(
        paths.contains(&"{includedir}/generated.adoc".to_string()),
        "{paths:?}"
    );
    // nested relative path
    assert!(
        paths.contains(&"sub/nested/deep.adoc".to_string()),
        "{paths:?}"
    );
    assert_eq!(
        paths.len(),
        6,
        "expected exactly 6 positive include paths, got {}: {paths:?}",
        paths.len()
    );
}

#[test]
fn asciidoc_imports_negative_non_include_macros_and_prose() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping asciidoc_imports_negative_non_include_macros_and_prose: run \
             `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("asciidoc").ok() else {
        eprintln!(
            "Skipping asciidoc_imports_negative_non_include_macros_and_prose: \
             asciidoc grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("asciidoc")
        .expect("asciidoc imports query missing");

    let negative_start = ASCIIDOC_VARIANTS
        .find("==== NEGATIVE")
        .expect("fixture must contain a NEGATIVE section marker");
    let negative_source = &ASCIIDOC_VARIANTS[negative_start..];

    let paths = collect_captures(&lang, negative_source, &query_str, "import.path");
    assert!(
        paths.is_empty(),
        "image:: macro and prose text mentioning 'include::' must not produce \
         any @import.path captures, got: {paths:?}"
    );
}
