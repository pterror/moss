//! Query fixture tests for json.
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
use tree_sitter::Parser;

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------
//
// JSON has exactly one query purpose: tags. There are no call/import/complexity/
// type concepts in JSON's grammar (verified: `src/grammar_loader.rs`'s
// `query_source_for("json", ...)` registers only `"json.tags.scm"`, and JSON's
// node-types.json has no call/import-shaped node kinds — see arborium-json
// 2.17.0 node-types.json, dumped in full during this remediation: the only
// node types are `_value`, `array`, `document`, `object`, `pair`, `string`,
// literal/punctuation tokens, `comment` (extra), `escape_sequence`, and
// `string_content`).

const JSON_SAMPLE: &str = include_str!("fixtures/json/sample.json");

const JSON_VARIANTS: &str = include_str!("fixtures/json/variants.json");

/// Returns `(loader, lang)` together — the `GrammarLoader` owns the loaded
/// dylib, so it must stay alive for as long as the returned `Language` is
/// used (dropping it early unloads the library and leaves `Language`'s
/// function pointers dangling, which segfaults rather than erroring).
fn json_lang() -> Option<(normalize_languages::GrammarLoader, tree_sitter::Language)> {
    let loader = normalize_languages::GrammarLoader::new();
    let lang = loader.get("json").ok()?;
    Some((loader, lang))
}

// --- Dimension 4: real-world fixture coverage (sample.json) -----------------

#[test]
fn json_tags_finds_nested_keys_in_realistic_document() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_finds_nested_keys: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let names = collect_captures(&lang, JSON_SAMPLE, &query_str, "name");

    // Top-level scalar/object/array-valued keys.
    for expected in [
        "\"name\"",
        "\"version\"",
        "\"scripts\"",
        "\"dependencies\"",
        "\"keywords\"",
        "\"contributors\"",
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "expected {expected} key in json sample tags, got: {names:?}"
        );
    }
    // Nested object keys (package.json-shaped: scripts.build, deeply nested
    // dependencies.serde.version) must also be found — nesting is structural
    // (pair > object > pair), not something the query itself constrains, but
    // this is the realistic idiom (dimension 4) that would surface a broken
    // query at any depth.
    for expected in ["\"build\"", "\"test\"", "\"tree-sitter\"", "\"version\""] {
        assert!(
            names.contains(&expected.to_string()),
            "expected {expected} nested key in json sample tags, got: {names:?}"
        );
    }
    // Object keys inside array elements (contributors: [{ name, role }, ...])
    // — array elements aren't pairs, but the objects *inside* them still are.
    assert!(
        names.iter().filter(|n| *n == "\"name\"").count() >= 3,
        "expected 'name' from top-level key + 2 contributor objects, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.json) -

/// Every `pair.value` variant node-types.json's `_value` supertype allows
/// (array, false, null, number, object, string, true) must produce a
/// @definition.var capture with kind `pair` on the key — the value's own
/// kind never gates whether the *key* is captured, only whether json.rs's
/// refine_kind() promotes it to a container (tested separately below).
#[test]
fn json_tags_completeness_all_value_kinds_have_captured_keys() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_completeness_all_value_kinds: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_VARIANTS, &query_str);

    // (capture_name, node_kind, text) — every key is a `string` node
    // regardless of its value's kind.
    let required: &[(&str, &str, &str)] = &[
        ("name", "string", "\"valueString\""), // value: string
        ("name", "string", "\"valueNumber\""), // value: number
        ("name", "string", "\"valueTrue\""),   // value: true
        ("name", "string", "\"valueFalse\""),  // value: false
        ("name", "string", "\"valueNull\""),   // value: null
        ("name", "string", "\"valueArray\""),  // value: array
        ("name", "string", "\"valueObject\""), // value: object
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in json.tags.scm output \
             for variants.json, got: {caps:?}"
        );
    }
}

