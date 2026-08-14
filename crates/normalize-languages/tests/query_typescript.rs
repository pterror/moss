//! Query fixture tests for typescript.
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
// TypeScript
// ---------------------------------------------------------------------------

const TS_SAMPLE: &str = include_str!("fixtures/typescript/sample.ts");

const TS_VARIANTS: &str = include_str!("fixtures/typescript/variants.ts");

// --- Dimension 4: real-world fixture coverage (sample.ts) -------------------

#[test]
fn typescript_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_tags: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let names = collect_captures(&lang, TS_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"FileLogger".to_string()),
        "expected 'FileLogger' class in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"formatPath".to_string()),
        "expected 'formatPath' function in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"groupBy".to_string()),
        "expected 'groupBy' function in typescript tags, got: {names:?}"
    );
    // Widget extends Entity implements Comparable<Widget> — both the
    // superclass and the generic interface must be found as references.
    assert!(
        names.contains(&"Entity".to_string()),
        "expected 'Entity' superclass reference in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Comparable".to_string()),
        "expected 'Comparable' generic interface reference in typescript tags, got: {names:?}"
    );
    // Private method #computeScore must be found as a method definition, not
    // silently dropped for having a private_property_identifier name.
    assert!(
        names.iter().any(|n| n == "#computeScore"),
        "expected private method '#computeScore' in typescript tags, got: {names:?}"
    );
    // `namespace Shapes { ... namespace Nested { ... } }` — both the outer
    // and nested namespace must be found as definition.module (internal_module).
    assert!(
        names.contains(&"Shapes".to_string()),
        "expected 'Shapes' namespace in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Nested".to_string()),
        "expected nested 'Nested' namespace in typescript tags, got: {names:?}"
    );
    // Closures assigned inside makeCounter must never appear as definitions.
    assert!(
        names.contains(&"makeCounter".to_string()),
        "expected 'makeCounter' function in typescript tags, got: {names:?}"
    );
}

#[test]
fn typescript_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_calls: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typescript")
        .expect("typescript calls query missing");
    let calls = collect_captures(&lang, TS_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "normalize" || c == "log" || c == "push"),
        "expected at least one of normalize/log/push calls in typescript sample, got: {calls:?}"
    );
    // Private method call site: this.#computeScore() inside score().
    assert!(
        calls.iter().any(|c| c == "#computeScore"),
        "expected private method call '#computeScore' in typescript sample, got: {calls:?}"
    );
    // Promise chain idiom: .then(...).catch(...) — both calls found.
    assert!(
        calls.iter().any(|c| c == "then"),
        "expected 'then' call in typescript sample, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "catch"),
        "expected 'catch' call in typescript sample, got: {calls:?}"
    );
}

#[test]
fn typescript_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_imports: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("typescript")
        .expect("typescript imports query missing");
    let paths = collect_captures(&lang, TS_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"events".to_string()),
        "expected 'events' in typescript import paths, got: {paths:?}"
    );
    assert!(
        paths.contains(&"path".to_string()),
        "expected 'path' in typescript import paths, got: {paths:?}"
    );
}

#[test]
fn typescript_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_complexity: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("typescript")
        .expect("typescript complexity query missing");
    let complexity = collect_captures(&lang, TS_SAMPLE, &query_str, "complexity");
    // formatPath has an if; groupBy has a for_in
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in typescript sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn typescript_types_finds_interface_and_class() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_types: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("typescript")
        .expect("typescript types query missing");
    let names = collect_captures(&lang, TS_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"FileLogger".to_string()) || names.contains(&"Logger".to_string()),
        "expected 'FileLogger' or 'Logger' in typescript types captures, got: {names:?}"
    );
}

#[test]
fn typescript_types_finds_extends_and_implements_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_types_extends_implements: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_types_extends_implements: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("typescript")
        .expect("typescript types query missing");
    let refs = collect_captures(&lang, TS_SAMPLE, &query_str, "type.reference");
    // class Widget extends Entity implements Comparable<Widget>
    assert!(
        refs.contains(&"Entity".to_string()),
        "expected 'Entity' extends-reference in typescript types, got: {refs:?}"
    );
    assert!(
        refs.contains(&"Comparable".to_string()),
        "expected 'Comparable' implements-reference (generic) in typescript types, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.ts) -

/// Every grammar-legal variant of `call_expression.function` that
/// typescript.calls.scm claims to support must actually match, with the
/// right capture *kind* (dimension 3) — not just the right text.
#[test]
fn typescript_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_calls_completeness: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typescript")
        .expect("typescript calls query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);

    // (capture_name, kind, text) triples we require, one per documented
    // function-field variant. See typescript.calls.scm's own comments for
    // the node-shape each of these lines exercises.
    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"),                  // plainCall
        ("call", "property_identifier", "push"), // methodCall: function: member_expression, property: property_identifier
        ("call", "private_property_identifier", "#compute"), // callPrivate: private method call
        ("call", "subscript_expression", "arr[0]"), // computedCall
        ("call", "parenthesized_expression", "(identity)"), // parenthesizedCall
        ("call", "non_null_expression", "identity!"), // nonNullCall
        ("call", "call_expression", "curried()"), // chainedCall
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in typescript.calls.scm \
             output for variants.ts, got: {caps:?}"
        );
    }

    // @call.qualifier must be present for method/computed calls and carry the
    // qualifier text, not the call name.
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
fn typescript_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_calls_negative: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typescript")
        .expect("typescript calls query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);
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
/// typescript.tags.scm claims to support (plain, private, computed) must
/// produce a @definition.method capture with the correct name text.
#[test]
fn typescript_tags_completeness_all_method_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_methods: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!(
            "Skipping typescript_tags_completeness_methods: typescript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);
    let method_defs: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "definition.method")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // definition.method is anchored on the whole method_definition node, so
    // check by substring/kind pairing on the @name capture instead.
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
    let _ = method_defs; // kept for readability of what's being asserted above
}

