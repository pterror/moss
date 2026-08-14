//! Query fixture tests for ada.
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
// Ada
// ---------------------------------------------------------------------------

const ADA_SAMPLE: &str = include_str!("fixtures/ada/sample.adb");

#[test]
fn ada_tags_finds_subprograms_and_packages() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_tags: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ada").expect("ada tags query missing");
    let names = collect_captures(&lang, ADA_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Add" || n == "Classify" || n == "Calculator"),
        "expected 'Add'/'Classify'/'Calculator' in ada tags, got: {names:?}"
    );
}

#[test]
fn ada_calls_finds_procedure_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_calls: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ada").expect("ada calls query missing");
    let calls = collect_captures(&lang, ADA_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "Print_Result" || c == "Put_Line" || c == "Add"),
        "expected a procedure call in ada sample, got: {calls:?}"
    );
}

#[test]
fn ada_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_complexity: ada grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("ada")
        .expect("ada complexity query missing");
    let complexity = collect_captures(&lang, ADA_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in ada sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn ada_imports_finds_with_clauses() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_imports: ada grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("ada")
        .expect("ada imports query missing");
    let paths = collect_captures(&lang, ADA_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Text_IO") || p.contains("Ada")),
        "expected 'Ada.Text_IO' in ada import paths, got: {paths:?}"
    );
}

#[test]
fn ada_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_types: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_types("ada").expect("ada types query missing");
    let refs = collect_captures(&lang, ADA_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in ada sample, got: {refs:?}"
    );
}

const ADA_VARIANTS: &str = include_str!("fixtures/ada/variants.adb");

#[test]
fn ada_tags_finds_child_package_and_generic_subprograms() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ada_tags_finds_child_package: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_tags_finds_child_package: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ada").expect("ada tags query missing");
    let pairs = collect_tag_pairs(&lang, ADA_SAMPLE, &query_str);
    // Dimension 4: dotted child-package name (Calculator.Utils) and generic
    // subprograms (Generic_Identity/Generic_Swap) — real-world Ada idioms
    // previously dropped entirely by ada.tags.scm.
    assert!(
        pairs.contains(&(
            "definition.module".to_string(),
            "Calculator.Utils".to_string()
        )),
        "expected 'Calculator.Utils' child package in ada tags, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&(
            "definition.function".to_string(),
            "Generic_Identity".to_string()
        )),
        "expected generic function 'Generic_Identity' in ada tags, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&(
            "definition.function".to_string(),
            "Generic_Swap".to_string()
        )),
        "expected generic procedure 'Generic_Swap' in ada tags, got: {pairs:?}"
    );
}

#[test]
fn ada_tags_completeness_name_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ada_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_tags_completeness: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ada").expect("ada tags query missing");
    let pairs = collect_tag_pairs(&lang, ADA_VARIANTS, &query_str);

    // package_declaration.name: identifier variant.
    assert_eq!(
        pairs
            .iter()
            .filter(|p| *p == &("definition.module".to_string(), "Plain_Pkg".to_string()))
            .count(),
        1,
        "expected exactly 1 'Plain_Pkg' definition.module, got: {pairs:?}"
    );
    // package_declaration.name: selected_component variant (dotted child package).
    assert_eq!(
        pairs
            .iter()
            .filter(|p| *p
                == &(
                    "definition.module".to_string(),
                    "Plain_Pkg.Child".to_string()
                ))
            .count(),
        1,
        "expected exactly 1 'Plain_Pkg.Child' definition.module, got: {pairs:?}"
    );
    // generic_package_declaration wrapping package_declaration: must produce
    // exactly ONE capture, not two (regression test for the duplicate-capture
    // bug the old dedicated generic_package_declaration pattern caused).
    assert_eq!(
        pairs
            .iter()
            .filter(|p| *p == &("definition.module".to_string(), "Generic_Pkg".to_string()))
            .count(),
        1,
        "expected exactly 1 'Generic_Pkg' definition.module (no duplicate from generic wrapper), got: {pairs:?}"
    );
    // generic_subprogram_declaration wrapping function_specification/procedure_specification.
    for name in ["Generic_Func", "Generic_Proc"] {
        assert_eq!(
            pairs
                .iter()
                .filter(|p| p.0 == "definition.function" && p.1 == name)
                .count(),
            1,
            "expected exactly 1 '{name}' definition.function, got: {pairs:?}"
        );
    }
    // Plain (non-generic) function/procedure names, baseline still holds.
    for name in ["Plain_Func", "Plain_Proc"] {
        assert!(
            pairs
                .iter()
                .any(|p| p.0 == "definition.function" && p.1 == name),
            "expected '{name}' definition.function, got: {pairs:?}"
        );
    }
    assert!(
        pairs.contains(&("definition.type".to_string(), "Plain_Type".to_string())),
        "expected 'Plain_Type' definition.type, got: {pairs:?}"
    );
}

#[test]
fn ada_types_completeness_qualified_subtype_mark() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ada_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_types_completeness: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_types("ada").expect("ada types query missing");
    let captures = collect_captures_full(&lang, ADA_VARIANTS, &query_str);
    // Every subtype_mark-bearing field (result_profile, component_definition,
    // subtype_declaration, derived_type_definition, plus the already-handled
    // object_declaration/parameter_specification) allows a selected_component
    // (package-qualified) type name per node-types.json. Six qualified
    // File_Type references exist in variants.adb: Qualified_Return (result
    // profile), the Rec_With_Qualified_Field component, Qualified_Subtype,
    // Qualified_Derived, Log_File (object decl), and the Use_File parameter.
    let file_type_refs = captures
        .iter()
        .filter(|(_, kind, text, _)| kind == "identifier" && text == "File_Type")
        .count();
    assert_eq!(
        file_type_refs, 6,
        "expected 6 qualified 'File_Type' type references (one per subtype_mark-bearing \
         construct), got {file_type_refs}: {captures:?}"
    );
}

#[test]
fn ada_calls_qualified_call_captures_selector_and_qualifier() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ada_calls_qualified: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_calls_qualified: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ada").expect("ada calls query missing");
    let captures = collect_captures_full(&lang, ADA_VARIANTS, &query_str);
    // Qualified_Call_Demo calls `Ada.Text_IO.Put_Line ("qualified");` — the
    // selector (callee) must be captured as @call with text "Put_Line", and
    // the prefix as @call.qualifier with text "Ada.Text_IO", not the whole
    // dotted expression collapsed into a single @call.
    assert!(
        captures
            .iter()
            .any(|(name, _, text, _)| name == "call" && text == "Put_Line"),
        "expected @call = 'Put_Line', got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(name, _, text, _)| name == "call.qualifier" && text == "Ada.Text_IO"),
        "expected @call.qualifier = 'Ada.Text_IO', got: {captures:?}"
    );
}

#[test]
fn ada_calls_negative_bare_reference_is_not_a_call() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ada_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_calls_negative: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ada").expect("ada calls query missing");
    let calls = collect_captures(&lang, ADA_VARIANTS, &query_str, "call");
    // Bare_Reference_Demo's `Local := Flag;` is an assignment, not a call —
    // neither operand may appear as a @call capture.
    assert!(
        !calls.iter().any(|c| c == "Local" || c == "Flag"),
        "assignment operands must not be captured as calls, got: {calls:?}"
    );
}

#[test]
fn ada_decorations_finds_pragma_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ada_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // pragma_g is the verified node name for Ada pragmas in tree-sitter-ada (RM 2.8).
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "ada",
        ADA_SAMPLE,
        &[
            "pragma Inline(Add);",
            "-- Add two integers and return the result",
        ],
    );
}
