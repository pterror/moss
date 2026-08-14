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

/// Regression test for the `normalize view` display bug where an annotated
/// definition's signature was the annotation text instead of the declaration
/// (e.g. `@Immutable class Point { ... }` rendered as `@Immutable`, with
/// `Point` never appearing — see TODO.md's 2026-08-14 entry). Runs the real
/// tags query to find the `@definition.class`/`@definition.method` node for
/// each annotated symbol, then calls `Language::build_signature` on it
/// directly — the same call `normalize-facts::extract` makes when building a
/// `Symbol`.
#[test]
fn groovy_build_signature_skips_leading_annotation() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("groovy").ok() else {
        eprintln!(
            "Skipping groovy_build_signature_skips_leading_annotation: groovy grammar not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("groovy")
        .expect("groovy tags query missing");

    use normalize_languages::Language;
    use tree_sitter::StreamingIterator;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(GROOVY_SAMPLE, None).expect("parse failed");
    let query = tree_sitter::Query::new(&lang, &query_str).expect("query compilation failed");
    let mut cursor = tree_sitter::QueryCursor::new();
    let source_bytes = GROOVY_SAMPLE.as_bytes();

    let mut signature_for_name: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        let mut name = None;
        let mut def_node = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            if cap_name == "name" {
                name = Some(cap.node.utf8_text(source_bytes).unwrap_or("").to_string());
            } else if cap_name.starts_with("definition.") {
                def_node = Some(cap.node);
            }
        }
        if let (Some(n), Some(node)) = (name, def_node) {
            signature_for_name.insert(
                n,
                normalize_languages::Groovy.build_signature(&node, GROOVY_SAMPLE),
            );
        }
    }

    let point_sig = signature_for_name
        .get("Point")
        .unwrap_or_else(|| panic!("no 'Point' definition found, got: {signature_for_name:?}"));
    assert_eq!(
        point_sig, "class Point {",
        "Point's signature should be its declaration, not its leading @Immutable annotation"
    );

    let distance_to_sig = signature_for_name
        .get("distanceTo")
        .unwrap_or_else(|| panic!("no 'distanceTo' definition found, got: {signature_for_name:?}"));
    assert_eq!(
        distance_to_sig, "double distanceTo(Point other) {",
        "distanceTo's signature should be its declaration, not its leading @Override annotation"
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
