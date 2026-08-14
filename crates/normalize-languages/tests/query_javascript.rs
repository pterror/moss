//! Query fixture tests for javascript.
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
// JavaScript
// ---------------------------------------------------------------------------

const JAVASCRIPT_SAMPLE: &str = include_str!("fixtures/javascript/sample.js");

const JAVASCRIPT_VARIANTS: &str = include_str!("fixtures/javascript/variants.js");

// --- Dimension 4: real-world fixture coverage (sample.js) -------------------

#[test]
fn javascript_tags_finds_functions_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_tags: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let names = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Stack" || n == "classify" || n == "fibonacci"),
        "expected 'Stack'/'classify'/'fibonacci' in javascript tags, got: {names:?}"
    );
    // SerializableStack extends Serializable(Stack) — the mixin-pattern
    // superclass expression must still surface Stack via @reference.class.
    assert!(
        names.contains(&"SerializableStack".to_string()),
        "expected 'SerializableStack' class in javascript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' superclass reference in javascript tags, got: {names:?}"
    );
    // Private method #peek must be found as a method definition, not
    // silently dropped for having a private_property_identifier name.
    assert!(
        names.iter().any(|n| n == "#peek"),
        "expected private method '#peek' in javascript tags, got: {names:?}"
    );
    // Generator function must still be found as a function definition.
    assert!(
        names.contains(&"range".to_string()),
        "expected generator function 'range' in javascript tags, got: {names:?}"
    );
}

#[test]
fn javascript_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_calls: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("javascript")
        .expect("javascript calls query missing");
    let calls = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "fibonacci" || c == "push"),
        "expected a function call in javascript sample, got: {calls:?}"
    );
    // Private method call site: this.#peek() inside SerializableStack.
    assert!(
        calls.iter().any(|c| c == "#peek"),
        "expected private method call '#peek' in javascript sample, got: {calls:?}"
    );
    // Tagged template call: html`<h1>${resolved}</h1>` — arguments is a bare
    // template_string, not the usual `arguments` node.
    assert!(
        calls.iter().any(|c| c == "html"),
        "expected tagged-template call 'html' in javascript sample, got: {calls:?}"
    );
    // Computed/bracket call: dispatch['classify'](0).
    assert!(
        calls.iter().any(|c| c.contains("dispatch")),
        "expected computed/bracket call on 'dispatch' in javascript sample, got: {calls:?}"
    );
}

#[test]
fn javascript_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_complexity: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("javascript")
        .expect("javascript complexity query missing");
    let complexity = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in javascript sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn javascript_imports_finds_es_module_imports() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_imports: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("javascript")
        .expect("javascript imports query missing");
    let paths = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p == "events" || p == "path" || p == "fs"),
        "expected module paths in javascript imports, got: {paths:?}"
    );
}

#[test]
fn javascript_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_types: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("javascript")
        .expect("javascript types query missing");
    let refs = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in javascript sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.js) -

/// Every grammar-legal variant of `call_expression.function` that
/// javascript.calls.scm claims to support must actually match, with the
/// right capture *kind* (dimension 3) — not just the right text.
#[test]
fn javascript_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_calls_completeness: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("javascript")
        .expect("javascript calls query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"),                  // plainCall
        ("call", "property_identifier", "push"), // methodCall: function: member_expression, property: property_identifier
        ("call", "private_property_identifier", "#compute"), // callPrivate: private method call
        ("call", "subscript_expression", "arr[0]"), // computedCall
        ("call", "parenthesized_expression", "(function iife() {})"), // parenthesizedCall (IIFE)
        ("call", "call_expression", "curried()"), // chainedCall
        ("call", "identifier", "taggedTemplateCall"), // tagged template call (arguments: template_string)
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in javascript.calls.scm \
             output for variants.js, got: {caps:?}"
        );
    }

    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"arr"),
        "expected 'arr' qualifier for the plain method call, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"this"),
        "expected 'this' qualifier for the private method call, got: {qualifiers:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn javascript_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_calls_negative: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("javascript")
        .expect("javascript calls query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` is a bare field read (no call parens); must never be a call.
    assert!(
        !call_texts.contains(&"field"),
        "bare field access 'holder.field' must not be captured as a call, got: {call_texts:?}"
    );
    // The closure definition site (`addOne`) must not appear as a call —
    // only the call site `addOne(1)` should.
    let add_one_calls = call_texts.iter().filter(|t| **t == "addOne").count();
    assert_eq!(
        add_one_calls, 1,
        "expected exactly 1 call to 'addOne' (the call site, not the closure \
         definition), got {add_one_calls}: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `method_definition.name` that
/// javascript.tags.scm claims to support (plain, private, computed) must
/// produce a @name capture with the correct kind.
#[test]
fn javascript_tags_completeness_all_method_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_tags_completeness_methods: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!(
            "Skipping javascript_tags_completeness_methods: javascript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);
    let name_kinds: Vec<(&str, &str)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, k, t, _)| (k.as_str(), t.as_str()))
        .collect();
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "property_identifier" && *t == "plainMethod"),
        "expected plain method name 'plainMethod' (property_identifier), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "private_property_identifier" && *t == "#privateMethod"),
        "expected private method name '#privateMethod' (private_property_identifier), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "computed_property_name" && *t == "[\"computedMethod\"]"),
        "expected computed method name (computed_property_name), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "property_identifier" && *t == "staticMethod"),
        "expected static method name 'staticMethod' (property_identifier), got: {name_kinds:?}"
    );
}

