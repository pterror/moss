//! Query fixture tests for yaml.
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
use std::sync::Arc;

// ---------------------------------------------------------------------------
// YAML
// ---------------------------------------------------------------------------

const YAML_SAMPLE: &str = include_str!("fixtures/yaml/sample.yaml");

const YAML_VARIANTS: &str = include_str!("fixtures/yaml/variants.yaml");

/// `(loader, language, tags)` — see `sql_lang_and_queries` for why the
/// loader must be kept alongside the language it produced (dropping the
/// `GrammarLoader` unloads the backing `.so` and dangles the `Language`'s
/// function pointers).
type YamlLangAndTags = (GrammarLoader, tree_sitter::Language, Arc<String>);

fn yaml_lang_and_tags() -> Option<YamlLangAndTags> {
    let gdir = grammar_dir()?;
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let lang = loader.get("yaml").ok()?;
    let tags = loader.get_tags("yaml")?;
    Some((loader, lang, tags))
}

// --- Dimension 4: real-world fixture coverage (sample.yaml) ----------------

#[test]
fn yaml_tags_finds_real_world_keys() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, YAML_SAMPLE, &tags, "name");
    // Block-style nested keys.
    assert!(
        names.iter().any(|n| n == "jobs"),
        "expected top-level 'jobs' key, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "runs-on"),
        "expected nested 'runs-on' key, got: {names:?}"
    );
    // YAML merge key (`<<: *defaults`) — a plain_scalar key like any other.
    assert!(
        names.iter().any(|n| n == "<<"),
        "expected merge key '<<', got: {names:?}"
    );
    // Flow-mapping keys inside a block sequence item
    // (`- { os: ubuntu-latest, arch: x64 }`).
    assert!(
        names.iter().any(|n| n == "os"),
        "expected flow_pair key 'os' inside matrix_include, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "arch"),
        "expected flow_pair key 'arch' inside matrix_include, got: {names:?}"
    );
    // Quoted top-level keys.
    assert!(
        names.iter().any(|n| n == "\"quoted top-level key\""),
        "expected double-quoted top-level key, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "'single quoted top-level key'"),
        "expected single-quoted top-level key, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix (variants.yaml) -------------------

/// Every `block_mapping_pair`/`flow_pair` key-scalar variant found by
/// cross-referencing arborium-yaml 2.17.0's node-types.json (plain_scalar,
/// double_quote_scalar, single_quote_scalar — for both block and flow
/// pairs) must produce a `definition.var` with the correct capture kind.
/// Uses `collect_captures_full` so a kind mismatch (e.g. a quoted key
/// accidentally matched by the plain_scalar clause, or vice versa) can't
/// hide behind string-only assertions.
#[test]
fn yaml_tags_completeness_key_scalar_variants() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let full = collect_captures_full(&lang, YAML_VARIANTS, &tags);
    let name_kind_text = |name: &str| -> Option<&str> {
        full.iter()
            .find(|(cap, _, text, _)| cap == "name" && text == name)
            .map(|(_, kind, ..)| kind.as_str())
    };

    // block_mapping_pair.key variants.
    assert_eq!(
        name_kind_text("plain_key"),
        Some("string_scalar"),
        "plain block key must capture as string_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("\"double quoted key\""),
        Some("double_quote_scalar"),
        "double-quoted block key must capture as double_quote_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("'single quoted key'"),
        Some("single_quote_scalar"),
        "single-quoted block key must capture as single_quote_scalar, full: {full:?}"
    );
    // Anchor/tag-prefixed keys still resolve to the plain scalar name —
    // the anchor/tag is a sibling within the same flow_node, not a wrapper.
    assert_eq!(
        name_kind_text("anchored_key"),
        Some("string_scalar"),
        "anchor-prefixed key must still capture the plain scalar name, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("tagged_key"),
        Some("string_scalar"),
        "tag-prefixed key must still capture the plain scalar name, full: {full:?}"
    );

    // flow_pair.key variants (all three scalar kinds inside a flow_mapping).
    assert_eq!(
        name_kind_text("flow_plain"),
        Some("string_scalar"),
        "flow_pair plain key must capture as string_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("\"flow double quoted\""),
        Some("double_quote_scalar"),
        "flow_pair double-quoted key must capture as double_quote_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("'flow single quoted'"),
        Some("single_quote_scalar"),
        "flow_pair single-quoted key must capture as single_quote_scalar, full: {full:?}"
    );

    // Multi-document stream: a key in the second document must be found
    // too — tags matching is structural, not document-scoped.
    assert_eq!(
        name_kind_text("doc2_key"),
        Some("string_scalar"),
        "key in second document of a multi-document stream must still be captured, full: {full:?}"
    );
}

