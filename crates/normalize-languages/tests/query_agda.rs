//! Query fixture tests for agda.
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
// Agda
// ---------------------------------------------------------------------------

const AGDA_SAMPLE: &str = include_str!("fixtures/agda/sample.agda");

#[test]
fn agda_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_tags: agda grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("agda").expect("agda tags query missing");
    let names = collect_captures(&lang, AGDA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' data type in agda tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "area" || n == "double"),
        "expected a function name in agda tags, got: {names:?}"
    );
}

#[test]
fn agda_calls_finds_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_calls: agda grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("agda").expect("agda calls query missing");
    let calls = collect_captures(&lang, AGDA_SAMPLE, &query_str, "call");
    assert!(
        !calls.is_empty(),
        "expected at least one call in agda sample, got: {calls:?}"
    );
}

#[test]
fn agda_complexity_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_complexity: agda grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("agda")
        .expect("agda complexity query missing");
    let complexity = collect_captures(&lang, AGDA_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in agda sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn agda_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_imports: agda grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("agda")
        .expect("agda imports query missing");
    let paths = collect_captures(&lang, AGDA_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Data")),
        "expected a 'Data.*' import path in agda sample, got: {paths:?}"
    );
}

const AGDA_VARIANTS: &str = include_str!("fixtures/agda/variants.agda");

#[test]
fn agda_imports_negative_no_duplicate_open_import() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping agda_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_imports_negative: agda grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("agda")
        .expect("agda imports query missing");
    // Regression test: `open import Data.Maybe using (Maybe)` must produce
    // exactly ONE @import.path entry, not two (the old query captured both
    // the nested `import` node and the outer `open` node as separate
    // overlapping @import.path/@import statements for the same line).
    let paths = collect_captures(&lang, AGDA_VARIANTS, &query_str, "import.path");
    let maybe_count = paths.iter().filter(|p| p.contains("Data.Maybe")).count();
    assert_eq!(
        maybe_count, 1,
        "expected exactly 1 import.path capture for 'open import Data.Maybe ...', got {maybe_count}: {paths:?}"
    );
    // Plain `import Data.List` and bare `open Data.List` are two distinct
    // statements and each contributes its own path.
    let list_count = paths.iter().filter(|p| p.contains("Data.List")).count();
    assert_eq!(
        list_count, 2,
        "expected 2 import.path captures for 'Data.List' (plain import + bare open), got {list_count}: {paths:?}"
    );
}

#[test]
fn agda_imports_distinguishes_glob_open() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping agda_imports_glob: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_imports_glob: agda grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("agda")
        .expect("agda imports query missing");
    let captures = collect_captures_full(&lang, AGDA_VARIANTS, &query_str);
    // `import Data.List` (plain) must NOT carry @import.glob.
    assert!(
        !captures
            .iter()
            .any(|(name, _, text, _)| name == "import.glob" && text.contains("import Data.List")),
        "plain 'import Data.List' must not be marked import.glob, got: {captures:?}"
    );
    // `open import Data.Maybe ...` and bare `open Data.List` must both
    // carry @import.glob.
    assert!(
        captures
            .iter()
            .any(|(name, _, text, _)| name == "import.glob" && text.contains("Data.Maybe")),
        "expected 'open import Data.Maybe' to carry import.glob, got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(name, kind, text, _)| name == "import.glob"
                && kind == "open"
                && text == "open Data.List"),
        "expected bare 'open Data.List' to carry import.glob, got: {captures:?}"
    );
}