/// Every grammar-legal variant of `new_expression.constructor` that
/// typescript.tags.scm claims to support (plain identifier, namespaced
/// member_expression) must produce a @reference.class capture.
#[test]
fn typescript_tags_completeness_new_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_new: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_tags_completeness_new: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let class_refs = tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "reference.class");
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "PrivateHolder"),
        "expected plain constructor 'PrivateHolder' (identifier), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "member_expression" && t == "ns2.Ctor"),
        "expected namespaced constructor 'ns2.Ctor' (member_expression), got: {class_refs:?}"
    );
}

/// Every grammar-legal variant of `module`/`internal_module` name (identifier,
/// nested_identifier, ambient string) that typescript.tags.scm claims to
/// support must produce a @definition.module capture.
#[test]
fn typescript_tags_completeness_module_and_namespace_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_modules: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!(
            "Skipping typescript_tags_completeness_modules: typescript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let module_defs = tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "definition.module");
    // `module LegacyModule {}` — module.name: identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "LegacyModule"),
        "expected legacy 'module LegacyModule' (module.name: identifier), got: {module_defs:?}"
    );
    // `module Legacy.Dotted {}` — module.name: nested_identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "nested_identifier" && t == "Legacy.Dotted"),
        "expected legacy 'module Legacy.Dotted' (module.name: nested_identifier), got: {module_defs:?}"
    );
    // `declare module "ambient-module-name" {}` — module.name: string
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "string" && t == "\"ambient-module-name\""),
        "expected ambient 'declare module \"...\"' (module.name: string), got: {module_defs:?}"
    );
    // `namespace SimpleNamespace {}` — internal_module.name: identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "SimpleNamespace"),
        "expected 'namespace SimpleNamespace' (internal_module.name: identifier), got: {module_defs:?}"
    );
    // `namespace Dotted.Nested {}` — internal_module.name: nested_identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "nested_identifier" && t == "Dotted.Nested"),
        "expected 'namespace Dotted.Nested' (internal_module.name: nested_identifier), got: {module_defs:?}"
    );
}

/// Every grammar-legal variant of `extends_clause`/`implements_clause` that
/// typescript.tags.scm claims to support must produce the correct
/// reference.class/reference.implementation capture.
#[test]
fn typescript_tags_completeness_extends_implements_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_extends: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!(
            "Skipping typescript_tags_completeness_extends: typescript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let class_refs = tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "reference.class");
    let impl_refs =
        tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "reference.implementation");
    let impl_ref_texts: Vec<&str> = impl_refs.iter().map(|(_, t)| t.as_str()).collect();

    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "Base"),
        "expected 'Base' extends-reference (identifier), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "member_expression" && t == "ns2.Ctor"),
        "expected 'ns2.Ctor' extends-reference (member_expression), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "call_expression" && t.starts_with("Mixin(")),
        "expected 'Mixin(Base)' extends-reference (call_expression, mixin pattern), got: {class_refs:?}"
    );
    assert!(
        impl_ref_texts.contains(&"Iface"),
        "expected 'Iface' implements-reference (plain type_identifier), got: {impl_refs:?}"
    );
    assert!(
        impl_ref_texts.contains(&"GenericIface"),
        "expected 'GenericIface' implements-reference (generic_type), got: {impl_refs:?}"
    );
}

/// Negative case: closures are not function_declarations/method_definitions
/// and must never appear as @definition.function or @definition.method.
#[test]
fn typescript_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_tags_negative: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);
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
/// that typescript.imports.scm claims to support must produce a correctly
/// shaped @import capture, including the previously-silent `default`-name
/// (anonymous-token) and `import X = require(...)`/`import()` gaps.
#[test]
fn typescript_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_imports_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_imports_completeness: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("typescript")
        .expect("typescript imports query missing");
    let names = collect_captures(&lang, TS_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, TS_VARIANTS, &query_str, "import.alias");
    let paths = collect_captures(&lang, TS_VARIANTS, &query_str, "import.path");
    let globs = collect_captures(&lang, TS_VARIANTS, &query_str, "import.glob");
    let reexports = collect_captures(&lang, TS_VARIANTS, &query_str, "import.reexport");

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
    // import fsThing = require('fs') — TS import-equals with require.
    assert!(
        names.contains(&"fsThing".to_string()) && paths.contains(&"fs".to_string()),
        "expected 'fsThing'/'fs' from import-equals-require, names={names:?} paths={paths:?}"
    );
    // export * as wildcardNs from ... — namespace re-export.
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture, got: {globs:?}"
    );
    assert!(
        aliases.contains(&"wildcardNs".to_string()),
        "expected 'wildcardNs' namespace re-export alias, got: {aliases:?}"
    );
    // export { default } from ... (bare default re-export) — must appear.
    assert!(
        reexports.len() >= 2,
        "expected multiple @import.reexport captures (named + default forms), got {}: {reexports:?}",
        reexports.len()
    );
    // export { default as renamedDefaultReexport } from ...
    assert!(
        aliases.contains(&"renamedDefaultReexport".to_string()),
        "expected 'renamedDefaultReexport' aliased-default-reexport alias, got: {aliases:?}"
    );
    // import('mod-dynamic') — dynamic import expression.
    assert!(
        paths.contains(&"mod-dynamic".to_string()),
        "expected 'mod-dynamic' from dynamic import(), got: {paths:?}"
    );
}

#[test]
fn typescript_decorations_finds_decorator_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping typescript_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "typescript",
        TS_SAMPLE,
        &["@Injectable()"],
    );
}
