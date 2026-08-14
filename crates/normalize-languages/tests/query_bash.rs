//! Query fixture tests for bash.
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
// Bash
// ---------------------------------------------------------------------------

const BASH_SAMPLE: &str = include_str!("fixtures/bash/sample.sh");

#[test]
fn bash_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_tags: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("bash").expect("bash tags query missing");
    let names = collect_captures(&lang, BASH_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "sum_array" || n == "greet"),
        "expected 'classify'/'sum_array'/'greet' in bash tags, got: {names:?}"
    );
}

#[test]
fn bash_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_calls: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("bash").expect("bash calls query missing");
    let calls = collect_captures(&lang, BASH_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "greet" || c == "sum_array"),
        "expected a function call in bash sample, got: {calls:?}"
    );
}

#[test]
fn bash_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let complexity = collect_captures(&lang, BASH_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in bash sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn bash_imports_finds_source_commands() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_imports: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("bash")
        .expect("bash imports query missing");
    let paths = collect_captures(&lang, BASH_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils") || p.contains("config")),
        "expected sourced file path in bash imports, got: {paths:?}"
    );
}

#[test]
fn bash_complexity_finds_real_world_idioms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_complexity_real_world: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity_real_world: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let complexity = collect_captures_full(&lang, BASH_SAMPLE, &query_str);
    let kinds: Vec<&str> = complexity
        .iter()
        .filter(|(cap, ..)| cap == "complexity")
        .map(|(_, kind, ..)| kind.as_str())
        .collect();
    // sample.sh's `retry()` function exercises c_style_for_statement,
    // binary_expression (&&/||), and ternary_expression together — none of
    // which the pre-fix query captured at all.
    assert!(
        kinds.contains(&"c_style_for_statement"),
        "expected c_style_for_statement complexity in bash sample, got: {kinds:?}"
    );
    assert!(
        kinds.contains(&"binary_expression"),
        "expected &&/|| binary_expression complexity in bash sample, got: {kinds:?}"
    );
    assert!(
        kinds.contains(&"ternary_expression"),
        "expected ternary_expression complexity in bash sample, got: {kinds:?}"
    );
}

const BASH_VARIANTS: &str = include_str!("fixtures/bash/variants.sh");

#[test]
fn bash_variants_fixture_parses_clean() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping bash_variants_fixture_parses_clean: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_variants_fixture_parses_clean: bash grammar .so not found");
        return;
    };
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(BASH_VARIANTS, None).expect("parse failed");
    assert!(
        !tree.root_node().has_error(),
        "bash variants.sh fixture must parse without ERROR nodes"
    );
}

/// tags.scm completeness: both function_definition syntaxes (`function
/// NAME { }`, `function NAME() { }`, `NAME() { }`) and the non-`{ }`
/// body variant (`if_statement` as body) all yield `name: (word) @name`
/// under `@definition.function` — the same shape, verified across every
/// syntactic form the grammar allows.
#[test]
fn bash_tags_completeness_function_definition_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_tags_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("bash").expect("bash tags query missing");
    let pairs = collect_tag_pairs(&lang, BASH_VARIANTS, &query_str);
    for expected in [
        "fn_keyword_no_parens",
        "fn_keyword_with_parens",
        "fn_posix_form",
        "fn_body_if_statement",
    ] {
        assert!(
            pairs
                .iter()
                .any(|(kind, name)| kind == "definition.function" && name == expected),
            "expected {expected} as @definition.function in bash tags, got: {pairs:?}"
        );
    }
    // Negative: a function name mentioned only inside a string literal must
    // not produce a @definition.function tag.
    assert!(
        !pairs
            .iter()
            .any(|(_, name)| name.contains("calling function")),
        "string contents must not produce a tags capture, got: {pairs:?}"
    );
}

/// calls.scm completeness: every `command_name` child variant a real bash
/// script can produce (bare word, relative/absolute path, quoted string,
/// simple and braced variable expansion) yields a `@call` capture, in every
/// structural context (pipeline stage, subshell, command substitution,
/// negated_command).
#[test]
fn bash_calls_completeness_command_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_calls_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("bash").expect("bash calls query missing");
    let calls = collect_captures_full(&lang, BASH_VARIANTS, &query_str);
    let call_texts: Vec<&str> = calls
        .iter()
        .filter(|(cap, ..)| cap == "call")
        .map(|(_, _, text, _)| text.as_str())
        .collect();
    for expected in [
        "ls",          // bare word
        "./script.sh", // relative path word
        "\"ls\"",      // quoted string command_name
        "$cmd",        // simple_expansion
        "${cmd}",      // expansion (braced)
        "grep",        // pipeline stage
        "sort",        // pipeline stage
    ] {
        assert!(
            call_texts.contains(&expected),
            "expected {expected:?} among bash @call captures, got: {call_texts:?}"
        );
    }
    // Every @call capture must be a `command_name` node (extraction depth:
    // kind, not just text).
    for (cap, kind, text, line) in &calls {
        if cap == "call" {
            assert_eq!(
                kind, "command_name",
                "@call capture {text:?} at line {line} has unexpected kind {kind:?}"
            );
        }
    }
}

