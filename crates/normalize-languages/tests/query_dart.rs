//! Query fixture tests for dart.
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
// Dart
// ---------------------------------------------------------------------------

const DART_SAMPLE: &str = include_str!("fixtures/dart/sample.dart");

#[test]
fn dart_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_tags: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("dart").expect("dart tags query missing");
    let names = collect_captures(&lang, DART_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in dart tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in dart tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in dart tags, got: {names:?}"
    );
}

#[test]
fn dart_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_calls: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("dart").expect("dart calls query missing");
    let calls = collect_captures(&lang, DART_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"print".to_string()) || calls.contains(&"push".to_string()),
        "expected 'print' or 'push' call in dart sample, got: {calls:?}"
    );
}

#[test]
fn dart_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_imports: dart grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("dart")
        .expect("dart imports query missing");
    let paths = collect_captures(&lang, DART_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("collection") || p.contains("dart")),
        "expected dart library path in dart import paths, got: {paths:?}"
    );
}

#[test]
fn dart_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_complexity: dart grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("dart")
        .expect("dart complexity query missing");
    let complexity = collect_captures(&lang, DART_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in dart sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn dart_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_types: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_types("dart").expect("dart types query missing");
    let refs = collect_captures(&lang, DART_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Point" || r == "int" || r == "String"),
        "expected type identifiers in dart sample, got: {refs:?}"
    );
}

const DART_VARIANTS: &str = include_str!("fixtures/dart/variants.dart");

#[test]
fn dart_tags_completeness_all_constructor_and_operator_forms() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_tags_completeness: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("dart").expect("dart tags query missing");
    let pairs = collect_tag_pairs(&lang, DART_VARIANTS, &query_str);

    // Unnamed/named constructor_signature, named factory_constructor_signature
    // (plain and arrow-body), named constant_constructor_signature, and named
    // redirecting_factory_constructor_signature — none of these node kinds
    // are function_signature/getter_signature/setter_signature, so they were
    // entirely untagged before.
    for name in ["Widget", "named", "fromId", "zero", "constUnnamed", "make"] {
        assert!(
            pairs.contains(&("definition.method".to_string(), name.to_string())),
            "expected @definition.method for constructor '{name}', got: {pairs:?}"
        );
    }

    // Operator overloads: binary (+, ==), unary (-), and index get/set
    // ([], []=) — the index forms are anonymous tokens, not identifiers.
    for op in ["+", "-", "==", "[]", "[]="] {
        assert!(
            pairs.contains(&("definition.method".to_string(), op.to_string())),
            "expected @definition.method for operator '{op}', got: {pairs:?}"
        );
    }

    // Pre-existing kinds must still work alongside the new ones.
    for (kind, name) in [
        ("definition.class", "Widget"),
        ("definition.class", "Direction"),
        ("definition.interface", "Flying"),
        ("reference.implementation", "IntExtras"),
        ("definition.function", "addOne"),
        ("definition.method", "value"), // getter and setter share this name
    ] {
        assert!(
            pairs.contains(&(kind.to_string(), name.to_string())),
            "expected @{kind} for '{name}', got: {pairs:?}"
        );
    }
}

#[test]
fn dart_calls_completeness_method_and_chained_calls() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_calls_completeness: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("dart").expect("dart calls query missing");
    let calls = collect_captures(&lang, DART_VARIANTS, &query_str, "call");

    // Bare call, member call (math.sqrt), chained calls (map().toList()),
    // generic method call (sort<int>()), named-constructor-style call
    // (Widget.fromId), and null-aware call (maybe?.toString()) — before this
    // fix only the bare-identifier form matched at all.
    for expected in [
        "addOne", "sqrt", "map", "toList", "sort", "fromId", "toString",
    ] {
        assert!(
            calls.iter().any(|c| c == expected),
            "expected call '{expected}' in dart calls, got: {calls:?}"
        );
    }
}

