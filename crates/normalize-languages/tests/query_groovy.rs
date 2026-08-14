//! Query fixture tests for groovy.
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
// Groovy / Elixir / Haskell live grammar tests (use ~/.config/normalize/grammars/)
// ---------------------------------------------------------------------------

#[test]
fn groovy_tags_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("groovy").ok() else {
        eprintln!("Skipping groovy_tags_live: groovy grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("groovy")
        .expect("groovy tags query missing");
    let names = collect_captures(&lang, GROOVY_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class, got: {names:?}"
    );
    assert!(
        names.contains(&"distanceTo".to_string()),
        "expected 'distanceTo' method, got: {names:?}"
    );
    assert!(
        names.contains(&"MathUtils".to_string()),
        "expected 'MathUtils' class, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' method, got: {names:?}"
    );
    assert!(
        names.contains(&"greet".to_string()),
        "expected 'greet' function, got: {names:?}"
    );
}

const GROOVY_SAMPLE: &str = include_str!("fixtures/groovy/sample.groovy");

#[test]
fn groovy_imports_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("groovy").ok() else {
        eprintln!("Skipping groovy_imports_live: groovy grammar not found");
        return;
    };
    let query_str = loader
        .get_imports("groovy")
        .expect("groovy imports query missing");
    let paths = collect_captures(&lang, GROOVY_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Immutable") || p.contains("groovy")),
        "expected 'groovy.transform.Immutable' in groovy import paths, got: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .any(|p| p.contains("ArrayList") || p.contains("java")),
        "expected 'java.util.ArrayList' in groovy import paths, got: {paths:?}"
    );
}

#[test]
fn groovy_types_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("groovy").ok() else {
        eprintln!("Skipping groovy_types_live: groovy grammar not found");
        return;
    };
    let query_str = loader
        .get_types("groovy")
        .expect("groovy types query missing");
    let types = collect_captures(&lang, GROOVY_SAMPLE, &query_str, "type.reference");
    assert!(
        types.contains(&"Point".to_string()),
        "expected 'Point' parameter type in groovy types, got: {types:?}"
    );
    assert!(
        types.contains(&"String".to_string()),
        "expected 'String' return type in groovy types, got: {types:?}"
    );
    assert!(
        types.contains(&"List".to_string()),
        "expected base generic type 'List' in groovy types, got: {types:?}"
    );
    assert!(
        types.contains(&"Integer".to_string()),
        "expected generic type argument 'Integer' in groovy types, got: {types:?}"
    );
}

#[test]
fn groovy_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping groovy_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "groovy",
        GROOVY_SAMPLE,
        &["@Immutable", "@Override"],
    );
}
