//! Query fixture tests for c.
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
// C
// ---------------------------------------------------------------------------

const C_SAMPLE: &str = include_str!("fixtures/c/sample.c");

const C_VARIANTS: &str = include_str!("fixtures/c/variants.c");

// --- Dimension 4: real-world fixture coverage (sample.c) --------------------

#[test]
fn c_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let names = collect_captures(&lang, C_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' struct in c tags, got: {names:?}"
    );
    assert!(
        names.contains(&"stack_new".to_string()) && names.contains(&"classify".to_string()),
        "expected 'stack_new' and 'classify' functions in c tags, got: {names:?}"
    );
    // Real-world callback-typedef idiom: `typedef int (*Comparator)(...)` —
    // previously dropped entirely (see c.tags.scm's own comments).
    assert!(
        names.contains(&"Comparator".to_string()),
        "expected 'Comparator' callback typedef in c tags, got: {names:?}"
    );
    // Tagged-union idiom: `union Cell { ... };` — previously mislabeled or
    // missed outright depending on shape (the struct/union asymmetry bug).
    assert!(
        names.contains(&"Cell".to_string()),
        "expected 'Cell' union in c tags, got: {names:?}"
    );
    // Object-like and function-like macros — zero tags coverage before this fix.
    assert!(
        names.contains(&"MAX_CAPACITY".to_string()),
        "expected 'MAX_CAPACITY' macro in c tags, got: {names:?}"
    );
    assert!(
        names.contains(&"CLAMP".to_string()),
        "expected 'CLAMP' function-like macro in c tags, got: {names:?}"
    );
}

#[test]
fn c_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_calls: c grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("c").expect("c calls query missing");
    let calls = collect_captures(&lang, C_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"malloc".to_string()) && calls.contains(&"printf".to_string()),
        "expected 'malloc' and 'printf' calls in c sample, got: {calls:?}"
    );
    // Callback idiom: qsort(..., cmp) plus a direct call through the
    // Comparator-typed function-pointer variable.
    assert!(
        calls.contains(&"qsort".to_string()),
        "expected 'qsort' call in c sample, got: {calls:?}"
    );
    assert!(
        calls.iter().filter(|c| *c == "cmp").count() >= 1,
        "expected at least 1 call through the 'cmp' function-pointer variable, got: {calls:?}"
    );
}

#[test]
fn c_imports_finds_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_imports: c grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("c").expect("c imports query missing");
    let paths = collect_captures(&lang, C_SAMPLE, &query_str, "import.path");
    // Raw capture text still carries the angle brackets (`<stdio.h>`); the
    // Rust-side extraction layer strips them, not the query itself.
    assert!(
        paths.iter().any(|p| p.contains("stdio.h"))
            && paths.iter().any(|p| p.contains("stdlib.h"))
            && paths.iter().any(|p| p.contains("string.h")),
        "expected all three system includes in c import paths, got: {paths:?}"
    );
}

#[test]
fn c_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_complexity: c grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("c")
        .expect("c complexity query missing");
    let complexity = collect_captures(&lang, C_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 5,
        "expected at least 5 complexity nodes in c sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn c_types_finds_type_identifiers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_types: c grammar .so not found");
        return;
    };
    let query_str = loader.get_types("c").expect("c types query missing");
    let refs = collect_captures(&lang, C_SAMPLE, &query_str, "type");
    assert!(
        refs.iter().any(|r| r == "Stack"),
        "expected 'Stack' in c type references, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.c) -

/// Every grammar-legal name+body variant of `struct_specifier`/`union_specifier`
/// that c.tags.scm claims to support — bare, typedef'd-anonymous, and
/// typedef'd-named — must produce a @definition.class capture with the
/// correct kind, not just the right text (dimension 3).
#[test]
fn c_tags_completeness_struct_and_union_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_completeness: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);

    // Bare union definition — the case the old query (declaration-wrapped
    // pattern) never matched at all.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "union_specifier"
            && t.contains("PlainUnion")),
        "expected 'PlainUnion' union_specifier as definition.class, got: {caps:?}"
    );
    // Named union nested inside a typedef — the other case the old query
    // never matched.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "union_specifier"
            && t.contains("TaggedUnion")),
        "expected 'TaggedUnion' union_specifier as definition.class, got: {caps:?}"
    );
    // The typedef alias itself is still captured via the (unrelated)
    // type_definition pattern.
    let type_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        type_names.contains(&"TaggedUnionAlias"),
        "expected 'TaggedUnionAlias' typedef name, got: {type_names:?}"
    );
    // Struct definitions still work unaffected by the union fix.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "struct_specifier"
            && t.contains("PlainStruct")),
        "expected 'PlainStruct' struct_specifier as definition.class, got: {caps:?}"
    );
}

