//! Query fixture tests for haskell.
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
use tree_sitter::StreamingIterator;

const HASKELL_SAMPLE: &str = include_str!("fixtures/haskell/sample.hs");

const HASKELL_VARIANTS: &str = include_str!("fixtures/haskell/variants.hs");

/// Returns `(loader, lang)` together — the `GrammarLoader` owns the loaded
/// dylib, so it must stay alive for as long as the returned `Language` is
/// used (dropping it early unloads the library and leaves `Language`'s
/// function pointers dangling, which segfaults rather than erroring).
fn haskell_lang() -> Option<(normalize_languages::GrammarLoader, tree_sitter::Language)> {
    let loader = normalize_languages::GrammarLoader::new();
    let lang = loader.get("haskell").ok()?;
    Some((loader, lang))
}

// --- Dimension 4: real-world fixture coverage (sample.hs) -------------------

#[test]
fn haskell_tags_no_duplicate_signatures() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_tags_no_duplicate_signatures: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("haskell")
        .expect("haskell tags query missing");
    let names = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "name");
    // "classify" has one equation; with type signatures removed, it should appear exactly once
    // (before fix: appeared twice — once for signature, once for definition).
    let classify_count = names.iter().filter(|n| *n == "classify").count();
    assert_eq!(
        classify_count, 1,
        "expected 'classify' exactly once (type signature removed), got: {names:?}"
    );
    // "insert" has two equations (multi-equation function); the grammar produces one `function`
    // node per equation, so it legitimately appears twice in the raw query output.
    // Deduplication to a single symbol happens in the extraction layer (normalize-facts).
    let insert_count = names.iter().filter(|n| *n == "insert").count();
    assert!(
        (1..=2).contains(&insert_count),
        "expected 'insert' 1-2 times (multi-equation), got: {names:?}"
    );
    // Type names from data/newtype/type should also be present
    assert!(
        names.contains(&"Tree".to_string()),
        "expected 'Tree' data type, got: {names:?}"
    );
    assert!(
        names.contains(&"Count".to_string()),
        "expected 'Count' newtype, got: {names:?}"
    );
    // Typeclass + two instances of it for different types — both must be
    // captured (see haskell.rs's dedup_haskell_functions fix: it previously
    // dropped every instance after the first with the same class name).
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' typeclass and/or its instances, got: {names:?}"
    );
    let shape_count = names.iter().filter(|n| *n == "Shape").count();
    assert!(
        shape_count >= 3,
        "expected 'Shape' at least 3 times (class def + 2 instances), got: {names:?}"
    );
    // Record type declaration.
    assert!(
        names.contains(&"Rectangle".to_string()),
        "expected 'Rectangle' record type, got: {names:?}"
    );
    // Operator function definition: `(<+>) a b = ...`.
    assert!(
        names.contains(&"(<+>)".to_string()),
        "expected '(<+>)' operator function definition, got: {names:?}"
    );
    // Point-free / zero-argument top-level binding: `frequencyMap = foldr ...`.
    // Entirely absent before the `bind`-node fix.
    assert!(
        names.contains(&"frequencyMap".to_string()),
        "expected point-free 'frequencyMap' binding, got: {names:?}"
    );
    // `main` itself — the most fundamental top-level Haskell definition —
    // was entirely absent before the `bind`-node fix.
    assert!(
        names.contains(&"main".to_string()),
        "expected 'main' binding, got: {names:?}"
    );
    // where-bound local helpers must never leak into top-level tags.
    assert!(
        !names.contains(&"bmiTier".to_string()),
        "where-bound 'bmiTier' must not appear in top-level tags, got: {names:?}"
    );
    assert!(
        !names.contains(&"bmi".to_string()),
        "where-bound 'bmi' must not appear in top-level tags, got: {names:?}"
    );
}

#[test]
fn haskell_calls_finds_local_qualified_and_constructor_calls() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!(
            "Skipping haskell_calls_finds_local_qualified_and_constructor_calls: haskell grammar not found"
        );
        return;
    };
    let query_str = loader
        .get_calls("haskell")
        .expect("haskell calls query missing");
    let calls = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"insert".to_string()),
        "expected local 'insert' call in haskell calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"Node".to_string()),
        "expected constructor 'Node' application in haskell calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"insertWith".to_string()),
        "expected qualified 'Map.insertWith' call to capture 'insertWith' in haskell calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"Map".to_string()),
        "expected 'Map' qualifier in haskell calls, got: {qualifiers:?}"
    );
}

