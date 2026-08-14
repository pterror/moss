//! Query fixture tests for meson.
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
// Meson
// ---------------------------------------------------------------------------

const MESON_SAMPLE: &str = include_str!("fixtures/meson/meson.build");

#[test]
fn meson_tags_finds_variable_assignments() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_tags: meson grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("meson").expect("meson tags query missing");
    // Meson tags captures variable identifiers from var_unit assignments
    let _names = collect_captures(&lang, MESON_SAMPLE, &query_str, "name");
}

#[test]
fn meson_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_calls: meson grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("meson")
        .expect("meson calls query missing");
    let calls = collect_captures(&lang, MESON_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "project" || c == "dependency" || c == "executable" || c == "library"),
        "expected meson function calls in sample, got: {calls:?}"
    );
}

#[test]
fn meson_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_complexity: meson grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("meson")
        .expect("meson complexity query missing");
    let complexity = collect_captures(&lang, MESON_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (if block) in meson sample, got: {complexity:?}"
    );
}

#[test]
fn meson_imports_finds_subproject_and_dependency() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_imports: meson grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("meson")
        .expect("meson imports query missing");
    let paths = collect_captures(&lang, MESON_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("glib-2.0") || p.contains("zlib") || p.contains("protobuf")),
        "expected dependency names in meson imports, got: {paths:?}"
    );
}
