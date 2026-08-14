//! Query fixture tests for caddy.
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
// Caddy / Dockerfile
// ---------------------------------------------------------------------------

const CADDY_SAMPLE: &str = include_str!("fixtures/caddy/sample.caddyfile");

#[test]
fn caddy_imports_finds_snippet_reference() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping caddy_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("caddy").ok() else {
        eprintln!("Skipping caddy_imports: caddy grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("caddy")
        .expect("caddy imports query missing");
    let paths = collect_captures(&lang, CADDY_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("common-headers")),
        "expected '(common-headers)' snippet reference in caddy imports, got: {paths:?}"
    );
}
