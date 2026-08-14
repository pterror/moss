//! Query fixture tests for devicetree.
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

// =============================================================================
// Batch 9: devicetree + ninja (query-testing methodology sweep)
// =============================================================================

// ---------------------------------------------------------------------------
// DeviceTree — imports.scm (query-testing methodology batch 9)
// ---------------------------------------------------------------------------

const DEVICETREE_SAMPLE: &str = include_str!("fixtures/devicetree/sample.dts");

const DEVICETREE_VARIANTS: &str = include_str!("fixtures/devicetree/variants.dts");

/// Dimension 4: the real-world-shaped sample (board overlay pulling in a
/// base `.dtsi` plus two vendor binding headers) must surface all three
/// `#include` directives as imports, in source order.
#[test]
fn devicetree_imports_finds_sample_includes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping devicetree_imports_finds_sample_includes: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("devicetree").ok() else {
        eprintln!(
            "Skipping devicetree_imports_finds_sample_includes: devicetree grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("devicetree")
        .expect("devicetree imports query missing");
    let paths = collect_captures(&lang, DEVICETREE_SAMPLE, &query_str, "import.path");
    assert_eq!(
        paths,
        vec![
            "\"board-base.dtsi\"",
            "<dt-bindings/gpio/gpio.h>",
            "<dt-bindings/pinctrl/nrf-pinctrl.h>",
        ],
        "expected exactly the three #include directives (quoted relative path \
         and two angle-bracket system paths), in source order, got: {paths:?}"
    );
}

/// Dimension 2/3 (completeness + extraction depth) for imports.scm:
/// `preproc_include.path` allows three node-type variants per
/// node-types.json — `string_literal`, `system_lib_string`, and a bare
/// `identifier` (macro-expanded include target, e.g. `#include
/// SOC_DTS_HEADER`). The `identifier` variant was missing before this batch
/// — silently dropping any macro-based include from extraction. Verified by
/// kind via `collect_captures_full` so the three variants can't hide behind
/// identical capture text.
#[test]
fn devicetree_imports_completeness_path_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping devicetree_imports_completeness_path_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("devicetree").ok() else {
        eprintln!(
            "Skipping devicetree_imports_completeness_path_variants: devicetree grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("devicetree")
        .expect("devicetree imports query missing");

    let full = collect_captures_full(&lang, DEVICETREE_VARIANTS, &query_str);
    let path_kinds: Vec<(&str, &str)> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.path")
        .map(|(_, kind, text, _)| (kind.as_str(), text.as_str()))
        .collect();
    assert_eq!(
        path_kinds,
        vec![
            ("string_literal", "\"relative-file.dtsi\""),
            ("system_lib_string", "<dt-bindings/gpio/gpio.h>"),
            ("identifier", "SOC_DTS_HEADER"),
        ],
        "expected exactly the three path-field variants node-types.json allows \
         for preproc_include, each with the correct node kind, got: {path_kinds:?}"
    );

    let import_count = full.iter().filter(|(cap, ..)| cap == "import").count();
    assert_eq!(
        import_count, 3,
        "expected exactly 3 @import matches (one per #include variant); the \
         property assignment and phandle reference in the NEGATIVE section \
         must not contribute extra matches"
    );
}

/// Dimension 3 negative case: a quoted-string property value and a `&label`
/// phandle reference (a distinct `reference` node type, not
/// `preproc_include`) must never be captured as imports.
#[test]
fn devicetree_imports_negative_property_and_phandle_reference() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping devicetree_imports_negative_property_and_phandle_reference: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("devicetree").ok() else {
        eprintln!(
            "Skipping devicetree_imports_negative_property_and_phandle_reference: devicetree grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("devicetree")
        .expect("devicetree imports query missing");

    // Isolate just the NEGATIVE section (everything after the third #include)
    // so the earlier positive matches don't mask a false positive here.
    let negative_start = DEVICETREE_VARIANTS
        .find("--- NEGATIVE")
        .expect("fixture must contain a NEGATIVE section marker");
    let negative_source = &DEVICETREE_VARIANTS[negative_start..];

    let paths = collect_captures(&lang, negative_source, &query_str, "import.path");
    assert!(
        paths.is_empty(),
        "the property assignment and phandle reference must not produce any \
         @import.path captures, got: {paths:?}"
    );
}