/// Every grammar-legal variant of class_heritage's superclass expression
/// (identifier, member_expression, call_expression/mixin) must produce a
/// @reference.class capture.
#[test]
fn javascript_tags_completeness_class_heritage_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_tags_completeness_heritage: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!(
            "Skipping javascript_tags_completeness_heritage: javascript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let class_refs =
        tags_matches_by_kind(&lang, JAVASCRIPT_VARIANTS, &query_str, "reference.class");
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "Base"),
        "expected 'Base' extends-reference (identifier), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "member_expression" && t == "nsObj.Ctor"),
        "expected 'nsObj.Ctor' extends-reference (member_expression), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "call_expression" && t.starts_with("Mixin(")),
        "expected 'Mixin(Base)' extends-reference (call_expression, mixin pattern), got: {class_refs:?}"
    );
}

/// Every grammar-legal variant of `new_expression.constructor` (already a
/// wildcard `(_)` in javascript.tags.scm) must produce a @reference.class
/// capture regardless of shape.
#[test]
fn javascript_tags_completeness_new_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_tags_completeness_new: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_tags_completeness_new: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let names = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "name");
    assert!(
        names.contains(&"PrivateHolder".to_string()),
        "expected plain constructor 'PrivateHolder', got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "nsObj.Ctor"),
        "expected namespaced constructor 'nsObj.Ctor' (member_expression), got: {names:?}"
    );
}

/// Negative case: closures are not function_declarations/method_definitions
/// and must never appear as @definition.function or @definition.method.
#[test]
fn javascript_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_tags_negative: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);
    let is_def_add_one = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t == "addOne"
    });
    assert!(
        !is_def_add_one,
        "closure binding 'addOne' must never be captured as a function/method \
         definition, got captures: {caps:?}"
    );
}

/// Every grammar-legal variant of import/re-export/require/dynamic-import
/// that javascript.imports.scm claims to support must produce a correctly
/// shaped @import capture, including the previously-silent `default`-name
/// (anonymous-token) gap.
#[test]
fn javascript_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_imports_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_imports_completeness: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("javascript")
        .expect("javascript imports query missing");
    let names = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.alias");
    let paths = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.path");
    let globs = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.glob");
    let reexports = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.reexport");

    assert!(
        names.contains(&"plainName".to_string()),
        "expected plain import name, got: {names:?}"
    );
    // import { default as renamedDefault } — previously silently dropped
    // entirely since `default` is an anonymous token, not (identifier).
    assert!(
        names.iter().any(|n| n == "default"),
        "expected a 'default' import name (import {{ default as ... }}), got: {names:?}"
    );
    assert!(
        aliases.contains(&"renamedDefault".to_string()),
        "expected 'renamedDefault' alias for the default-import, got: {aliases:?}"
    );
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture, got: {globs:?}"
    );
    assert!(
        aliases.contains(&"wildcardNs".to_string()),
        "expected 'wildcardNs' namespace re-export alias, got: {aliases:?}"
    );
    assert!(
        reexports.len() >= 2,
        "expected multiple @import.reexport captures (named + default forms), got {}: {reexports:?}",
        reexports.len()
    );
    assert!(
        aliases.contains(&"renamedDefaultReexport".to_string()),
        "expected 'renamedDefaultReexport' aliased-default-reexport alias, got: {aliases:?}"
    );
    // const { statSync } = require('fs') — destructured require, shorthand.
    assert!(
        names.contains(&"statSync".to_string()),
        "expected 'statSync' from destructured require, got: {names:?}"
    );
    // import('mod-dynamic') — dynamic import expression.
    assert!(
        paths.contains(&"mod-dynamic".to_string()),
        "expected 'mod-dynamic' from dynamic import(), got: {paths:?}"
    );
}

#[test]
fn javascript_decorations_finds_decorator_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping javascript_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "javascript",
        JAVASCRIPT_SAMPLE,
        &["@sealed", "// A stack data structure"],
    );
}
