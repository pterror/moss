//! Query fixture tests for vue.
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
// Vue
// ---------------------------------------------------------------------------

const VUE_SAMPLE: &str = include_str!("fixtures/vue/sample.vue");

#[test]
fn vue_tags_finds_script_template_and_style_blocks() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vue_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vue").ok() else {
        eprintln!("Skipping vue_tags: vue grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vue").expect("vue tags query missing");
    let names = collect_captures(&lang, VUE_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "script" || n == "template" || n == "style"),
        "expected SFC block tag names in vue tags, got: {names:?}"
    );
}

#[test]
fn vue_calls_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vue_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vue").ok() else {
        eprintln!("Skipping vue_calls: vue grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vue").expect("vue calls query missing");
    // Vue calls query is intentionally empty (JS in <script> is raw_text)
    let _calls = collect_captures(&lang, VUE_SAMPLE, &query_str, "call");
}

#[test]
fn vue_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vue_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vue").ok() else {
        eprintln!("Skipping vue_complexity: vue grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vue")
        .expect("vue complexity query missing");
    let _complexity = collect_captures(&lang, VUE_SAMPLE, &query_str, "complexity");
}
