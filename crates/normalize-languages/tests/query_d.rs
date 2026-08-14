//! Query fixture tests for d.
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
// D
// ---------------------------------------------------------------------------

const D_SAMPLE: &str = include_str!("fixtures/d/sample.d");

#[test]
fn d_tags_finds_functions_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_tags: d grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("d").expect("d tags query missing");
    let names = collect_captures(&lang, D_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in d tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' class in d tags, got: {names:?}"
    );
}

#[test]
fn d_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_calls: d grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("d").expect("d calls query missing");
    let calls = collect_captures(&lang, D_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "writeln" || c == "sqrt"),
        "expected 'writeln' or 'sqrt' call in d sample, got: {calls:?}"
    );
}

#[test]
fn d_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_complexity: d grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("d")
        .expect("d complexity query missing");
    let complexity = collect_captures(&lang, D_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in d sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn d_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_imports: d grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("d").expect("d imports query missing");
    let paths = collect_captures(&lang, D_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("std")),
        "expected std module in d import paths, got: {paths:?}"
    );
}

#[test]
fn d_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_types: d grammar .so not found");
        return;
    };
    let query_str = loader.get_types("d").expect("d types query missing");
    let refs = collect_captures(&lang, D_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected type references in d sample, got: {refs:?}"
    );
}

const D_VARIANTS: &str = include_str!("fixtures/d/variants.d");

#[test]
fn d_tags_completeness_all_definition_kinds() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_tags_completeness: d grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("d").expect("d tags query missing");
    let pairs = collect_tag_pairs(&lang, D_VARIANTS, &query_str);

    // class_declaration / class_template_declaration / struct_declaration /
    // struct_template_declaration / union_declaration / union_template_declaration
    // all classify as @definition.class (D has no separate "generic class" tag kind).
    for name in [
        "PlainClass",
        "GenericClass",
        "PlainStruct",
        "GenericStruct",
        "PlainUnion",
        "GenericUnion",
    ] {
        assert!(
            pairs.contains(&("definition.class".to_string(), name.to_string())),
            "expected @definition.class for '{name}', got: {pairs:?}"
        );
    }

    // interface_declaration / interface_template_declaration -> @definition.interface
    for name in ["PlainInterface", "GenericInterface"] {
        assert!(
            pairs.contains(&("definition.interface".to_string(), name.to_string())),
            "expected @definition.interface for '{name}', got: {pairs:?}"
        );
    }

    // enum_declaration -> @definition.type
    assert!(
        pairs.contains(&("definition.type".to_string(), "Color".to_string())),
        "expected @definition.type for 'Color', got: {pairs:?}"
    );

    // func_declaration and auto_func_declaration -> @definition.function
    for name in ["plainFunc", "autoFunc"] {
        assert!(
            pairs.contains(&("definition.function".to_string(), name.to_string())),
            "expected @definition.function for '{name}', got: {pairs:?}"
        );
    }
}

#[test]
fn d_types_completeness_structural_positions() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_types_completeness: d grammar .so not found");
        return;
    };
    let query_str = loader.get_types("d").expect("d types query missing");
    let refs = collect_captures_full(&lang, D_VARIANTS, &query_str);

    // Every variant from the completeness matrix must appear with kind
    // "qualified_identifier" — var declaration, nested/qualified type name,
    // parameter, cast target, `new` target, function return type, and alias
    // target all resolve to the same underlying node kind.
    let texts: Vec<&str> = refs.iter().map(|(_, _, t, _)| t.as_str()).collect();
    for expected in [
        "PlainClass",  // var_declarations (global)
        "std.math.PI", // var_declarations, nested qualified chain (outermost only)
        "Color",       // final_switch_statement scrutinee type via `type`
    ] {
        assert!(
            texts.contains(&expected),
            "expected '{expected}' in d type references, got: {texts:?}"
        );
    }
    // PlainClass appears at 8 distinct structural positions in variants.d:
    // global var, param, local var, cast target, new target (x2: typeSites
    // and returnTypeSite), return type, alias target.
    let plain_class_count = texts.iter().filter(|t| **t == "PlainClass").count();
    assert_eq!(
        plain_class_count, 8,
        "expected 8 PlainClass type references (var/param/local/cast/new x2/return/alias), got {plain_class_count}: {refs:?}"
    );
    for (cap, kind, _, _) in &refs {
        assert_eq!(
            cap, "type.reference",
            "unexpected capture name {cap} in d types query"
        );
        assert_eq!(
            kind, "qualified_identifier",
            "expected qualified_identifier kind for d type reference, got {kind}"
        );
    }
}

