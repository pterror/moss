//! Query fixture tests for gleam.
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

const GLEAM_SAMPLE: &str = include_str!("fixtures/gleam/sample.gleam");

#[test]
fn gleam_decorations_finds_doc_comment_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping gleam_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "gleam",
        GLEAM_SAMPLE,
        &["/// Classify", "// Type definition"],
    );
}

#[test]
fn gleam_tags_finds_functions_types_and_constants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_tags: gleam grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("gleam").expect("gleam tags query missing");
    let names = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"factorial".to_string()),
        "expected 'factorial' function in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' custom type in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Name".to_string()),
        "expected 'Name' type alias in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"max_size".to_string()),
        "expected 'max_size' constant in gleam tags, got: {names:?}"
    );
}

#[test]
fn gleam_calls_finds_local_and_qualified_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_calls: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("gleam")
        .expect("gleam calls query missing");
    let calls = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"factorial".to_string()),
        "expected recursive 'factorial' call in gleam calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"filter".to_string()),
        "expected qualified 'list.filter' call to capture 'filter' in gleam calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"println".to_string()),
        "expected qualified 'io.println' call to capture 'println' in gleam calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"list".to_string()),
        "expected 'list' qualifier in gleam calls, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"io".to_string()),
        "expected 'io' qualifier in gleam calls, got: {qualifiers:?}"
    );
}

#[test]
fn gleam_complexity_finds_case_expressions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_complexity: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("gleam")
        .expect("gleam complexity query missing");
    let complexity = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 5,
        "expected at least 5 complexity nodes (case + case_clause) in gleam sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn gleam_imports_finds_module_paths_aliases_and_names() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_imports: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("gleam")
        .expect("gleam imports query missing");
    let paths = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"gleam/io".to_string()),
        "expected 'gleam/io' import path in gleam imports, got: {paths:?}"
    );
    assert!(
        paths.contains(&"gleam/list".to_string()),
        "expected 'gleam/list' import path in gleam imports, got: {paths:?}"
    );
    assert!(
        paths.contains(&"gleam/int".to_string()),
        "expected 'gleam/int' import path in gleam imports, got: {paths:?}"
    );
}

// ==================================================================
// ==== BATCH 8: gleam, kdl (query-testing methodology sweep) =====
// ==================================================================

const GLEAM_VARIANTS: &str = include_str!("fixtures/gleam/variants.gleam");

/// Dimension 2/3: pipe-target call variants that `function_call`-only
/// patterns miss entirely — bare-identifier (`x |> f`) and point-free
/// qualified (`x |> module.func`) pipe targets are never wrapped in a
/// function_call node by the grammar. Verified via `normalize syntax ast`.
#[test]
fn gleam_calls_completeness_pipe_targets() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping gleam_calls_completeness_pipe_targets: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_calls_completeness_pipe_targets: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("gleam")
        .expect("gleam calls query missing");
    let caps = collect_captures_full(&lang, GLEAM_VARIANTS, &query_str);

    // Bare pipe target: `x |> identity` (pipe_bare_identifier).
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "identifier" && t == "identity"),
        "expected bare pipe-target 'identity' as @call(identifier) in gleam.calls.scm \
         output for variants.gleam, got: {caps:?}"
    );
    // Qualified pipe target: `values |> list.length` (pipe_qualified) — must
    // produce both @call and @call.qualifier, same as a parenthesized
    // qualified call would.
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "label" && t == "length"),
        "expected qualified pipe-target 'length' as @call(label) in gleam.calls.scm \
         output for variants.gleam, got: {caps:?}"
    );
    assert!(
        caps.iter()
            .any(|(cn, _, t, _)| cn == "call.qualifier" && t == "list"),
        "expected 'list' qualifier for the qualified pipe target, got: {caps:?}"
    );
}