#[test]
fn dart_calls_negative_property_access_is_not_a_call() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_calls_negative: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("dart").expect("dart calls query missing");
    let calls = collect_captures(&lang, DART_VARIANTS, &query_str, "call");
    assert!(
        !calls.iter().any(|c| c == "id"),
        "'w.id' is a property read with no call selector, must not appear as @call: {calls:?}"
    );
}

#[test]
fn dart_imports_completeness_part_directives() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_imports_completeness: dart grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("dart")
        .expect("dart imports query missing");
    let paths = collect_captures(&lang, DART_VARIANTS, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("variants_part.dart")),
        "expected part directive path in dart imports, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("dart:collection")),
        "expected regular import path to still work, got: {paths:?}"
    );

    // part_of_directive can't coexist with `library`/`part` in the same
    // file (part-of files declare which library they belong to and must
    // not themselves declare one), so it's exercised as a standalone
    // source string rather than folded into variants.dart.
    let part_of_source = "part of 'main.dart';\n";
    let part_of_paths = collect_captures(&lang, part_of_source, &query_str, "import.path");
    assert_eq!(
        part_of_paths,
        vec!["'main.dart'"],
        "expected part_of_directive's URI form to produce @import.path, got: {part_of_paths:?}"
    );
}

#[test]
fn dart_complexity_finds_switch_expression_arms() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_complexity_switch_expr: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_complexity_switch_expr: dart grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("dart")
        .expect("dart complexity query missing");
    let captures = collect_captures_full(&lang, DART_VARIANTS, &query_str);
    let switch_expr_arms = captures
        .iter()
        .filter(|(cap, kind, ..)| cap == "complexity" && kind == "switch_expression_case")
        .count();
    assert_eq!(
        switch_expr_arms, 3,
        "expected 3 switch_expression_case arms counted as @complexity, got: {captures:?}"
    );
    let if_null = captures
        .iter()
        .any(|(cap, kind, ..)| cap == "complexity" && kind == "if_null_expression");
    assert!(
        if_null,
        "expected if_null_expression (??) counted as @complexity like && / ||, got: {captures:?}"
    );
}

#[test]
fn dart_cfg_finds_switch_expression_match() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_cfg_switch_expr: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_cfg_switch_expr: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_cfg("dart").expect("dart cfg query missing");
    let arms = collect_captures(&lang, DART_VARIANTS, &query_str, "cfg.match.arm");
    assert!(
        arms.iter().any(|a| a.contains("'zero'")),
        "expected a switch_expression arm among cfg.match.arm captures, got: {arms:?}"
    );
    // Both the plain switch_statement and the switch_expression must
    // produce match arms — the fix must not regress the pre-existing form.
    assert!(
        arms.iter().any(|a| a.starts_with("case 0")),
        "expected the plain switch_statement's case arm to still be found, got: {arms:?}"
    );
}

#[test]
fn dart_refactor_completeness_constructor_and_operator_function_defs() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_refactor_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_refactor_completeness: dart grammar .so not found");
        return;
    };
    let query_str = loader
        .get_refactor("dart")
        .expect("dart refactor query missing");
    let captures = collect_captures_full(&lang, DART_VARIANTS, &query_str);
    let function_defs: Vec<&str> = captures
        .iter()
        .filter(|(cap, ..)| cap == "refactor.function_def")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    for expected_substr in [
        "Widget(this.id)",
        "Widget.named(this.id)",
        "factory Widget.fromId(int id)",
        "const Widget.constUnnamed(int x)",
        "operator +(Widget other)",
        "operator []=(int i, int v)",
        "factory RedirectingWidget.make(int id) = RedirectingWidget",
    ] {
        assert!(
            function_defs.iter().any(|f| f.contains(expected_substr)),
            "expected a @refactor.function_def containing '{expected_substr}', got: {function_defs:?}"
        );
    }
}

#[test]
fn dart_decorations_finds_annotation_and_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "dart",
        DART_SAMPLE,
        &["@pragma", "/// Classify"],
    );
}
