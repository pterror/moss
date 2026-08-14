//! Query fixture tests for r.
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

const R_SAMPLE: &str = include_str!("fixtures/r/sample.r");

#[test]
fn r_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "r",
        R_SAMPLE,
        &["# Classify a number"],
    );
}

#[test]
fn r_calls_finds_local_and_namespace_qualified_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping r_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_calls: r grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("r").expect("r calls query missing");
    let calls = collect_captures(&lang, R_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"classify".to_string()),
        "expected local 'classify' call in r calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"median".to_string()),
        "expected namespace-qualified 'stats::median' call to capture 'median' in r calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, R_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"stats".to_string()),
        "expected 'stats' qualifier in r calls, got: {qualifiers:?}"
    );
}

// ==================== r query tests ====================

const R_VARIANTS: &str = include_str!("fixtures/r/variants.r");

#[test]
fn r_tags_finds_dollar_assigned_and_right_assigned_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping r_tags_dollar_right: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_tags_dollar_right: r grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("r").expect("r tags query missing");
    let pairs = collect_tag_pairs(&lang, R_SAMPLE, &query_str);
    // Environment/`$`-based method definitions inside make_stack() (the
    // pre-R6-package OOP idiom: self$push <- function(...) {...}).
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "push"),
        "expected 'push' (self$push <- function...) as definition.function, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "pop"),
        "expected 'pop' (self$pop <- function...) as definition.function, got: {pairs:?}"
    );
    // Right-assignment: (function(x) x * x) -> square.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "square"),
        "expected 'square' (right-assigned function) as definition.function, got: {pairs:?}"
    );
}

#[test]
fn r_tags_completeness_all_assignment_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_tags_completeness: r grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("r").expect("r tags query missing");
    let pairs = collect_tag_pairs(&lang, R_VARIANTS, &query_str);
    for name in [
        "left_arrow_fn",         // lhs: identifier, operator "<-"
        "equals_fn",             // lhs: identifier, operator "="
        "inner",                 // lhs: identifier, operator "<<-"
        "method_dollar",         // lhs: extract_operator ($), operator "<-"
        "method_dollar_eq",      // lhs: extract_operator ($), operator "="
        "right_arrow_fn",        // lhs: parenthesized function, operator "->"
        "right_arrow_global_fn", // lhs: parenthesized function, operator "->>"
        "lambda_fn",             // rhs: function_definition with `\` name form
    ] {
        assert!(
            pairs
                .iter()
                .any(|(k, n)| k == "definition.function" && n == name),
            "expected '{name}' as definition.function in variants.r, got: {pairs:?}"
        );
    }
}

#[test]
fn r_calls_finds_method_and_bracket_style_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping r_calls_method_bracket: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_calls_method_bracket: r grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("r").expect("r calls query missing");
    let calls = collect_captures(&lang, R_SAMPLE, &query_str, "call");
    // stack$push(42) / stack$pop() — extract_operator ($) call form.
    assert!(
        calls.contains(&"push".to_string()),
        "expected 'push' method-style call (stack$push(...)) in r calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"pop".to_string()),
        "expected 'pop' method-style call (stack$pop()) in r calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, R_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"stack".to_string()),
        "expected 'stack' qualifier for stack$push/stack$pop, got: {qualifiers:?}"
    );
}

#[test]
fn r_calls_completeness_all_function_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_calls_completeness: r grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("r").expect("r calls query missing");
    let caps = collect_captures_full(&lang, R_VARIANTS, &query_str);

    let calls: Vec<(&str, &str)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, k, t, _)| (k.as_str(), t.as_str()))
        .collect();

    // function: (identifier) — plain call
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "identifier" && *t == "plain_call_target"),
        "expected plain identifier call 'plain_call_target', got: {calls:?}"
    );
    // function: (namespace_operator) — pkg::fn()
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "identifier" && *t == "median"),
        "expected namespace-qualified call 'median' (stats::median), got: {calls:?}"
    );
    // function: (extract_operator) via `$` — method-style call
    assert!(
        calls.iter().any(|(k, t)| *k == "identifier" && *t == "run"),
        "expected dollar method-style call 'run' (receiver_env$run()), got: {calls:?}"
    );
    // function: (subset2) — obj[["name"]]() call, whole node captured (no
    // resolvable static name, so kind must be 'subset2', not 'identifier').
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "subset2" && t.contains("bracket2_holder")),
        "expected subset2 call-site capture for bracket2_holder[[\"fn\"]](), got: {calls:?}"
    );
    // function: (subset) — obj["name"]() call, same rationale.
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "subset" && t.contains("bracket1_holder")),
        "expected subset call-site capture for bracket1_holder[\"fn\"](), got: {calls:?}"
    );

    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"stats"),
        "expected 'stats' namespace qualifier, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"receiver_env"),
        "expected 'receiver_env' dollar-call qualifier, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"bracket2_holder"),
        "expected 'bracket2_holder' subset2-call qualifier, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"bracket1_holder"),
        "expected 'bracket1_holder' subset-call qualifier, got: {qualifiers:?}"
    );
}