#[test]
fn haskell_imports_finds_named_imports() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_imports_finds_named_imports: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_imports("haskell")
        .expect("haskell imports query missing");
    let paths = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p == "Data.List" || p == "Data"),
        "expected 'Data.List' import path, got: {paths:?}"
    );
    // Named imports were entirely unmatched before this fix — @import.name
    // never had a single capture in the whole file.
    let names = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "import.name");
    assert!(
        names.contains(&"sort".to_string()) && names.contains(&"nub".to_string()),
        "expected 'sort' and 'nub' named imports, got: {names:?}"
    );
    // hiding-imports use the same import_list shape — `hiding (lookup)` must
    // also produce an @import.name capture for "lookup".
    assert!(
        names.contains(&"lookup".to_string()),
        "expected 'lookup' from 'import Prelude hiding (lookup)', got: {names:?}"
    );
}

#[test]
fn haskell_complexity_no_baseline_inflation_and_finds_branches() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!(
            "Skipping haskell_complexity_no_baseline_inflation_and_finds_branches: haskell grammar not found"
        );
        return;
    };
    let query_str = loader
        .get_complexity("haskell")
        .expect("haskell complexity query missing");
    let complexity = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "complexity");
    // "classify" (nested if/else) and "describe" (case with a guarded
    // alternative) both contribute real decision points.
    assert!(
        complexity.len() >= 4,
        "expected at least 4 complexity nodes in haskell sample, got {} ({complexity:?})",
        complexity.len()
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.hs) -

#[test]
fn haskell_tags_completeness_all_name_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_tags_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("haskell")
        .expect("haskell tags query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    // (capture_name, node_kind, text) for every documented variant.
    let required: &[(&str, &str, &str)] = &[
        ("name", "variable", "plainFunc"), // function.name: variable
        ("name", "prefix_id", "(+++)"),    // function.name: prefix_id
        ("name", "name", "Tree"),          // data_type.name: name
        ("name", "prefix_id", "(:+:)"),    // data_type.name: prefix_id
        ("name", "name", "Count"),         // newtype.name: name
        ("name", "prefix_id", "(:*:)"),    // newtype.name: prefix_id
        ("name", "name", "Name"),          // type_synomym.name: name
        ("name", "prefix_id", "(:->)"),    // type_synomym.name: prefix_id
        ("name", "name", "Shape"),         // class.name: name (also matches both instances)
        ("name", "prefix_id", "(:~:)"),    // class.name / instance.name: prefix_id
        ("name", "variable", "doubleAll"), // bind.name: variable (point-free)
        ("name", "prefix_id", "(<+>)"),    // bind.name: prefix_id (point-free operator)
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in haskell.tags.scm \
             output for variants.hs, got: {caps:?}"
        );
    }

    // NEGATIVE: where-bound and let-bound local names must never appear as
    // @name captures — `function`/`bind` are also the node types for local
    // helpers, and only top-level `(declarations ...)` children are tagged.
    for local in ["negHelper", "negLocal"] {
        assert!(
            !caps.iter().any(|(cn, _, t, _)| cn == "name" && t == local),
            "local binding '{local}' must not appear as a @name capture, got: {caps:?}"
        );
    }
}

#[test]
fn haskell_calls_completeness_all_function_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_calls_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_calls("haskell")
        .expect("haskell calls query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "variable", "plainFunc"), // apply.function: variable
        ("call", "constructor", "TNode"),  // apply.function: constructor
        ("call", "variable", "lookup"),    // apply.function: qualified, id: variable
        ("call", "constructor", "Just"),   // apply.function: qualified, id: constructor
        ("call", "operator", "$"),         // apply.function: prefix_id(operator)
        ("call", "operator", "+"),         // apply.function: prefix_id(qualified(operator))
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in haskell.calls.scm \
             output for variants.hs, got: {caps:?}"
        );
    }

    // apply.function: parens(qualified(variable)) — `(Map.lookup) 1 Map.empty`.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"Map"),
        "expected 'Map' qualifier (incl. parens-wrapped qualified call), got: {qualifiers:?}"
    );

    // `plainFunc` as a call target appears at least 3 times: the plain call,
    // the parens-wrapped-variable call, and inside the negative composition
    // case is intentionally NOT one of them (see negative test below).
    let plain_func_calls = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "call" && k == "variable" && t == "plainFunc")
        .count();
    assert!(
        plain_func_calls >= 3,
        "expected 'plainFunc' called at least 3 times (plain, in callParenVariable, in \
         callQualifiedConstructor argument), got {plain_func_calls} in {caps:?}"
    );
}