/// `node_name`/`refine_kind`/`container_body` (Rust-side symbol
/// reconstruction) are covered directly in `crates/normalize-languages/src/yaml.rs`'s
/// own unit tests, since they operate on `tree_sitter::Node` rather than
/// producing query captures — this test only asserts the raw query's
/// container-key captures (both nesting paths: a block key with an inline
/// flow-mapping value, and a flow pair nested inside another flow mapping).
#[test]
fn yaml_tags_completeness_container_nesting() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags_container_nesting: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, YAML_VARIANTS, &tags, "name");

    // block_mapping_pair value = block_node > block_mapping.
    assert!(names.iter().any(|n| n == "block_container"));
    assert!(names.iter().any(|n| n == "nested_key"));

    // block_mapping_pair value = flow_node > flow_mapping (inline flow
    // value on a block-style key).
    assert!(names.iter().any(|n| n == "inline_flow_container"));
    assert!(names.iter().any(|n| n == "a"));
    assert!(names.iter().any(|n| n == "b"));

    // flow_pair value = flow_node > flow_mapping (flow mapping nested
    // inside another flow mapping).
    assert!(names.iter().any(|n| n == "nested_flow_container"));
    assert!(names.iter().any(|n| n == "outer"));
    assert!(names.iter().any(|n| n == "inner"));
}

/// Negative cases: constructs that must NOT produce a `@name` capture.
/// Asserts the exact total count on `variants.yaml` so a stray extra match
/// (e.g. a sequence item accidentally matched) can't hide behind a
/// `.any()`-only check.
#[test]
fn yaml_tags_negative_sequences_and_complex_keys() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, YAML_VARIANTS, &tags, "name");

    // Sequence items themselves are never named symbols — only the key
    // that holds the sequence is.
    assert!(
        !names.iter().any(|n| n == "item1" || n == "item2"),
        "block sequence items must not be captured as symbols, got: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n == "1" || n == "2" || n == "3"),
        "flow sequence items must not be captured as symbols, got: {names:?}"
    );

    // Explicit complex key (`? [a, b]`) — a flow_sequence key has no single
    // scalar name to extract; must not match.
    assert!(
        !names.iter().any(|n| n.contains('[') || n.contains(']')),
        "complex flow-sequence key must not be captured, got: {names:?}"
    );

    // Explicit block-scalar key (`? |`) — multi-line blob, must not match.
    assert!(
        !names.iter().any(|n| n.contains("block scalar key")),
        "explicit block-scalar key must not be captured, got: {names:?}"
    );

    // Anchors/aliases: `alias_source`/`alias_use` are ordinary block-mapping
    // keys (captured normally); the anchor definition (`&shared_anchor`)
    // and alias reference (`*shared_anchor`) themselves must not produce
    // separate captures — this codebase's tags pipeline has no
    // definition/reference convention for them (see yaml.tags.scm).
    assert!(
        names.iter().any(|n| n == "alias_source"),
        "expected 'alias_source' key itself to be captured, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "alias_use"),
        "expected 'alias_use' key itself to be captured, got: {names:?}"
    );
    assert!(
        !names
            .iter()
            .any(|n| n == "shared_anchor" || n == "&shared_anchor" || n == "*shared_anchor"),
        "anchor/alias names themselves must not be captured as symbols, got: {names:?}"
    );

    // Exact total count on variants.yaml: pins the full set of matches so
    // any future regression (extra or missing match) is caught precisely.
    assert_eq!(
        names.len(),
        22,
        "expected exactly 22 @name captures in variants.yaml, got {}: {names:?}",
        names.len()
    );
}
