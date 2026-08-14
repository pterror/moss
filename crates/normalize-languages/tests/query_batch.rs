//! Query fixture tests for batch.
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

const BATCH_SAMPLE: &str = include_str!("fixtures/batch/sample.bat");

const BATCH_VARIANTS: &str = include_str!("fixtures/batch/variants.bat");

// --- batch complexity @nesting: dimension 4 (real-world sample) ------------

#[test]
fn batch_complexity_nesting_finds_sample_labels_including_known_false_positives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping batch_complexity_nesting_finds_sample_labels_including_known_false_positives: \
             run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("batch").ok() else {
        eprintln!(
            "Skipping batch_complexity_nesting_finds_sample_labels_including_known_false_positives: \
             batch grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_complexity("batch")
        .expect("batch complexity query missing");

    let captures = collect_captures_full(&lang, BATCH_SAMPLE, &query_str);
    let nesting: Vec<&str> = captures
        .iter()
        .filter(|(name, ..)| name == "nesting")
        .map(|(_, _, text, _)| text.as_str())
        .collect();

    // The two genuine label definitions in the sample.
    assert!(nesting.contains(&":main"), "{nesting:?}");
    assert!(nesting.contains(&":cleanup"), "{nesting:?}");

    // Every @nesting capture must be a `function_definition` node kind
    // (extraction-depth check).
    for (name, kind, text, _line) in &captures {
        if name == "nesting" {
            assert_eq!(
                kind, "function_definition",
                "expected @nesting capture to be a `function_definition` node, \
                 got kind '{kind}' for text '{text}'"
            );
        }
    }

    // Exact count including the documented false positives: `:main` and
    // `:cleanup` each also appear as spurious `goto :label` targets (see
    // batch.cfg.scm doc comment). Sample has: `goto :cleanup`, `goto :main`,
    // real `:main` def, `goto :cleanup`, real `:cleanup` def = 5 total.
    assert_eq!(
        nesting.len(),
        5,
        "expected 5 total @nesting captures (2 genuine defs + 3 goto-target \
         false positives, per the documented grammar limitation), got {}: {nesting:?}",
        nesting.len()
    );
}

// --- batch complexity @nesting: dimension 2 + 3 (completeness matrix) ------

#[test]
fn batch_complexity_nesting_completeness_label_and_target_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping batch_complexity_nesting_completeness_label_and_target_variants: \
             run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("batch").ok() else {
        eprintln!(
            "Skipping batch_complexity_nesting_completeness_label_and_target_variants: \
             batch grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_complexity("batch")
        .expect("batch complexity query missing");

    let captures = collect_captures_full(&lang, BATCH_VARIANTS, &query_str);
    let nesting: Vec<(&str, usize)> = captures
        .iter()
        .filter(|(name, ..)| name == "nesting")
        .map(|(_, _, text, line)| (text.as_str(), *line))
        .collect();

    // Genuine label definition at statement start.
    assert!(
        nesting.iter().any(|(t, l)| *t == ":real_label" && *l == 10),
        "expected genuine :real_label definition, got: {nesting:?}"
    );
    // goto :label target — documented false positive.
    assert!(
        nesting.iter().any(|(t, l)| *t == ":real_label" && *l == 17),
        "expected goto-target false positive for :real_label, got: {nesting:?}"
    );
    // goto :eof target — documented false positive.
    assert!(
        nesting.iter().any(|(t, l)| *t == ":eof" && *l == 21),
        "expected goto-target false positive for :eof, got: {nesting:?}"
    );
    // call :label target — documented false positive (no keyword anchor
    // even available, since `call` isn't a recognized keyword).
    assert!(
        nesting.iter().any(|(t, l)| *t == ":real_label" && *l == 27),
        "expected call-target false positive for :real_label, got: {nesting:?}"
    );
    // second genuine label definition.
    assert!(
        nesting
            .iter()
            .any(|(t, l)| *t == ":second_real_label" && *l == 29),
        "expected genuine :second_real_label definition, got: {nesting:?}"
    );
    assert_eq!(
        nesting.len(),
        5,
        "expected exactly 5 @nesting captures in the variants fixture, got \
         {}: {nesting:?}",
        nesting.len()
    );
}

// --- batch cfg: documented N/A (no @cfg.* capture vocabulary applies) ------

#[test]
fn batch_cfg_query_produces_no_captures() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping batch_cfg_query_produces_no_captures: run \
             `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("batch").ok() else {
        eprintln!("Skipping batch_cfg_query_produces_no_captures: batch grammar .so not found");
        return;
    };
    let query_str = loader.get_cfg("batch").expect("batch cfg query missing");

    // batch.cfg.scm intentionally has zero patterns: the grammar collapses
    // all control-flow keywords (IF/FOR/GOTO/...) into a generic `keyword`
    // node with no distinguishing field, so no @cfg.branch/@cfg.loop/
    // @cfg.match/@cfg.exit.* capture can be produced without false
    // positives. Confirm this stays true (rather than silently drifting)
    // across both fixtures, for every capture name the CFG vocabulary uses.
    for prefix in ["cfg.branch", "cfg.loop", "cfg.match", "cfg.exit", "nesting"] {
        let sample_caps = collect_captures(&lang, BATCH_SAMPLE, &query_str, prefix);
        let variants_caps = collect_captures(&lang, BATCH_VARIANTS, &query_str, prefix);
        assert!(
            sample_caps.is_empty(),
            "expected no @{prefix} captures on the batch sample, got: {sample_caps:?}"
        );
        assert!(
            variants_caps.is_empty(),
            "expected no @{prefix} captures on the batch variants fixture, got: {variants_caps:?}"
        );
    }
}

// --- batch calls: documented N/A (no call-expression node in the grammar) --

#[test]
fn batch_calls_query_produces_no_captures() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping batch_calls_query_produces_no_captures: run \
             `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("batch").ok() else {
        eprintln!("Skipping batch_calls_query_produces_no_captures: batch grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("batch")
        .expect("batch calls query missing");

    // batch.calls.scm intentionally has zero patterns: the grammar has no
    // distinct call-expression node type (verified: `call foo.bat` and
    // `call :label` both parse as `identifier` inside an `ERROR` node, not
    // as any dedicated call node). Confirm this stays true on both fixtures
    // rather than silently drifting if the grammar or query ever changes.
    let sample_calls = collect_captures(&lang, BATCH_SAMPLE, &query_str, "call");
    let variants_calls = collect_captures(&lang, BATCH_VARIANTS, &query_str, "call");
    assert!(
        sample_calls.is_empty(),
        "expected no @call captures on the batch sample, got: {sample_calls:?}"
    );
    assert!(
        variants_calls.is_empty(),
        "expected no @call captures on the batch variants fixture, got: {variants_calls:?}"
    );
}