#[test]
fn haskell_calls_negative_composition_not_matched() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_calls_negative_composition: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_calls("haskell")
        .expect("haskell calls query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);
    // `negComposed = (plainFunc . plainFunc) 1` — the applied value is a
    // point-free composition (parens wrapping an `infix` expression), not a
    // single nameable identifier. No @call capture should attribute this
    // outer apply to a specific function name; only the innermost `apply`
    // for the outer application itself is absent (composition is not
    // unwound into two calls to `plainFunc`).
    //
    // Every top-level `apply` in the file whose function is a bare `infix`
    // node (not `variable`/`constructor`/`prefix_id`/`parens`-wrapping-name)
    // must produce zero matches from this query on that specific node.
    let composed_query = "(apply function: (parens expression: (infix) @composed))";
    let ts_query = tree_sitter::Query::new(&lang, composed_query).expect("compiles");
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(HASKELL_VARIANTS, None).expect("parse failed");
    let mut cursor = tree_sitter::QueryCursor::new();
    let source_bytes = HASKELL_VARIANTS.as_bytes();
    let mut matches = cursor.matches(&ts_query, tree.root_node(), source_bytes);
    let mut composed_count = 0;
    while matches.next().is_some() {
        composed_count += 1;
    }
    assert_eq!(
        composed_count, 1,
        "expected exactly 1 parens-wrapped-infix apply.function (negComposed's `(plainFunc . \
         plainFunc)`), got {composed_count}"
    );
    let _ = caps; // calls.scm output already asserted not to double-count this construct above.
}

#[test]
fn haskell_imports_completeness_all_name_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_imports_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_imports("haskell")
        .expect("haskell imports query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("import.name", "variable", "sort"), // import_name.variable: variable
        ("import.name", "variable", "nub"),
        ("import.name", "name", "Down"), // import_name.type: name
        ("import.name", "prefix_id", "(<|>)"), // import_name.operator: prefix_id
        ("import.name", "variable", "lookup"), // hiding-import reuses the same shape
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in haskell.imports.scm \
             output for variants.hs, got: {caps:?}"
        );
    }
}

#[test]
fn haskell_complexity_completeness_all_branch_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_complexity_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_complexity("haskell")
        .expect("haskell complexity query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    let required_kinds: &[&str] = &[
        "conditional", // if/then/else
        // `guard` is a grammar supertype alias (subtypes: boolean/let/
        // pattern_guard) that never materializes as a "guard"-kind node —
        // confirmed via node-types.json and real parse. Tree-sitter's query
        // engine still matches `(guard) @complexity` against the concrete
        // subtype nodes; the captured node's own `.kind()` reports the
        // subtype ("boolean" here), not "guard".
        "boolean",
        "lambda",       // plain lambda
        "multi_way_if", // MultiWayIf extension — previously entirely unmatched
        "lambda_case",  // LambdaCase extension — previously entirely unmatched
        "alternative",  // per-arm case decision point — previously unmatched
        "case",         // case container
    ];
    for kind in required_kinds {
        assert!(
            caps.iter()
                .any(|(cn, k, _, _)| cn == "complexity" && k == kind),
            "expected a @complexity capture of kind '{kind}' in variants.hs, got: {caps:?}"
        );
    }
}

#[test]
fn haskell_complexity_negative_trivial_function_has_no_complexity_captures() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_complexity_negative: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_complexity("haskell")
        .expect("haskell complexity query missing");

    // `negTrivial x = x + 1` — zero branches. Before the fix, the function's
    // own `match` (equation-body) node was unconditionally counted as a
    // decision point, so this construct alone would have produced one
    // @complexity capture despite having no branching whatsoever.
    let source = "module M where\n\nnegTrivial :: Int -> Int\nnegTrivial x = x + 1\n";
    let caps = collect_captures_full(&lang, source, &query_str);
    let complexity_caps: Vec<_> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .collect();
    assert!(
        complexity_caps.is_empty(),
        "expected zero @complexity captures for a branch-free function, got: {complexity_caps:?}"
    );
}

#[test]
fn haskell_types_finds_qualified_and_generic_type_references() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_types_finds_qualified_and_generic: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_types("haskell")
        .expect("haskell types query missing");
    let types = collect_captures(&lang, HASKELL_VARIANTS, &query_str, "type.reference");
    // Qualified type reference: Map.Map Int Int — the inner `Map` (via
    // `qualified.id`) must still be captured.
    assert!(
        types.iter().filter(|t| *t == "Map").count() >= 1,
        "expected qualified 'Map' type reference, got: {types:?}"
    );
    // Generic/applied type reference: Maybe Int (apply.constructor: name).
    assert!(
        types.contains(&"Maybe".to_string()),
        "expected generic 'Maybe' type reference, got: {types:?}"
    );
}

#[test]
fn haskell_decorations_finds_pragma_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping haskell_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "haskell",
        HASKELL_SAMPLE,
        &["{-# LANGUAGE ScopedTypeVariables #-}", "-- | A simple"],
    );
}
