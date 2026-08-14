//! Query fixture tests for toml.
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
// TOML
// ---------------------------------------------------------------------------
//
// TOML only has a tags.scm (no calls/complexity/imports/types — data-format
// grammar, not a programming language). arborium-toml 2.17.0's grammar has
// no named fields at all: the header key of a `table`/`table_array_element`
// and the key of a `pair` is whichever of three sibling node kinds appears —
// `bare_key`, `quoted_key`, or `dotted_key` — verified via `node-types.json`
// and cross-checked against real parse output (`normalize syntax ast`/
// `normalize syntax query`). The original query only matched `bare_key`,
// silently dropping every quoted-key and dotted-key table/pair name; dotted
// table headers alone (`[workspace.dependencies]`, `[profile.dev.build-
// override]`, …) appear 40+ times and dotted pair keys 370+ times in this
// repo's own `.toml` files.

const TOML_SAMPLE: &str = include_str!("fixtures/toml/sample.toml");

const TOML_VARIANTS: &str = include_str!("fixtures/toml/variants.toml");

// --- Dimension 4: real-world fixture coverage (sample.toml) ----------------

#[test]
fn toml_tags_finds_sample_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_finds_sample_definitions: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!("Skipping toml_tags_finds_sample_definitions: toml grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let pairs = collect_tag_pairs(&lang, TOML_SAMPLE, &query_str);
    let class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.class")
        .map(|(_, n)| n.as_str())
        .collect();
    let var_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.var")
        .map(|(_, n)| n.as_str())
        .collect();

    // Plain table header.
    assert!(
        class_names.contains(&"package"),
        "expected 'package' table in toml tags, got: {class_names:?}"
    );
    // Dotted table header: [workspace.dependencies].
    assert!(
        class_names.contains(&"workspace.dependencies"),
        "expected dotted table header 'workspace.dependencies' in toml tags, got: {class_names:?}"
    );
    // Multi-segment dotted table header: [profile.dev.build-override].
    assert!(
        class_names.contains(&"profile.dev.build-override"),
        "expected dotted table header 'profile.dev.build-override' in toml tags, got: {class_names:?}"
    );
    // Quoted-key table header: ["cfg(unix)".dependencies].
    assert!(
        class_names.contains(&"\"cfg(unix)\".dependencies"),
        "expected quoted-key dotted table header in toml tags, got: {class_names:?}"
    );
    // Array-of-tables headers: three [[bin]] entries, each its own definition.
    let bin_count = class_names.iter().filter(|n| **n == "bin").count();
    assert_eq!(
        bin_count, 3,
        "expected 3 separate '[[bin]]' array-table entries in toml tags, got {bin_count}: {class_names:?}"
    );
    // Dotted pair key nested inside a table: [lints.rust] -> rust.unused = "warn".
    assert!(
        var_names.contains(&"rust.unused"),
        "expected dotted pair key 'rust.unused' in toml tags, got: {var_names:?}"
    );
    // Quoted pair key: "ci.badge-url" = "...".
    assert!(
        var_names.contains(&"\"ci.badge-url\""),
        "expected quoted pair key '\"ci.badge-url\"' in toml tags, got: {var_names:?}"
    );
    // Dotted array-of-tables header: [[metadata.matrix]], twice.
    let matrix_count = class_names
        .iter()
        .filter(|n| **n == "metadata.matrix")
        .count();
    assert_eq!(
        matrix_count, 2,
        "expected 2 separate '[[metadata.matrix]]' entries in toml tags, got {matrix_count}: {class_names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.toml) -

#[test]
fn toml_tags_completeness_table_header_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_completeness_table_header_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!(
            "Skipping toml_tags_completeness_table_header_variants: toml grammar .so not found"
        );
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let captures = collect_captures_full(&lang, TOML_VARIANTS, &query_str);

    // (expected @name text, expected node kind, which container: table | table_array_element)
    let expected: &[(&str, &str, &str)] = &[
        // table header key variants
        ("bare_table", "bare_key", "table"),
        ("\"quoted table header\"", "quoted_key", "table"),
        ("dotted.table.header", "dotted_key", "table"),
        ("dotted.\"quoted segment\".header", "dotted_key", "table"),
        // table_array_element header key variants
        ("bare_array_table", "bare_key", "table_array_element"),
        (
            "\"quoted array table header\"",
            "quoted_key",
            "table_array_element",
        ),
        ("dotted.array.table", "dotted_key", "table_array_element"),
    ];

    for (text, kind, _container) in expected {
        let found = captures
            .iter()
            .any(|(cap, k, t, _line)| cap == "name" && k == kind && t == text);
        assert!(
            found,
            "expected @name capture text={text:?} kind={kind:?} in toml_tags_completeness_table_header_variants, \
             got captures: {captures:?}"
        );
    }

    // Every table/table_array_element header must co-occur with the right
    // definition.* tag kind (dimension: definition-kind distinction).
    let tag_pairs = collect_tag_pairs(&lang, TOML_VARIANTS, &query_str);
    assert!(
        tag_pairs.contains(&(
            "definition.class".to_string(),
            "dotted.table.header".to_string()
        )),
        "expected dotted table header tagged as definition.class, got: {tag_pairs:?}"
    );
    assert!(
        tag_pairs.contains(&(
            "definition.class".to_string(),
            "dotted.array.table".to_string()
        )),
        "expected dotted array-table header tagged as definition.class, got: {tag_pairs:?}"
    );
}

#[test]
fn toml_tags_completeness_pair_key_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_completeness_pair_key_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!("Skipping toml_tags_completeness_pair_key_variants: toml grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let tag_pairs = collect_tag_pairs(&lang, TOML_VARIANTS, &query_str);

    // Top-level pair key variants.
    for kind_text in [
        "bare_pair",
        "\"quoted pair key\"",
        "dotted.pair.key",
        "dotted.\"quoted segment\".key",
    ] {
        assert!(
            tag_pairs.contains(&("definition.var".to_string(), kind_text.to_string())),
            "expected pair key '{kind_text}' tagged as definition.var, got: {tag_pairs:?}"
        );
    }

    // Same three key-shape variants nested inside an explicit [nested] table
    // — the key variants must be recognized regardless of container.
    let var_count_dotted_nested = tag_pairs
        .iter()
        .filter(|(k, n)| k == "definition.var" && n == "dotted.pair.key")
        .count();
    assert_eq!(
        var_count_dotted_nested, 2,
        "expected 'dotted.pair.key' as a pair key both top-level and nested in [nested], got {var_count_dotted_nested}: {tag_pairs:?}"
    );

    // collect_captures_full: verify node kind, not just text, for the
    // dotted_key variant specifically (extraction-depth dimension).
    let captures = collect_captures_full(&lang, TOML_VARIANTS, &query_str);
    let dotted_pair_kind = captures
        .iter()
        .find(|(cap, _k, t, _line)| cap == "name" && t == "dotted.pair.key")
        .map(|(_cap, k, _t, _line)| k.as_str());
    assert_eq!(
        dotted_pair_kind,
        Some("dotted_key"),
        "expected 'dotted.pair.key' @name capture to be node kind dotted_key, got: {dotted_pair_kind:?} \
         (full captures: {captures:?})"
    );
}

#[test]
fn toml_tags_negative_values_and_scalars_not_captured_as_names() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_negative_values_and_scalars_not_captured_as_names: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!(
            "Skipping toml_tags_negative_values_and_scalars_not_captured_as_names: toml grammar .so not found"
        );
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let names = collect_captures(&lang, TOML_VARIANTS, &query_str, "name");

    // `value_looks_like_key = "not.a.real.key"` — the *string value* text
    // must never itself appear as a captured @name; only the pair's own key
    // ("value_looks_like_key") should.
    assert!(
        names.contains(&"value_looks_like_key".to_string()),
        "expected the pair's own key to be captured, got: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.contains("not.a.real.key")),
        "string value text must never be captured as a @name, got: {names:?}"
    );

    // Scalar array elements (`plain_array = [1, 2, 3]`) are never keys —
    // only the pair's own key ("plain_array") should be captured, not "1",
    // "2", or "3".
    assert!(
        !names.contains(&"1".to_string())
            && !names.contains(&"2".to_string())
            && !names.contains(&"3".to_string()),
        "scalar array elements must never be captured as a @name, got: {names:?}"
    );

    // Inline tables (`inline = { a = 1, b = 2 }`): the pair's own key
    // ("inline") is captured as a definition.var, same as any other pair.
    // Its inner pairs ("a", "b") also match the query as written — this is
    // documented, existing behavior (see the comment atop toml.tags.scm and
    // `is_inside_inline_table` in toml.rs): the query intentionally doesn't
    // special-case inline tables, and downstream Rust-level `node_name()`
    // filters inline-table-nested pairs back out of actual symbol
    // extraction. This assertion pins that query-level behavior so a future
    // change to either layer is deliberate, not accidental.
    assert!(
        names.contains(&"inline".to_string())
            && names.contains(&"a".to_string())
            && names.contains(&"b".to_string()),
        "expected inline table's own key plus its inner pair keys to all match \
         at the query level (filtered downstream in toml.rs), got: {names:?}"
    );
}
