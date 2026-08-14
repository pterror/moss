//! Query fixture tests for powershell.
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
// PowerShell
// ---------------------------------------------------------------------------

const POWERSHELL_SAMPLE: &str = include_str!("fixtures/powershell/sample.ps1");

#[test]
fn powershell_tags_finds_functions_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_tags: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("powershell")
        .expect("powershell tags query missing");
    let names = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Invoke-Classify" || n == "Get-Sum" || n == "Calculator"),
        "expected 'Invoke-Classify'/'Get-Sum'/'Calculator' in powershell tags, got: {names:?}"
    );
}

#[test]
fn powershell_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_calls: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("powershell")
        .expect("powershell calls query missing");
    let calls = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "Invoke-Classify" || c == "Get-Sum" || c == "Write-Host"),
        "expected a call in powershell sample, got: {calls:?}"
    );
}

#[test]
fn powershell_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_complexity: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("powershell")
        .expect("powershell complexity query missing");
    let complexity = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in powershell sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn powershell_imports_finds_import_module() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_imports: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("powershell")
        .expect("powershell imports query missing");
    let paths = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("PSReadLine") || p.contains("PowerShell")),
        "expected a module path in powershell imports, got: {paths:?}"
    );
}

#[test]
fn powershell_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_types: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("powershell")
        .expect("powershell types query missing");
    let refs = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in powershell sample, got: {refs:?}"
    );
}