/// imports.scm completeness: `source`/`.` cover bare word, quoted string,
/// simple-expansion, and expansion-containing-a-string paths — and the `.`
/// field anchor means a `source file.sh arg1 arg2` command captures only
/// the path, never the trailing positional arguments.
#[test]
fn bash_imports_completeness_source_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_imports_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("bash")
        .expect("bash imports query missing");
    let paths = collect_captures(&lang, BASH_VARIANTS, &query_str, "import.path");
    assert_eq!(
        paths,
        vec![
            "./plain_path.sh",
            "./dot_path.sh",
            "\"./quoted_path.sh\"",
            "$lib_path",
            "\"$lib_path/sub.sh\"",
            "./with_args.sh",
        ],
        "expected exactly these @import.path captures (in source order), got: {paths:?}"
    );
}

/// Regression test for the trailing-argument bug: `source file.sh arg1
/// arg2` must produce exactly one @import.path capture, never one per
/// argument.
#[test]
fn bash_imports_negative_no_trailing_arguments_as_path() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping bash_imports_negative_trailing_args: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_imports_negative_trailing_args: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("bash")
        .expect("bash imports query missing");
    let source = "source ./with_args.sh arg1 arg2\n";
    let paths = collect_captures(&lang, source, &query_str, "import.path");
    assert_eq!(
        paths,
        vec!["./with_args.sh"],
        "expected exactly one @import.path (the sourced file, not trailing args), got: {paths:?}"
    );
}

/// complexity.scm completeness: every control-flow/decision node variant —
/// including c_style_for_statement, the &&/|| binary_expression operators,
/// and ternary_expression, none of which the pre-fix query captured —
/// produces exactly the expected count of @complexity/@nesting captures.
#[test]
fn bash_complexity_completeness_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let complexity = collect_captures_full(&lang, BASH_VARIANTS, &query_str);

    let count_of = |kind: &str| -> usize {
        complexity
            .iter()
            .filter(|(cap, k, ..)| cap == "complexity" && k == kind)
            .count()
    };

    assert_eq!(count_of("if_statement"), 4, "if_statement complexity count");
    assert_eq!(count_of("elif_clause"), 2, "elif_clause complexity count");
    assert_eq!(
        count_of("for_statement"),
        1,
        "for_statement complexity count"
    );
    assert_eq!(
        count_of("c_style_for_statement"),
        1,
        "c_style_for_statement complexity count (previously uncounted entirely)"
    );
    assert_eq!(
        count_of("while_statement"),
        2,
        "while_statement complexity count (covers both `while` and `until`)"
    );
    assert_eq!(
        count_of("case_statement"),
        1,
        "case_statement complexity count"
    );
    assert_eq!(count_of("case_item"), 3, "case_item complexity count");
    assert_eq!(count_of("pipeline"), 2, "pipeline complexity count");
    assert_eq!(
        count_of("list"),
        2,
        "list (&&/|| statement-level chain) complexity count"
    );
    assert_eq!(
        count_of("ternary_expression"),
        1,
        "ternary_expression complexity count (previously uncounted entirely)"
    );

    let binary_and_or = complexity
        .iter()
        .filter(|(cap, k, ..)| cap == "complexity" && k == "binary_expression")
        .count();
    assert_eq!(
        binary_and_or, 2,
        "expected exactly 2 binary_expression complexity captures (one &&, one ||); \
         plain arithmetic/comparison operators (+=, <, >=, ...) must NOT count, got: {complexity:?}"
    );
}

/// Negative regression test for the binary_expression overcounting risk:
/// ordinary arithmetic (`+=`, `<`) inside `(( ))` must contribute zero
/// complexity — only literal `&&`/`||` operators do.
#[test]
fn bash_complexity_negative_arithmetic_not_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping bash_complexity_negative_arithmetic: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity_negative_arithmetic: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let source = "negative_arithmetic_not_complexity() {\n    local -i total=0\n    (( total += 1 ))\n    (( total < 100 ))\n}\n";
    let complexity = collect_captures_full(&lang, source, &query_str);
    let binary_hits: Vec<_> = complexity
        .iter()
        .filter(|(cap, k, ..)| cap == "complexity" && k == "binary_expression")
        .collect();
    assert!(
        binary_hits.is_empty(),
        "plain arithmetic (+=, <) must not produce binary_expression complexity, got: {binary_hits:?}"
    );
}
