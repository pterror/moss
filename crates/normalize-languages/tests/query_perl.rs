//! Query fixture tests for perl.
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
// Perl
// ---------------------------------------------------------------------------

const PERL_SAMPLE: &str = include_str!("fixtures/perl/sample.pl");

#[test]
fn perl_tags_finds_subroutines_and_packages() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_tags: perl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("perl").expect("perl tags query missing");
    let names = collect_captures(&lang, PERL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "sum_array" || n == "factorial"),
        "expected 'classify'/'sum_array'/'factorial' in perl tags, got: {names:?}"
    );
}

#[test]
fn perl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_calls: perl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("perl").expect("perl calls query missing");
    let calls = collect_captures(&lang, PERL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "sum_array" || c == "factorial"),
        "expected a function call in perl sample, got: {calls:?}"
    );
}

#[test]
fn perl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_complexity: perl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("perl")
        .expect("perl complexity query missing");
    let complexity = collect_captures(&lang, PERL_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in perl sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn perl_imports_finds_use_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_imports: perl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("perl")
        .expect("perl imports query missing");
    let paths = collect_captures(&lang, PERL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("List") || p.contains("POSIX") || p.contains("warnings")),
        "expected a module path in perl imports, got: {paths:?}"
    );
}

#[test]
fn perl_decorations_finds_pod_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping perl_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In tree-sitter-perl, POD documentation blocks (=head1 ... =cut) are pod nodes (not pod_statement).
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "perl",
        PERL_SAMPLE,
        &[
            "=head1 NAME",
            "# Classify a number as negative, zero, or positive",
        ],
    );
}

const PERL_VARIANTS: &str = include_str!("fixtures/perl/variants.pl");

/// `perl.calls.scm` previously only matched `function_call_expression`
/// (parenthesized calls). `ambiguous_function_call_expression` — the node
/// type for parenless calls (`print "x"`, any bareword sub invocation
/// without parens) — was entirely unmatched, silently dropping one of the
/// two most common Perl call forms. Assert both node kinds are captured.
#[test]
fn perl_calls_completeness_parenthesized_and_parenless_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_calls_completeness: perl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("perl").expect("perl calls query missing");
    let full = collect_captures_full(&lang, PERL_VARIANTS, &query_str);

    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "function" && text == "plain_func"),
        "expected function_call_expression-derived @call 'plain_func' in variants.pl, \
         got: {full:?}"
    );
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "function" && text == "print"),
        "expected ambiguous_function_call_expression-derived @call 'print' (parenless) \
         in variants.pl, got: {full:?}"
    );
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "function" && text == "bareword_no_args"),
        "expected ambiguous_function_call_expression-derived @call 'bareword_no_args' \
         (parenless, no arguments) in variants.pl, got: {full:?}"
    );
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "method" && text == "new"),
        "expected method_call_expression @call 'new' in variants.pl, got: {full:?}"
    );
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "method" && text == "render"),
        "expected method_call_expression @call 'render' in variants.pl, got: {full:?}"
    );
}

/// `require_expression` has no named field for its argument; the module
/// name (`bareword`) and file-path (`string_literal`) forms are
/// structurally distinct child node types. Before the fix, neither was
/// captured as `@import.path` at all (only the whole-statement `@import`
/// was). Assert both are now captured with the expected node kind.
#[test]
fn perl_imports_completeness_require_bareword_and_string_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_imports_completeness: perl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("perl")
        .expect("perl imports query missing");
    let full = collect_captures_full(&lang, PERL_VARIANTS, &query_str);
    let paths: Vec<_> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.path")
        .cloned()
        .collect();

    assert!(
        paths
            .iter()
            .any(|(_, kind, text, _)| kind == "bareword" && text == "Scalar::Util"),
        "expected bareword-kind @import.path 'Scalar::Util' (require Module::Name) in \
         variants.pl, got: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .any(|(_, kind, text, _)| kind == "string_literal" && text == "'legacy_helpers.pl'"),
        "expected string_literal-kind @import.path \"'legacy_helpers.pl'\" (require \
         'file.pl') in variants.pl, got: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .any(|(_, kind, text, _)| kind == "package" && text == "List::Util"),
        "expected package-kind @import.path 'List::Util' (use Module::Name) in \
         variants.pl, got: {paths:?}"
    );
}

/// `for_statement` (foreach form) and `cstyle_for_statement` (C-style
/// `for (init; cond; step)`) are structurally distinct node types.
/// `perl.complexity.scm` previously only matched `for_statement`, so every
/// C-style for-loop contributed zero complexity/nesting. Assert both loop
/// forms are now counted.
#[test]
fn perl_complexity_completeness_foreach_and_cstyle_for_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_complexity_completeness: perl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("perl")
        .expect("perl complexity query missing");
    let full = collect_captures_full(&lang, PERL_VARIANTS, &query_str);
    let complexity_kinds: Vec<_> = full
        .iter()
        .filter(|(cap, ..)| cap == "complexity")
        .map(|(_, kind, ..)| kind.as_str())
        .collect();

    assert!(
        complexity_kinds.contains(&"for_statement"),
        "expected a for_statement (foreach form) @complexity node in variants.pl, \
         got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"cstyle_for_statement"),
        "expected a cstyle_for_statement (C-style for) @complexity node in variants.pl, \
         got: {complexity_kinds:?}"
    );

    let nesting_kinds: Vec<_> = full
        .iter()
        .filter(|(cap, ..)| cap == "nesting")
        .map(|(_, kind, ..)| kind.as_str())
        .collect();
    assert!(
        nesting_kinds.contains(&"cstyle_for_statement"),
        "expected a cstyle_for_statement @nesting node in variants.pl, got: {nesting_kinds:?}"
    );
}

/// Negative case: `func1op_call_expression` (a builtin operator with fixed
/// arity, e.g. `shift`) is a structurally distinct node type whose
/// `function` field is a closed set of builtin-keyword tokens — never the
/// generic `function` bareword node either @call clause matches. Confirm
/// `shift` is not swept in as a call.
#[test]
fn perl_calls_negative_builtin_operator_not_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_calls_negative: perl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("perl").expect("perl calls query missing");
    let calls = collect_captures(&lang, PERL_VARIANTS, &query_str, "call");
    assert!(
        !calls.iter().any(|c| c == "shift"),
        "the func1op builtin 'shift' must not be captured as @call, got: {calls:?}"
    );
}

/// Real-world sample regression check: sample.pl already leans heavily on
/// parenless calls (`print "...", ...`) and now also exercises `require`
/// and a C-style for-loop. Assert the parenless calls and the new
/// constructs are captured end-to-end (not just on the synthetic
/// variants fixture).
#[test]
fn perl_calls_finds_parenless_print_in_real_world_sample() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_calls_sample: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_calls_sample: perl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("perl").expect("perl calls query missing");
    const PERL_SAMPLE: &str = include_str!("fixtures/perl/sample.pl");
    let calls = collect_captures(&lang, PERL_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "print"),
        "expected parenless 'print' calls captured in sample.pl, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "log_message"),
        "expected parenless user-sub call 'log_message' captured in sample.pl, \
         got: {calls:?}"
    );
}
