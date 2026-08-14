//! Query fixture tests for vim.
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
// Vim script
// ---------------------------------------------------------------------------

const VIM_SAMPLE: &str = include_str!("fixtures/vim/sample.vim");

#[test]
fn vim_tags_finds_functions_and_augroups() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_tags: vim grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vim").expect("vim tags query missing");
    let names = collect_captures(&lang, VIM_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "ToggleOption" || n == "FormatBuffer" || n == "OpenTerminal"),
        "expected function names in vim tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "MyPlugin" || n == "FileTypeSettings"),
        "expected augroup names in vim tags, got: {names:?}"
    );
}

#[test]
fn vim_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_calls: vim grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vim").expect("vim calls query missing");
    let calls = collect_captures(&lang, VIM_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "FormatBuffer" || c == "getpos" || c == "setpos"),
        "expected function calls in vim sample, got: {calls:?}"
    );
}

#[test]
fn vim_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_complexity: vim grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vim")
        .expect("vim complexity query missing");
    let complexity = collect_captures(&lang, VIM_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (if block) in vim sample, got: {complexity:?}"
    );
}

#[test]
fn vim_imports_finds_source_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_imports: vim grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("vim")
        .expect("vim imports query missing");
    let paths = collect_captures(&lang, VIM_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils.vim") || p.contains("defaults.vim")),
        "expected sourced file paths in vim imports, got: {paths:?}"
    );
}

// ==================== vim / perl query tests ====================
// Query-completeness sweep per docs/query-testing-methodology.md, applied
// to vim.{tags,calls,imports,complexity}.scm and
// perl.{tags,calls,imports,complexity}.scm. Field constraints were
// cross-referenced against arborium-vim-2.17.0 and arborium-perl-2.17.0's
// node-types.json, then every candidate gap was verified against real
// parse output via `normalize syntax query`/`normalize syntax ast` before
// being treated as a bug. See vim.{calls,tags,imports}.scm and
// perl.{calls,imports,complexity}.scm for the per-fix rationale comments.

const VIM_VARIANTS: &str = include_str!("fixtures/vim/variants.vim");

/// `call_expression.function` allows a large set of expression node types
/// in node-types.json, but only `identifier`, `scoped_identifier`,
/// `field_expression` (dict-bound method calls), and `index_expression`
/// (dynamic dispatch-table calls) were ever verified to actually occur as
/// a call target. This asserts each of the four handled variants produces
/// a capture of the expected node kind.
#[test]
fn vim_calls_completeness_function_field_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_calls_completeness: vim grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vim").expect("vim calls query missing");
    let full = collect_captures_full(&lang, VIM_VARIANTS, &query_str);

    // identifier: plain call
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "identifier" && text == "PlainFunc"),
        "expected identifier-kind @call 'PlainFunc' in variants.vim, got: {full:?}"
    );
    // scoped_identifier's nested identifier: scoped call
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "identifier" && text == "ScopedFunc"),
        "expected identifier-kind @call 'ScopedFunc' (from scoped_identifier) in \
         variants.vim, got: {full:?}"
    );
    // field_expression: dict-bound method call
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "field_expression" && text == "s:obj.FieldFunc"),
        "expected field_expression-kind @call 's:obj.FieldFunc' in variants.vim, got: {full:?}"
    );
    // index_expression: dynamic dispatch-table call
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "index_expression" && text == "s:dispatch['go']"),
        "expected index_expression-kind @call \"s:dispatch['go']\" in variants.vim, \
         got: {full:?}"
    );
    // `->` method-chain calls still resolve through the plain identifier
    // clause (each link is a nested call_expression whose function is an
    // ordinary identifier, not the method_expression wrapper itself).
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "identifier" && text == "filter"),
        "expected identifier-kind @call 'filter' from the -> chain in variants.vim, \
         got: {full:?}"
    );
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "identifier" && text == "join"),
        "expected identifier-kind @call 'join' from the -> chain in variants.vim, \
         got: {full:?}"
    );
}