/// Real, previously-silent bug (found via a killed prior session, verified
/// against real parse output in this session): empty-string keys (`"":
/// value`) parse as a `string` node with NO `string_content` child at all —
/// the grammar only emits that child for non-empty runs. A
/// `(string (string_content) @name)` pattern requires the child and so
/// silently drops the pair from extraction entirely. json.tags.scm now
/// captures the whole `string` node instead, which does not depend on
/// `string_content` existing.
#[test]
fn json_tags_completeness_empty_string_key_is_captured() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_completeness_empty_string_key: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_VARIANTS, &query_str);

    let empty_key_defs: Vec<_> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "definition.var" && k == "pair" && t.starts_with("\"\":"))
        .collect();
    assert_eq!(
        empty_key_defs.len(),
        1,
        "expected exactly 1 @definition.var for the empty-string-key pair, got: {caps:?}"
    );
}

/// Second real, previously-unverified bug found while validating the prior
/// session's fix: keys containing an escape sequence (`"a\nb"`) parse as a
/// `string` node with *multiple* `string_content` children (one literal run
/// per side of each `escape_sequence`). A per-child `@name` pattern produces
/// one separate query match per run — duplicating the @definition.var match
/// for the same pair and truncating @name to a single run ("a" or "b", never
/// the full key). Capturing the whole `string` node produces exactly one
/// match per pair regardless of how many string_content/escape_sequence
/// children it has internally.
#[test]
fn json_tags_completeness_escape_sequence_key_is_single_match() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_completeness_escape_sequence_key: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_VARIANTS, &query_str);

    // "a\nb" key: exactly one @definition.var match (not one per string_content run).
    let escape_run_defs: Vec<_> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "definition.var" && k == "pair" && t.contains("\"a\\nb\":"))
        .collect();
    assert_eq!(
        escape_run_defs.len(),
        1,
        "expected exactly 1 @definition.var for the 'a\\nb' escape-run key, got: {caps:?}"
    );

    // "\n" key (pure escape sequence, no string_content child at all): must
    // still be captured, not dropped the way the empty-key case was.
    let pure_escape_defs: Vec<_> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "definition.var" && k == "pair" && t.starts_with("\"\\n\":"))
        .collect();
    assert_eq!(
        pure_escape_defs.len(),
        1,
        "expected exactly 1 @definition.var for the pure-escape '\\n' key, got: {caps:?}"
    );
}

/// NEGATIVE: array elements are values, not pairs — a bare array of scalars
/// must contribute zero @name/@definition.var captures of its own (only the
/// pairs nested in the *objects* an array may contain are captured).
#[test]
fn json_tags_negative_array_scalar_elements_not_captured() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_negative_array_scalar_elements: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_SAMPLE, &query_str);

    // "keywords": ["parsing", "tree-sitter", "cli"] — none of the array's
    // string elements may appear as a @name capture (they are values, not
    // pair keys).
    for scalar in ["\"parsing\"", "\"cli\""] {
        assert!(
            !caps.iter().any(|(cn, _, t, _)| cn == "name" && t == scalar),
            "array scalar element {scalar} must never appear as a @name capture, got: {caps:?}"
        );
    }
}

// --- json.rs Language trait: node_name() / refine_kind() extraction depth ---

