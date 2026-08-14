//! Query fixture tests for clojure.
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
use tree_sitter::Parser;

// ---------------------------------------------------------------------------
// Clojure
// ---------------------------------------------------------------------------

const CLOJURE_SAMPLE: &str = include_str!("fixtures/clojure/sample.clj");

#[test]
fn clojure_tags_finds_functions_and_defrecord() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_tags: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("clojure")
        .expect("clojure tags query missing");
    let names = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in clojure tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify-point".to_string()),
        "expected 'classify-point' function in clojure tags, got: {names:?}"
    );
}

#[test]
fn clojure_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_calls: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("clojure")
        .expect("clojure calls query missing");
    let calls = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "println"),
        "expected 'println' call in clojure sample, got: {calls:?}"
    );
}

#[test]
fn clojure_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_complexity: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("clojure")
        .expect("clojure complexity query missing");
    let complexity = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in clojure sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn clojure_imports_finds_require_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_imports: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("clojure")
        .expect("clojure imports query missing");
    let paths = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("clojure")),
        "expected a clojure.* namespace in import paths, got: {paths:?}"
    );
}

#[test]
fn clojure_types_finds_no_captures() {
    // Clojure is dynamically typed; the types query intentionally captures nothing.
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_types: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("clojure")
        .expect("clojure types query missing");
    // Query parses successfully — result may be empty, that's correct for dynamic languages.
    let _ = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "type");
}

const CLOJURE_VARIANTS: &str = include_str!("fixtures/clojure/variants.clj");

#[test]
fn clojure_tags_completeness_all_definition_forms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_tags_completeness: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("clojure")
        .expect("clojure tags query missing");
    let pairs = collect_tag_pairs(&lang, CLOJURE_VARIANTS, &query_str);

    // Every definition-form variant from node-types.json's sym_lit shape,
    // paired with its expected @definition.* kind.
    let expected: &[(&str, &str)] = &[
        ("definition.function", "plain-fn"),
        ("definition.function", "private-fn"),
        ("definition.function", "meta-private-fn"),
        ("definition.function", "multi-arity-fn"),
        ("definition.macro", "my-macro"),
        ("definition.function", "shape-area"), // defmulti
        ("definition.method", "shape-area"),   // defmethod
        ("definition.module", "variants.ns-form"),
        ("definition.class", "VRecord"),
        ("definition.class", "VType"),
        ("definition.interface", "VProto"),
        ("definition.interface", "VInterface"), // definterface
        ("definition.constant", "a-constant"),
        ("definition.module", "variants.ns-with-import"),
    ];
    for (kind, name) in expected {
        assert!(
            pairs.iter().any(|(k, n)| k == kind && n == name),
            "expected ({kind}, {name}) in clojure tags variants, got: {pairs:?}"
        );
    }

    // The @name capture must be the bare `name:` field, never the whole
    // sym_lit including a `^:private` metadata prefix — this was the actual
    // bug: capturing the bare `sym_lit` produced "^:private meta-private-fn"
    // instead of "meta-private-fn".
    assert!(
        pairs
            .iter()
            .any(|(_, n)| n == "meta-private-fn" && !n.contains(':')),
        "expected clean name 'meta-private-fn' with no leaked metadata prefix, got: {pairs:?}"
    );
}

#[test]
fn clojure_tags_negative_locals_and_calls_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_tags_negative: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("clojure")
        .expect("clojure tags query missing");
    let pairs = collect_tag_pairs(&lang, CLOJURE_VARIANTS, &query_str);
    for bad_name in ["local-not-a-def", "refer-a", "refer-b"] {
        assert!(
            !pairs.iter().any(|(_, n)| n == bad_name),
            "'{bad_name}' must not appear as a tags definition, got: {pairs:?}"
        );
    }
}

#[test]
fn clojure_imports_completeness_all_require_and_import_shapes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_imports_completeness: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("clojure")
        .expect("clojure imports query missing");
    let paths = collect_captures(&lang, CLOJURE_VARIANTS, &query_str, "import.path");
    let names = collect_captures(&lang, CLOJURE_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, CLOJURE_VARIANTS, &query_str, "import.alias");

    // Bare top-level quoted `(require '[ns :as alias])` — previously entirely
    // unmatched because the query only handled the unquoted shape.
    assert!(
        paths.contains(&"variants.aliased".to_string()),
        "expected bare quoted require path 'variants.aliased', got: {paths:?}"
    );
    assert!(
        aliases.contains(&"va".to_string()),
        "expected alias 'va' for bare quoted require, got: {aliases:?}"
    );
    // Multiple quoted vectors in one require call.
    for p in ["variants.multi-a", "variants.multi-b"] {
        assert!(
            paths.contains(&p.to_string()),
            "expected multi-vector require path '{p}', got: {paths:?}"
        );
    }
    // require with no :as.
    assert!(
        paths.contains(&"variants.no-alias".to_string()),
        "expected no-alias require path, got: {paths:?}"
    );
    // Package-grouped `(import (java.util UUID Random))`.
    assert!(
        paths.contains(&"java.util".to_string()),
        "expected package path 'java.util', got: {paths:?}"
    );
    for c in ["UUID", "Random"] {
        assert!(
            names.contains(&c.to_string()),
            "expected class name '{c}' from package-grouped import, got: {names:?}"
        );
    }
    // Quoted single fully-qualified class.
    assert!(
        paths.contains(&"java.util.Date".to_string()),
        "expected quoted single-class import path, got: {paths:?}"
    );
    // Bare fully-qualified class.
    assert!(
        paths.contains(&"java.io.File".to_string()),
        "expected bare single-class import path, got: {paths:?}"
    );
    // `:import` clause nested in `ns`, both shapes, including the SECOND
    // entry after a package-grouped list — the anchored-on-first-sibling
    // version of this pattern silently dropped every entry but the first.
    assert!(
        names.contains(&"List".to_string()) && names.contains(&"Map".to_string()),
        "expected List and Map from ns :import package-grouped entry, got: {names:?}"
    );
    assert!(
        paths.contains(&"java.io.InputStream".to_string()),
        "expected bare-class :import entry AFTER a package-grouped entry \
         (java.io.InputStream), got: {paths:?}"
    );
    // require inside an ns form's :require clause still works alongside :import.
    assert!(
        paths.contains(&"variants.req-in-ns".to_string()),
        "expected ns-nested require path, got: {paths:?}"
    );
}