/// Negative cases for gleam.calls.scm: constructs that must never appear in
/// @call captures.
#[test]
fn gleam_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_calls_negative: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("gleam")
        .expect("gleam calls query missing");
    let caps = collect_captures_full(&lang, GLEAM_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // The closure *definition* site (`fn(n: Int) -> Int { n + 1 }`) must
    // never appear as a call; only the call site `add_one(x)` should.
    let add_one_calls = call_texts.iter().filter(|t| **t == "add_one").count();
    assert_eq!(
        add_one_calls, 1,
        "expected exactly 1 call to 'add_one' (the call site, not the closure \
         definition), got {add_one_calls}: {call_texts:?}"
    );
    // A bare variable read (`let _tag = holder`) is not a call.
    assert!(
        !call_texts.contains(&"holder"),
        "bare identifier read 'holder' must not be captured as a call, got: {call_texts:?}"
    );
}

/// Dimension 2/3: external_function definitions and the (rare but
/// grammar-legal) remote_type_identifier variant of type_name.name.
#[test]
fn gleam_tags_completeness_external_function_and_type_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_tags_completeness: gleam grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("gleam").expect("gleam tags query missing");
    let pairs = collect_tag_pairs(&lang, GLEAM_VARIANTS, &query_str);

    assert!(
        pairs.contains(&("definition.function".to_string(), "native_add".to_string())),
        "expected external_function 'native_add' as @definition.function in gleam tags, \
         got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.class".to_string(), "Color".to_string())),
        "expected custom type 'Color' as @definition.class, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.type".to_string(), "Meters".to_string())),
        "expected type alias 'Meters' as @definition.type, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "sqrt".to_string())),
        "expected bodyless @external-attributed function 'sqrt' as @definition.function, \
         got: {pairs:?}"
    );
}

/// Negative case for gleam.tags.scm: a closure literal (anonymous_function)
/// must never appear as a @definition.function tag — only named `function`/
/// `external_function` nodes should.
#[test]
fn gleam_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_tags_negative: gleam grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("gleam").expect("gleam tags query missing");
    let pairs = collect_tag_pairs(&lang, GLEAM_VARIANTS, &query_str);
    let function_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.function")
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        !function_names.contains(&"add_one"),
        "closure literal bound to 'add_one' must not be tagged as a \
         @definition.function, got: {function_names:?}"
    );
}

/// Dimension 2: unqualified_import's own `alias:` field
/// (`Some as MySome`/`type Request as HttpRequest`) — distinct from the
/// whole-import module `alias:` field, and previously uncaptured entirely.
#[test]
fn gleam_imports_completeness_unqualified_aliases() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_imports_completeness: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("gleam")
        .expect("gleam imports query missing");
    let caps = collect_captures_full(&lang, GLEAM_VARIANTS, &query_str);

    // `Some as MySome` inside `import gleam/list.{type Option, Some as MySome}`.
    assert!(
        caps.iter()
            .any(|(cn, _, t, _)| cn == "import.alias" && t == "MySome"),
        "expected unqualified-import alias 'MySome' as @import.alias, got: {caps:?}"
    );
    // Whole-module alias `import gleam/result as res`.
    assert!(
        caps.iter()
            .any(|(cn, _, t, _)| cn == "import.alias" && t == "res"),
        "expected whole-module alias 'res' as @import.alias, got: {caps:?}"
    );
}

/// Dimension 4 (real-world) + dimension 2 (completeness): Gleam attributes
/// (`@deprecated(...)`, `@external(...)`) are a distinct decoration
/// construct, analogous to Rust's `#[attr]`/Python's `@decorator`, and were
/// entirely absent from gleam.decorations.scm.
#[test]
fn gleam_decorations_finds_attributes() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!(
            "Skipping gleam_decorations_finds_attributes: run `cargo xtask build-grammars` first"
        );
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "gleam",
        GLEAM_VARIANTS,
        &[
            "@deprecated(\"use plain_function instead\")",
            "@external(erlang, \"math\", \"sqrt\")",
        ],
    );
}