/// Typedef'd function pointers (`typedef int (*FuncPtr)(int, int);`) must
/// produce a @definition.type capture for the alias name, verifying the
/// three-level-nested declarator pattern (function_declarator >
/// parenthesized_declarator > pointer_declarator > type_identifier).
#[test]
fn c_tags_completeness_typedef_function_pointer() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping c_tags_completeness_typedef_fnptr: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_completeness_typedef_fnptr: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "name" && k == "type_identifier" && t == "FuncPtr"),
        "expected 'FuncPtr' typedef'd-function-pointer name as (name, type_identifier), got: {caps:?}"
    );
}

/// Object-like and function-like macro definitions must both produce
/// @definition.macro captures — previously zero macro tags coverage at all.
#[test]
fn c_tags_completeness_macro_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags_completeness_macros: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_completeness_macros: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.macro"
            && k == "preproc_def"
            && t.contains("MAX_SIZE")),
        "expected object-like macro 'MAX_SIZE' as (definition.macro, preproc_def), got: {caps:?}"
    );
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.macro"
            && k == "preproc_function_def"
            && t.contains("SQUARE")),
        "expected function-like macro 'SQUARE' as (definition.macro, preproc_function_def), got: {caps:?}"
    );
}

/// Negative cases: constructs that must never be tagged as
/// @definition.function/@definition.class.
#[test]
fn c_tags_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_negative: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);

    // `int (*negative_function_pointer_variable)(int);` declares a variable
    // of function-pointer type, not a function — function_declarator's
    // declarator field is parenthesized_declarator, never a bare identifier.
    let def_fn_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "definition.function")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        !def_fn_names
            .iter()
            .any(|n| n.contains("negative_function_pointer_variable")),
        "function-pointer *variable* declaration must never be @definition.function, got: {def_fn_names:?}"
    );

    // `union NegativeUsage;` (bodyless forward reference) must never produce
    // @definition.class — this is exactly the false positive the old
    // declaration-wrapped union pattern produced.
    assert!(
        !caps
            .iter()
            .any(|(cn, _, t, _)| cn == "definition.class" && t.contains("NegativeUsage")),
        "bodyless union forward-reference 'NegativeUsage' must never be @definition.class, got: {caps:?}"
    );
}

/// Negative case: a bare field read through `->` with no call parens must
/// never appear in a @call capture.
#[test]
fn c_calls_negative_bare_field_access_is_not_a_call() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_calls_negative: c grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("c").expect("c calls query missing");
    let calls = collect_captures(&lang, C_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access 'holder->field' must not be captured as a call, got: {calls:?}"
    );
}

/// Field/pointer member calls through a function-pointer struct member
/// (`p->fp(...)`, `v.fp(...)`) must be captured with the correct qualifier,
/// for both the `->` and `.` operator forms.
#[test]
fn c_calls_completeness_field_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_calls_completeness: c grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("c").expect("c calls query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);
    let fp_calls: Vec<&(String, String, String, usize)> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "call" && k == "field_identifier" && t == "fp")
        .collect();
    assert_eq!(
        fp_calls.len(),
        2,
        "expected exactly 2 field-expression calls to 'fp' (via -> and via .), got {}: {fp_calls:?}",
        fp_calls.len()
    );
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"p") && qualifiers.contains(&"v"),
        "expected 'p' (-> form) and 'v' (. form) qualifiers, got: {qualifiers:?}"
    );
}

#[test]
fn c_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping c_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // #include is preproc_include, not preproc_call — the query captures comments and
    // generic preproc_call directives (#pragma etc.) but not #include or #define.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "c",
        C_SAMPLE,
        &["/* Creates a new stack with the given capacity. */"],
    );
}
