//! Query fixture tests for ninja.
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

// ---------------------------------------------------------------------------
// Ninja — imports.scm (query-testing methodology batch 9)
// ---------------------------------------------------------------------------

const NINJA_SAMPLE: &str = include_str!("fixtures/ninja/sample.build.ninja");

const NINJA_VARIANTS: &str = include_str!("fixtures/ninja/variants.ninja");

/// Dimension 4: the real-world-shaped sample (variables, rules, build
/// edges, a pool, a default target) must surface both import-like
/// directives — `include` (definitions merged into current scope) and
/// `subninja` (scoped sub-build) — as imports. Both must be captured, not
/// just one, since they are structurally distinct node types
/// (`include` vs `subninja`) each requiring their own query pattern.
#[test]
fn ninja_imports_finds_sample_include_and_subninja() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping ninja_imports_finds_sample_include_and_subninja: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ninja").ok() else {
        eprintln!(
            "Skipping ninja_imports_finds_sample_include_and_subninja: ninja grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("ninja")
        .expect("ninja imports query missing");
    let paths = collect_captures(&lang, NINJA_SAMPLE, &query_str, "import.path");
    assert_eq!(
        paths,
        vec!["rules.ninja", "subdir/build.ninja"],
        "expected exactly one @import.path for the `include` directive and \
         one for the `subninja` directive, in source order, got: {paths:?}"
    );
}

/// Dimension 2/3 (completeness + extraction depth) for imports.scm: `include`
/// and `subninja` are structurally identical in node-types.json (an
/// unfielded `path` child, `multiple: false`, `required: true`), so there is
/// exactly one shape per directive — verified by kind via
/// `collect_captures_full` so a query that accidentally matched the wrong
/// node type couldn't hide behind identical capture text.
#[test]
fn ninja_imports_completeness_include_and_subninja_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping ninja_imports_completeness_include_and_subninja_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ninja").ok() else {
        eprintln!(
            "Skipping ninja_imports_completeness_include_and_subninja_variants: ninja grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("ninja")
        .expect("ninja imports query missing");

    let full = collect_captures_full(&lang, NINJA_VARIANTS, &query_str);
    let import_kinds: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import")
        .map(|(_, kind, ..)| kind.as_str())
        .collect();
    assert_eq!(
        import_kinds,
        vec!["include", "subninja"],
        "expected exactly one @import of kind `include` and one of kind \
         `subninja`, got: {import_kinds:?}"
    );

    let path_texts: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.path")
        .map(|(_, _, text, _)| text.as_str())
        .collect();
    assert_eq!(
        path_texts,
        vec!["rules.ninja", "subdir/build.ninja"],
        "got: {path_texts:?}"
    );
}

/// Dimension 3 negative case: a variable value that merely contains the
/// words "include"/"subninja" as text, and a build edge whose input/output
/// filenames literally contain "include.ninja", must not be mistaken for
/// the `include`/`subninja` directives.
#[test]
fn ninja_imports_negative_lookalike_text_and_filenames() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping ninja_imports_negative_lookalike_text_and_filenames: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ninja").ok() else {
        eprintln!(
            "Skipping ninja_imports_negative_lookalike_text_and_filenames: ninja grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("ninja")
        .expect("ninja imports query missing");

    let negative_start = NINJA_VARIANTS
        .find("--- NEGATIVE")
        .expect("fixture must contain a NEGATIVE section marker");
    let negative_source = &NINJA_VARIANTS[negative_start..];

    let paths = collect_captures(&lang, negative_source, &query_str, "import.path");
    assert!(
        paths.is_empty(),
        "the lookalike variable text and include.ninja-named build edge must \
         not produce any @import.path captures, got: {paths:?}"
    );
}