#[test]
fn d_types_negative_calls_are_not_types() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_types_negative: d grammar .so not found");
        return;
    };
    let query_str = loader.get_types("d").expect("d types query missing");
    let refs = collect_captures(&lang, D_VARIANTS, &query_str, "type.reference");
    // This was the original bug: `(qualified_identifier) @type.reference` bare
    // matched every call target and member access, not just type positions.
    // None of these call/member-access names must appear as a type reference.
    for not_a_type in ["plainFunc", "io.writeln", "helper", "GenericClass!int"] {
        assert!(
            !refs.iter().any(|r| r == not_a_type),
            "'{not_a_type}' is a call target, must not appear as @type.reference: {refs:?}"
        );
    }
}

#[test]
fn d_imports_completeness_alias_and_bindings() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_imports_completeness: d grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("d").expect("d imports query missing");
    let refs = collect_captures_full(&lang, D_VARIANTS, &query_str);

    let aliases: Vec<&str> = refs
        .iter()
        .filter(|(cap, ..)| cap == "import.alias")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert_eq!(
        aliases,
        vec!["io"],
        "expected exactly one @import.alias ('io') for `import io = std.stdio;`, got: {aliases:?}"
    );

    let paths: Vec<&str> = refs
        .iter()
        .filter(|(cap, ..)| cap == "import.path")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert_eq!(
        paths,
        vec!["std.stdio", "std.math"],
        "expected both import paths with no duplicates from the alias pattern, got: {paths:?}"
    );
}

#[test]
fn d_calls_completeness_call_shapes() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_calls_completeness: d grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("d").expect("d calls query missing");
    let calls = collect_captures(&lang, D_VARIANTS, &query_str, "call");
    for expected in ["plainFunc", "io.writeln", "GenericClass!int"] {
        assert!(
            calls.iter().any(|c| c == expected),
            "expected call '{expected}' in d calls, got: {calls:?}"
        );
    }
}

#[test]
fn d_complexity_finds_final_switch_as_distinct_node() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_complexity_final_switch: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_complexity_final_switch: d grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("d")
        .expect("d complexity query missing");
    let captures = collect_captures_full(&lang, D_VARIANTS, &query_str);
    let final_switch_complexity = captures
        .iter()
        .filter(|(cap, kind, ..)| cap == "complexity" && kind == "final_switch_statement")
        .count();
    assert_eq!(
        final_switch_complexity, 1,
        "expected final_switch_statement to be counted as @complexity distinctly from switch_statement, got: {captures:?}"
    );
    let plain_switch_complexity = captures
        .iter()
        .filter(|(cap, kind, ..)| cap == "complexity" && kind == "switch_statement")
        .count();
    assert_eq!(
        plain_switch_complexity, 1,
        "expected plain switch_statement to still be counted as @complexity, got: {captures:?}"
    );
}

#[test]
fn d_cfg_finds_final_switch_match_arms() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_cfg_final_switch: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_cfg_final_switch: d grammar .so not found");
        return;
    };
    let query_str = loader.get_cfg("d").expect("d cfg query missing");
    let arms = collect_captures(&lang, D_VARIANTS, &query_str, "cfg.match.arm");
    assert!(
        arms.iter().any(|a| a.contains("Color.Red")),
        "expected a final-switch case arm among cfg.match.arm captures, got: {arms:?}"
    );
}

#[test]
fn d_refactor_final_switch_is_a_statement() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping d_refactor_final_switch: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_refactor_final_switch: d grammar .so not found");
        return;
    };
    let query_str = loader.get_refactor("d").expect("d refactor query missing");
    let captures = collect_captures_full(&lang, D_VARIANTS, &query_str);
    assert!(
        captures
            .iter()
            .any(|(cap, kind, ..)| cap == "refactor.statement" && kind == "final_switch_statement"),
        "expected final_switch_statement classified as @refactor.statement, got: {captures:?}"
    );
}