#[test]
fn r_calls_negative_field_and_bracket_reads_not_captured() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_calls_negative: r grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("r").expect("r calls query missing");
    let calls = collect_captures(&lang, R_VARIANTS, &query_str, "call");
    // `receiver_env$run` (bare field read, not `receiver_env$run()`) must
    // never surface as a call. The only "run" call comes from the earlier
    // `receiver_env$run()` call-site above the negative section.
    assert_eq!(
        calls.iter().filter(|c| c.as_str() == "run").count(),
        1,
        "expected exactly 1 'run' call (the actual call site, not the bare \
         field read negative_field_read <- receiver_env$run), got: {calls:?}"
    );
    // Bracket-subscript reads (not calls) must not add extra subset/subset2
    // call-site captures beyond the two real call sites above.
    let full = collect_captures_full(&lang, R_VARIANTS, &query_str);
    let subset2_calls = full
        .iter()
        .filter(|(cn, k, _, _)| cn == "call" && k == "subset2")
        .count();
    assert_eq!(
        subset2_calls, 1,
        "expected exactly 1 subset2 @call (bracket2_holder[[\"fn\"]]()), \
         negative_bracket_read <- bracket2_holder[[\"fn\"]] must not count, got {subset2_calls}"
    );
    let subset_calls = full
        .iter()
        .filter(|(cn, k, _, _)| cn == "call" && k == "subset")
        .count();
    assert_eq!(
        subset_calls, 1,
        "expected exactly 1 subset @call (bracket1_holder[\"fn\"]()), \
         negative_bracket1_read <- bracket1_holder[\"fn\"] must not count, got {subset_calls}"
    );
}

#[test]
fn r_imports_finds_all_library_require_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_imports_variants: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_imports_variants: r grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("r").expect("r imports query missing");
    let paths = collect_captures(&lang, R_VARIANTS, &query_str, "import.path");
    for expected in [
        "utils",       // library(pkg) bareword
        "\"tools\"",   // library("pkg") quoted
        "\"methods\"", // library(package = "pkg") named argument
        "grDevices",   // require(pkg)
        "\"grid\"",    // requireNamespace("pkg")
    ] {
        assert!(
            paths.iter().any(|p| p == expected),
            "expected import.path {expected:?} in variants.r, got: {paths:?}"
        );
    }
}

#[test]
fn r_imports_negative_named_option_and_non_import_calls_not_captured() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_imports_negative: r grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("r").expect("r imports query missing");
    let paths = collect_captures(&lang, R_VARIANTS, &query_str, "import.path");
    // library(utils, character.only = TRUE) must contribute exactly ONE
    // import.path ("utils"), never "character.only = TRUE" or the bare
    // comma token that separates the arguments (both were real bugs before
    // the `arguments: (arguments . (argument value: (_) @import.path))`
    // fix — an unanchored `(_)` matched every child of `arguments`,
    // including the named-option argument and the comma node, since R's
    // grammar marks `comma` as a *named* node type).
    assert!(
        !paths.iter().any(|p| p.contains("character.only")),
        "named option argument must never be captured as import.path, got: {paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p == ","),
        "comma separator token must never be captured as import.path, got: {paths:?}"
    );
    assert_eq!(
        paths.iter().filter(|p| p.as_str() == "utils").count(),
        2,
        "expected exactly 2 'utils' import.path captures (bareword library(utils) \
         and library(utils, character.only = TRUE)), got: {paths:?}"
    );
    // `library` used as a bare value (not called), and `loadNamespace(...)`
    // (a real but different R import mechanism the query intentionally
    // doesn't special-case), must not contribute any import.path captures:
    // the total stays at exactly the 6 accounted for above (utils x2,
    // "tools", "methods", grDevices, "grid").
    assert_eq!(
        paths.len(),
        6,
        "expected exactly 6 import.path captures in variants.r (library_as_value \
         and loadNamespace(...) must not contribute any), got: {paths:?}"
    );
}

#[test]
fn r_complexity_completeness_all_branch_loop_and_logical_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_complexity_completeness: r grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("r")
        .expect("r complexity query missing");
    let caps = collect_captures_full(&lang, R_VARIANTS, &query_str);

    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    for kind in [
        "if_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
        "&&",
        "||",
    ] {
        assert!(
            complexity_kinds.contains(&kind),
            "expected a @complexity capture of kind '{kind}' in variants.r, \
             got kinds: {complexity_kinds:?}"
        );
    }

    let nesting_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "nesting")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    for kind in [
        "if_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
        "function_definition",
    ] {
        assert!(
            nesting_kinds.contains(&kind),
            "expected a @nesting capture of kind '{kind}' in variants.r, got kinds: \
             {nesting_kinds:?}"
        );
    }
    // "&&" / "||" must never appear as @nesting (only if/for/while/repeat/
    // function_definition nest; logical operators only add complexity).
    assert!(
        !nesting_kinds.contains(&"&&") && !nesting_kinds.contains(&"||"),
        "logical operators must not contribute @nesting, got: {nesting_kinds:?}"
    );
}