/// `node_name()` must return the full raw key text (between the string
/// node's own quotes), not just the first `string_content` run — regression
/// test for the escape-run truncation bug at the Rust-code level (as
/// opposed to the query level tested above), since `node_name()` is what
/// actually supplies the symbol name consumed by `collect_symbols_from_tags`.
#[test]
fn json_node_name_handles_empty_and_escape_keys() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_node_name_handles_empty_and_escape_keys: json grammar not found");
        return;
    };
    let _ = &loader;
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(JSON_VARIANTS, None).expect("parse failed");

    let json_lang_impl = normalize_languages::Json;
    let mut cursor = tree.walk();
    let mut found: Vec<(String, String)> = Vec::new();

    fn walk(
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        support: &dyn normalize_languages::Language,
        found: &mut Vec<(String, String)>,
    ) {
        loop {
            let node = cursor.node();
            if node.kind() == "pair"
                && let Some(name) = support.node_name(&node, content)
            {
                let value_text = node
                    .child_by_field_name("value")
                    .map(|v| v.utf8_text(content.as_bytes()).unwrap_or(""))
                    .unwrap_or("");
                found.push((name.to_string(), value_text.to_string()));
            }
            if cursor.goto_first_child() {
                walk(cursor, content, support, found);
                cursor.goto_parent();
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    walk(&mut cursor, JSON_VARIANTS, &json_lang_impl, &mut found);

    assert!(
        found.contains(&(String::new(), "\"emptyKeyValue\"".to_string())),
        "expected node_name() to return \"\" (not None) for the empty-string key, got: {found:?}"
    );
    assert!(
        found.contains(&("a\\nb".to_string(), "\"escapeRunKeyValue\"".to_string())),
        "expected node_name() to return the FULL 'a\\nb' key text (not truncated to 'a'), \
         got: {found:?}"
    );
    assert!(
        found.contains(&("\\n".to_string(), "\"pureEscapeKeyValue\"".to_string())),
        "expected node_name() to return '\\n' (pure-escape key, no string_content child), \
         got: {found:?}"
    );
    assert!(
        found.contains(&("plainKey".to_string(), "\"plainKeyValue\"".to_string())),
        "expected node_name() to return the plain key unchanged, got: {found:?}"
    );
}

/// `refine_kind()` must promote a pair to `Module` (container) only when its
/// value is an `object` — every other `_value` variant (array/false/null/
/// number/string/true) must remain a plain `Variable`. Verified via
/// `collect_symbols_from_tags`'s nesting logic (only `is_container_kind`
/// containers can hold children) rather than re-implementing refine_kind's
/// logic here.
#[test]
fn json_refine_kind_only_object_values_are_containers() {
    let Some((loader, _lang)) = json_lang() else {
        eprintln!(
            "Skipping json_refine_kind_only_object_values_are_containers: json grammar not found"
        );
        return;
    };
    let _ = &loader;
    let json_impl = normalize_languages::Json;

    let mut parser = Parser::new();
    let lang2 = normalize_languages::GrammarLoader::new()
        .get("json")
        .expect("json grammar");
    parser.set_language(&lang2).expect("set_language failed");
    let tree = parser.parse(JSON_VARIANTS, None).expect("parse failed");
    let root = tree.root_node();

    // Find the "valueObject" and "valueArray" pair nodes directly and check
    // refine_kind()'s classification of each.
    fn find_pair<'t>(
        node: tree_sitter::Node<'t>,
        key_text: &str,
        content: &str,
    ) -> Option<tree_sitter::Node<'t>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "pair"
                && let Some(key) = child.child_by_field_name("key")
                && key.utf8_text(content.as_bytes()).unwrap_or("") == key_text
            {
                return Some(child);
            }
            if let Some(found) = find_pair(child, key_text, content) {
                return Some(found);
            }
        }
        None
    }

    let value_object_pair = find_pair(root, "\"valueObject\"", JSON_VARIANTS)
        .expect("valueObject pair not found in variants.json");
    let value_array_pair = find_pair(root, "\"valueArray\"", JSON_VARIANTS)
        .expect("valueArray pair not found in variants.json");

    let refined_object = normalize_languages::Language::refine_kind(
        &json_impl,
        &value_object_pair,
        JSON_VARIANTS,
        normalize_languages::SymbolKind::Variable,
    );
    let refined_array = normalize_languages::Language::refine_kind(
        &json_impl,
        &value_array_pair,
        JSON_VARIANTS,
        normalize_languages::SymbolKind::Variable,
    );

    assert_eq!(
        refined_object,
        normalize_languages::SymbolKind::Module,
        "object-valued pair must refine to Module (container)"
    );
    assert_eq!(
        refined_array,
        normalize_languages::SymbolKind::Variable,
        "array-valued pair must remain Variable (not a container), got: {refined_array:?}"
    );
}