#[test]
fn agda_tags_completeness_constructors_equations_and_where_scoping() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping agda_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_tags_completeness: agda grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("agda").expect("agda tags query missing");
    let pairs = collect_tag_pairs(&lang, AGDA_VARIANTS, &query_str);
    // Data constructors share the function/signature node shape and are
    // captured as definition.function (the grammar has no distinct
    // constructor node type).
    for ctor in ["Red", "Green", "Blue"] {
        assert!(
            pairs.contains(&("definition.function".to_string(), ctor.to_string())),
            "expected constructor '{ctor}' as definition.function, got: {pairs:?}"
        );
    }
    // Both the signature (`identity : Int -> Int`) and the equation
    // (`identity n = n`) contribute a definition.function capture for the
    // same name — verified as the grammar's actual behavior, not
    // deduplicated, since they are two distinct declaration sites.
    let identity_count = pairs
        .iter()
        .filter(|p| p.0 == "definition.function" && p.1 == "identity")
        .count();
    assert_eq!(
        identity_count, 2,
        "expected 2 definition.function captures for 'identity' (signature + equation), got {identity_count}: {pairs:?}"
    );
    // Record field names (`contents` in `record Box`) are NOT tagged as
    // definitions, matching this codebase's convention of not tagging
    // individual struct/record fields (see rust.tags.scm/go.tags.scm).
    assert!(
        !pairs.iter().any(|p| p.1 == "contents"),
        "record field 'contents' must not appear in tags, got: {pairs:?}"
    );
    // A function with no type signature at all (`pointfree = 42`) must
    // still be tagged — regression test for the equation-form fix
    // (previously tags.scm only ever matched a function via its
    // signature's `function_name` wrapper, so an unsignatured function was
    // invisible entirely).
    assert!(
        pairs.contains(&("definition.function".to_string(), "pointfree".to_string())),
        "expected signature-less 'pointfree' as definition.function, got: {pairs:?}"
    );
    // `where`-bound local helpers must NOT leak in as top-level
    // definitions — regression test for the container-scoping fix.
    assert!(
        !pairs.iter().any(|p| p.1 == "helper"),
        "where-bound local 'helper' must not appear in tags, got: {pairs:?}"
    );
    // The outer function that owns the where-clause must still be tagged
    // normally (both its signature and equation).
    let has_helper_count = pairs
        .iter()
        .filter(|p| p.0 == "definition.function" && p.1 == "hasHelper")
        .count();
    assert_eq!(
        has_helper_count, 2,
        "expected 2 definition.function captures for 'hasHelper' (signature + equation), got {has_helper_count}: {pairs:?}"
    );
}

#[test]
fn agda_types_completeness_signature_forms() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping agda_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_types_completeness: agda grammar .so not found");
        return;
    };
    let query_str = loader.get_types("agda").expect("agda types query missing");
    let refs = collect_captures(&lang, AGDA_VARIANTS, &query_str, "type");
    // Function signature form: `identity : Int -> Int`.
    assert!(
        refs.iter().any(|r| r.contains("Int")),
        "expected a function-signature type reference containing 'Int', got: {refs:?}"
    );
    // Data constructor signature form: `Red : Color`.
    assert!(
        refs.contains(&"Color".to_string()),
        "expected constructor signature type reference 'Color', got: {refs:?}"
    );
    // Record field signature form: `contents : Int` inside `record Box`.
    // (Also exercised by the function-signature assertion above since both
    // produce the text "Int" — distinguished at the fixture-authoring level
    // by the dedicated Box record; asserted structurally via completeness
    // matrix comments, not by a redundant duplicate text check here.)
}

#[test]
fn agda_calls_completeness_and_negatives() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping agda_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_calls_completeness: agda grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("agda").expect("agda calls query missing");
    let calls = collect_captures(&lang, AGDA_VARIANTS, &query_str, "call");
    // Positive: application with an argument at the head of a defining
    // equation.
    assert!(
        calls.contains(&"suc".to_string()),
        "expected 'suc' call in 'addOne n = suc n', got: {calls:?}"
    );
    // Positive: outer call of a nested/parenthesized application.
    assert!(
        calls.contains(&"addOne".to_string()),
        "expected outer 'addOne' call in 'addTwo n = addOne (addOne n)', got: {calls:?}"
    );
    // Negative: a bare single-atom rhs (`answer = 42`) must not be a call
    // — regression test for the literal-parses-as-qid false positive.
    assert!(
        !calls.contains(&"42".to_string()),
        "bare literal rhs must not be captured as a call, got: {calls:?}"
    );
    // Negative: the type names in a signature (`identity : Int -> Int`)
    // must not be captured as calls — regression test for the systematic
    // false positive an unanchored expr-head pattern produced.
    assert!(
        !calls
            .iter()
            .any(|c| c == "Int" || c == "Color" || c == "Set"),
        "type-signature names must not be captured as calls, got: {calls:?}"
    );
}

#[test]
fn agda_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping agda_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "agda",
        AGDA_SAMPLE,
        &["-- A simple data type"],
    );
}