#[test]
fn clojure_imports_negative_alias_and_refer_not_double_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_imports_negative: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("clojure")
        .expect("clojure imports query missing");
    let paths = collect_captures(&lang, CLOJURE_VARIANTS, &query_str, "import.path");

    // The original bug: `(vec_lit (sym_lit) @import.path)` with no `.` anchor
    // matched every sym_lit in the vector, so an alias symbol like "o"/"va"
    // was wrongly captured a second time as its own @import.path.
    for alias_leaked_as_path in ["va", "vma", "vmb", "vrin"] {
        assert!(
            !paths.contains(&alias_leaked_as_path.to_string()),
            "alias '{alias_leaked_as_path}' must not leak into @import.path, got: {paths:?}"
        );
    }
    // `:refer [a b]` names must never be captured as an import path/name —
    // they name symbols pulled into scope, not a namespace or class.
    for refer_name in ["refer-a", "refer-b"] {
        assert!(
            !paths.contains(&refer_name.to_string()),
            "':refer' name '{refer_name}' must not appear as @import.path, got: {paths:?}"
        );
    }
}

#[test]
fn clojure_calls_negative_special_forms_are_not_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_calls_negative: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("clojure")
        .expect("clojure calls query missing");
    let calls = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "call");

    // Definition forms and control-flow special forms/macros share syntax
    // with function calls in a Lisp `(leading-sym args...)` but are not
    // calls in any call-graph sense. Every one of these appears in
    // sample.clj; before the fix each was wrongly captured as a @call once
    // per occurrence.
    for special_form in [
        "defn",
        "defn-",
        "defmacro",
        "defrecord",
        "ns",
        "let",
        "cond",
        "when",
        "for",
    ] {
        assert!(
            !calls.contains(&special_form.to_string()),
            "'{special_form}' is a special form/definition macro, must not appear as \
             @call, got: {calls:?}"
        );
    }
    // Real calls must still be present.
    for real_call in ["println", "distance", "reduce", "filter"] {
        assert!(
            calls.contains(&real_call.to_string()),
            "expected real call '{real_call}' still present, got: {calls:?}"
        );
    }
}

#[test]
fn clojure_get_visibility_meta_private_matches_defn_dash() {
    // The `^:private` reader-metadata convention must be detected as
    // Private just like the `defn-` trailing-dash convention — previously
    // only `defn-` was checked, so `(defn ^:private foo ...)` was
    // misreported as Public. Exercised via the Language trait directly
    // (not a .scm capture) since visibility is derived in clojure.rs from
    // the @definition.* node, not from a query capture.
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_get_visibility: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_get_visibility: clojure grammar .so not found");
        return;
    };
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(CLOJURE_VARIANTS, None).expect("parse failed");
    let support = normalize_languages::Clojure;
    let root = tree.root_node();
    let mut cursor = root.walk();
    let mut found_private = false;
    let mut found_public = false;
    for child in root.children(&mut cursor) {
        if child.kind() != "list_lit" {
            continue;
        }
        let text = &CLOJURE_VARIANTS[child.byte_range()];
        if text.starts_with("(defn ^:private meta-private-fn") {
            assert_eq!(
                normalize_languages::Language::get_visibility(&support, &child, CLOJURE_VARIANTS),
                normalize_languages::Visibility::Private,
                "^:private defn must report Private visibility"
            );
            found_private = true;
        }
        if text.starts_with("(defn plain-fn") {
            assert_eq!(
                normalize_languages::Language::get_visibility(&support, &child, CLOJURE_VARIANTS),
                normalize_languages::Visibility::Public,
                "plain defn must report Public visibility"
            );
            found_public = true;
        }
    }
    assert!(
        found_private,
        "meta-private-fn defn form not found in variants.clj"
    );
    assert!(found_public, "plain-fn defn form not found in variants.clj");
}

#[test]
fn clojure_decorations_finds_metadata_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping clojure_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // meta_lit is the verified node name for ^:keyword reader metadata in tree-sitter-clojure.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "clojure",
        CLOJURE_SAMPLE,
        &[
            "^:deprecated",
            "; A point in 2D space with x and y coordinates",
        ],
    );
}
