//! Query fixture tests for awk.
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
// AWK
// ---------------------------------------------------------------------------

const AWK_SAMPLE: &str = include_str!("fixtures/awk/sample.awk");

#[test]
fn awk_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping awk_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_tags: awk grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("awk").expect("awk tags query missing");
    let names = collect_captures(&lang, AWK_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "max" || n == "trim"),
        "expected 'classify'/'max'/'trim' in awk tags, got: {names:?}"
    );
}

#[test]
fn awk_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping awk_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_calls: awk grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("awk").expect("awk calls query missing");
    let calls = collect_captures(&lang, AWK_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "max" || c == "trim" || c == "gsub"),
        "expected a function call in awk sample, got: {calls:?}"
    );
}

#[test]
fn awk_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping awk_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_complexity: awk grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("awk")
        .expect("awk complexity query missing");
    let complexity = collect_captures(&lang, AWK_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in awk sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn awk_imports_finds_include_directive() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping awk_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_imports: awk grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("awk")
        .expect("awk imports query missing");
    let paths = collect_captures(&lang, AWK_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("helpers.awk")),
        "expected '@include \"helpers.awk\"' in awk import paths, got: {paths:?}"
    );
}

const AWK_VARIANTS: &str = include_str!("fixtures/awk/variants.awk");

#[test]
fn awk_imports_completeness_include_load_and_negative_namespace() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping awk_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_imports_completeness: awk grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("awk")
        .expect("awk imports query missing");
    let paths = collect_captures(&lang, AWK_VARIANTS, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("lib.awk")),
        "expected '@include \"lib.awk\"' path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("ext")),
        "expected '@load \"ext\"' path, got: {paths:?}"
    );
    // NEGATIVE: @namespace must not be captured as an import.
    assert!(
        !paths.iter().any(|p| p.contains("mylib")),
        "'@namespace \"mylib\"' must not be captured as an import, got: {paths:?}"
    );
}

#[test]
fn awk_tags_completeness_qualified_function_definition() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping awk_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_tags_completeness: awk grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("awk").expect("awk tags query missing");
    let names = collect_captures(&lang, AWK_VARIANTS, &query_str, "name");
    assert!(
        names.contains(&"plain_fn".to_string()),
        "expected plain 'plain_fn' in awk tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("qualified_fn")),
        "expected namespace-qualified function definition in awk tags, got: {names:?}"
    );
}

#[test]
fn awk_calls_completeness_qualified_and_indirect() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping awk_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_calls_completeness: awk grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("awk").expect("awk calls query missing");
    let calls = collect_captures(&lang, AWK_VARIANTS, &query_str, "call");
    assert!(
        calls.contains(&"plain_fn".to_string()),
        "expected plain call 'plain_fn', got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c.contains("qualified_fn")),
        "expected namespace-qualified call, got: {calls:?}"
    );
    // Indirect call via a variable (`@f(...)`) — the variable identifier
    // is captured (best available static information), not the
    // ultimately-called function.
    assert!(
        calls.contains(&"f".to_string()),
        "expected indirect call variable 'f', got: {calls:?}"
    );
}

#[test]
fn awk_complexity_completeness_switch_case() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping awk_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_complexity_completeness: awk grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("awk")
        .expect("awk complexity query missing");
    let captures = collect_captures_full(&lang, AWK_VARIANTS, &query_str);
    // switch_demo has 3 switch_case nodes (0, 1, 2) — each a complexity
    // point — and exactly one switch_statement, counted for nesting only.
    let case_complexity = captures
        .iter()
        .filter(|(name, kind, _, _)| name == "complexity" && kind == "switch_case")
        .count();
    assert_eq!(
        case_complexity, 3,
        "expected 3 switch_case complexity captures, got {case_complexity}: {captures:?}"
    );
    let switch_nesting = captures
        .iter()
        .filter(|(name, kind, _, _)| name == "nesting" && kind == "switch_statement")
        .count();
    assert_eq!(
        switch_nesting, 1,
        "expected 1 switch_statement nesting capture, got {switch_nesting}: {captures:?}"
    );
    // switch_default must NOT be counted as complexity (no condition).
    assert!(
        !captures
            .iter()
            .any(|(name, kind, _, _)| name == "complexity" && kind == "switch_default"),
        "switch_default must not be counted as complexity, got: {captures:?}"
    );
}

#[test]
fn awk_cfg_finds_switch_match_arms() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping awk_cfg: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_cfg: awk grammar .so not found");
        return;
    };
    let query_str = loader.get_cfg("awk").expect("awk cfg query missing");
    let captures = collect_captures_full(&lang, AWK_VARIANTS, &query_str);
    assert!(
        captures
            .iter()
            .any(|(name, _, text, _)| name == "cfg.match.scrutinee" && text == "n"),
        "expected switch scrutinee 'n', got: {captures:?}"
    );
    let arm_count = captures
        .iter()
        .filter(|(name, _, _, _)| name == "cfg.match.arm")
        .count();
    // 3 switch_case arms (0, 1, 2) + 1 switch_default arm = 4.
    assert_eq!(
        arm_count, 4,
        "expected 4 cfg.match.arm captures (3 case + 1 default), got {arm_count}: {captures:?}"
    );
}