/// `function_declaration.name` allows `field_expression` in addition to
/// `identifier`/`scoped_identifier` — the dict-bound-method function
/// definition pattern (`function! obj.Method() dict`). Assert all three
/// produce a `@definition.function` capture with the expected `@name` kind.
#[test]
fn vim_tags_completeness_function_declaration_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_tags_completeness: vim grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vim").expect("vim tags query missing");
    let full = collect_captures_full(&lang, VIM_VARIANTS, &query_str);
    let names: Vec<_> = full
        .iter()
        .filter(|(cap, ..)| cap == "name")
        .cloned()
        .collect();

    assert!(
        names
            .iter()
            .any(|(_, kind, text, _)| kind == "identifier" && text == "PlainFunc"),
        "expected identifier-kind @name 'PlainFunc' in variants.vim, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|(_, kind, text, _)| kind == "scoped_identifier" && text == "s:ScopedFunc"),
        "expected scoped_identifier-kind @name 's:ScopedFunc' in variants.vim, got: {names:?}"
    );
    assert!(
        names.iter().any(|(_, kind, text, _)| kind
            == "scoped_identifier"
            && text == "foo#bar#AutoloadFunc"),
        "expected scoped_identifier-kind @name 'foo#bar#AutoloadFunc' in variants.vim, \
         got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|(_, kind, text, _)| kind == "field_expression" && text == "s:obj.FieldFunc"),
        "expected field_expression-kind @name 's:obj.FieldFunc' in variants.vim, \
         got: {names:?}"
    );
}

/// Regression test for the bang (`source!`/`runtime!`) false-positive:
/// before the fix, an unconstrained `(_)` child pattern captured the
/// `bang` node too, producing a spurious `@import.path` whose text was
/// literally `"!"`. Also asserts the real paths on both bang and
/// non-bang forms are still captured.
#[test]
fn vim_imports_negative_bang_not_captured_and_all_paths_found() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_imports_negative: vim grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("vim")
        .expect("vim imports query missing");
    let paths = collect_captures(&lang, VIM_VARIANTS, &query_str, "import.path");

    assert!(
        !paths.iter().any(|p| p == "!"),
        "bang must never be captured as @import.path, got: {paths:?}"
    );
    for expected in [
        "~/.vim/plain.vim",
        "~/.vim/optional.vim",
        "plugin/single.vim",
        "plugin/*.vim",
    ] {
        assert!(
            paths.iter().any(|p| p == expected),
            "expected '{expected}' as @import.path in variants.vim, got: {paths:?}"
        );
    }
}

/// Negative cases: a lambda assignment is not a function definition, and a
/// bare identifier reference (not the `function` field of a
/// call_expression) is not a call.
#[test]
fn vim_negative_lambda_not_definition_and_bare_ref_not_call() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_negative: vim grammar .so not found");
        return;
    };
    let tags_query = loader.get_tags("vim").expect("vim tags query missing");
    let tags_names = collect_captures(&lang, VIM_VARIANTS, &tags_query, "name");
    assert!(
        !tags_names.iter().any(|n| n.contains("Lambda")),
        "lambda assignment must not be captured as a function definition, got: {tags_names:?}"
    );

    let calls_query = loader.get_calls("vim").expect("vim calls query missing");
    let calls = collect_captures(&lang, VIM_VARIANTS, &calls_query, "call");
    // "PlainFunc" is a legitimate call target once (`call PlainFunc()`);
    // the later bare-reference assignment (`let s:not_a_call = PlainFunc`)
    // must not add a second occurrence.
    let plain_func_count = calls.iter().filter(|c| *c == "PlainFunc").count();
    assert_eq!(
        plain_func_count, 1,
        "expected exactly 1 @call 'PlainFunc' (the bare-reference assignment must not \
         also match), got {plain_func_count}: {calls:?}"
    );
}
