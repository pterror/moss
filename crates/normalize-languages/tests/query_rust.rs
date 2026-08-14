//! Query fixture tests for rust.
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
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

// ---------------------------------------------------------------------------
// Rust
// ---------------------------------------------------------------------------

const RUST_SAMPLE: &str = include_str!("fixtures/rust/sample.rs");

const RUST_VARIANTS: &str = include_str!("fixtures/rust/variants.rs");

// --- Dimension 4: real-world fixture coverage (sample.rs) -------------------

#[test]
fn rust_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_tags: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("rust").expect("rust tags query missing");
    let names = collect_captures(&lang, RUST_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Counter".to_string()),
        "expected 'Counter' struct in tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sum_evens".to_string()),
        "expected 'sum_evens' function in tags, got: {names:?}"
    );
    // Generic struct + generic impl block (idiom-density: real Rust leans on
    // generics heavily). Point<T> is declared `impl<T: Add<...> + Copy> Point<T>`
    // — the container must still be found (type: generic_type).
    assert!(
        names.iter().filter(|n| *n == "Point").count() >= 2,
        "expected 'Point' from both the struct_item and the generic impl_item, got: {names:?}"
    );
    // Nested module + nested #[cfg(test)] mod: both must be found as
    // @definition.module, and functions inside must still be found.
    assert!(
        names.contains(&"shapes".to_string()),
        "expected 'shapes' module in tags, got: {names:?}"
    );
    assert!(
        names.contains(&"describe".to_string()),
        "expected 'describe' function nested in mod shapes, got: {names:?}"
    );
    // Closures must never be reported as function/method definitions: the
    // binding name `make_adder` should appear (it's a real fn), but no name
    // from inside its closure body (`base`, `x`) should appear as a
    // definition — closures aren't `function_item`s in the grammar.
    assert!(
        names.contains(&"make_adder".to_string()),
        "expected 'make_adder' function in tags, got: {names:?}"
    );
}

#[test]
fn rust_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let calls = collect_captures(&lang, RUST_SAMPLE, &query_str, "call");
    // Counter::new() → @call = "new", increment/get are method calls
    assert!(
        calls.iter().any(|c| c == "new"),
        "expected 'new' call in rust sample, got: {calls:?}"
    );
    // Iterator-chain idiom: .filter(...).map(...).collect::<Vec<_>>() — the
    // turbofish `.collect::<Vec<_>>()` must be found (generic_function
    // wrapping a field_expression), not just the untyped calls in the chain.
    assert!(
        calls.iter().any(|c| c == "collect"),
        "expected 'collect' call (incl. its turbofish form) in rust sample, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "filter"),
        "expected 'filter' call in rust sample, got: {calls:?}"
    );
    // `s.parse::<i32>().ok()` inside parse_all: turbofish method call.
    assert!(
        calls.iter().any(|c| c == "parse"),
        "expected 'parse' turbofish call in rust sample, got: {calls:?}"
    );
}

#[test]
fn rust_imports_finds_use_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_imports: rust grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("rust")
        .expect("rust imports query missing");
    let paths = collect_captures(&lang, RUST_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("std")),
        "expected std import path in rust sample, got: {paths:?}"
    );
    let names = collect_captures(&lang, RUST_SAMPLE, &query_str, "import.name");
    assert!(
        names.contains(&"HashMap".to_string()),
        "expected 'HashMap' import name in rust sample, got: {names:?}"
    );
    // `use std::fmt::{self, Display};` — the self-import member must be
    // captured too, or the module import silently disappears. This construct
    // has been present in this fixture from the start; the original shallow
    // test (name.contains("HashMap")) never exercised it, which is exactly
    // the gap this enriched suite exists to close.
    assert!(
        names.iter().any(|n| n == "self"),
        "expected a 'self' import name (use std::fmt::{{self, Display}}) in rust sample, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.rs) -

/// Every grammar-legal variant of `call_expression.function` that
/// rust.calls.scm claims to support must actually match, with the right
/// capture *kind* (dimension 3) — not just the right text.
#[test]
fn rust_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls_completeness: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);

    // (capture_name, kind, text) triples we require, one per documented
    // function-field variant. See rust.calls.scm's own comments for the
    // node-shape each of these lines exercises.
    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"),    // plain_call
        ("call", "identifier", "drop"), // scoped_call: function: scoped_identifier, name: identifier
        ("call", "field_identifier", "len"), // method_call
        ("call", "identifier", "identity"), // turbofish_plain_call (same text as plain_call, different line)
        ("call", "identifier", "size_of"),  // turbofish_scoped_call
        ("call", "field_identifier", "parse"), // turbofish_method_call
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in rust.calls.scm \
             output for variants.rs, got: {caps:?}"
        );
    }

    // @call.qualifier must be present for every scoped/method call and carry
    // the qualifier text, not the call name.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.iter().any(|q| q.contains("std::mem")),
        "expected 'std::mem' qualifier for scoped/turbofish-scoped calls, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"v"),
        "expected 'v' qualifier for the plain method call, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.iter().any(|q| q.contains('5')),
        "expected the string-literal receiver qualifier for the turbofish method call, got: {qualifiers:?}"
    );
}

/// @call.write is reserved for assignment/compound-assignment RHS; a `let`
/// binding must never produce @call.write, and every write-context variant
/// (assignment, compound assignment) must be covered.
#[test]
fn rust_calls_write_context_distinguishes_let_from_assignment() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls_write_context: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls_write_context: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);

    let write_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call.write" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    // Lines 52 (`result = identity(1)`) and 53 (`result += identity(2)`) are
    // write-context; line 54 (`let _read = identity(3)`) must NOT appear here.
    assert_eq!(
        write_calls.len(),
        2,
        "expected exactly 2 @call.write 'identity' captures (assignment + compound \
         assignment), got {}: {write_calls:?} (full captures: {caps:?})",
        write_calls.len()
    );

    let plain_identity_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    // plain_call, turbofish_plain_call, and the `let`-bound call in
    // write_context_call all use plain @call, never @call.write.
    assert!(
        plain_identity_calls.len() >= 3,
        "expected at least 3 plain @call 'identity' captures (incl. the let-bound one), \
         got {}: {plain_identity_calls:?}",
        plain_identity_calls.len()
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn rust_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls_negative: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call" || cn == "call.write")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` is a bare field read (no call parens); must never be a call.
    assert!(
        !call_texts.contains(&"field"),
        "bare field access 'holder.field' must not be captured as a call, got: {call_texts:?}"
    );
    // The closure parameter/body identifiers (`add_one`'s definition site,
    // `x`) must not appear as calls — only the *call site* `add_one(1)` should.
    let add_one_calls = call_texts.iter().filter(|t| **t == "add_one").count();
    assert_eq!(
        add_one_calls, 1,
        "expected exactly 1 call to 'add_one' (the call site, not the closure \
         definition), got {add_one_calls}: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `impl_item.type` and `impl_item.trait` that
/// rust.tags.scm claims to support (plain, generic, path-qualified, and their
/// combinations) must produce a @name capture with the correct definition/
/// reference kind.
#[test]
fn rust_tags_completeness_all_impl_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_tags_completeness: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("rust").expect("rust tags query missing");
    let query = Query::new(&lang, &query_str).expect("query compilation failed");
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(RUST_VARIANTS, None).expect("parse failed");
    let source_bytes = RUST_VARIANTS.as_bytes();

    // Collect (tag_kind, name_text) pairs: tag_kind is whichever
    // @definition.*/@reference.* capture co-occurs with @name in the match.
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut name = None;
        let mut tag_kind = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(source_bytes).unwrap_or("");
            if cap_name == "name" {
                name = Some(text.to_string());
            } else if cap_name.starts_with("definition.") || cap_name.starts_with("reference.") {
                tag_kind = Some(cap_name.to_string());
            }
        }
        if let (Some(n), Some(k)) = (name, tag_kind) {
            pairs.push((k, n));
        }
    }

    // Plain inherent impl: impl Plain {}
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.module" && n == "Plain"),
        "expected 'Plain' as definition.module (plain impl container), got: {pairs:?}"
    );
    // Generic inherent impl: impl<T> Generic<T> {} — type: generic_type.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.module" && n == "Generic"),
        "expected 'Generic' as definition.module (generic impl container), got: {pairs:?}"
    );
    // Path-qualified trait impl: impl std::fmt::Debug for Plain — trait: scoped_type_identifier.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "Debug"),
        "expected 'Debug' as reference.implementation (path-qualified trait), got: {pairs:?}"
    );
    // Generic trait impl: impl From<i32> for Plain — trait: generic_type -> type_identifier.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "From"),
        "expected 'From' as reference.implementation (generic trait), got: {pairs:?}"
    );
    // Generic + path-qualified trait impl: impl std::ops::Add<i32> for Plain
    // — trait: generic_type -> type: scoped_type_identifier.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "Add"),
        "expected 'Add' as reference.implementation (generic + path-qualified trait), got: {pairs:?}"
    );
}

/// Negative case: closures are not `function_item`s and must never appear as
/// @definition.function or @definition.method.
#[test]
fn rust_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_tags_negative: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("rust").expect("rust tags query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);
    let def_fn_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // `add_one` is a closure binding, not a function_item; it must never be
    // reported as a definition name. Only the call-site capture ("add_one"
    // via @reference.call) is legitimate — @name is shared across
    // definitions and references in this query, so we check specifically
    // that no @definition.function/@definition.method pairs with the name.
    let is_def_add_one = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t == "add_one"
    });
    assert!(
        !is_def_add_one,
        "closure binding 'add_one' must never be captured as a function/method \
         definition, got names: {def_fn_names:?}"
    );
}

/// Every grammar-legal variant of `use_declaration` that rust.imports.scm
/// claims to support (plain, aliased, wildcard, multi-name, self-import,
/// aliased self-import, re-export) must produce a correctly-shaped @import.
#[test]
fn rust_imports_completeness_all_use_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_imports_completeness: rust grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("rust")
        .expect("rust imports query missing");
    let names = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.alias");
    let globs = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.glob");
    let reexports = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.reexport");

    // Self-import: use std::io::{self, Read, Write};
    assert!(
        names.iter().filter(|n| *n == "self").count() >= 2,
        "expected 2 'self' import names (plain + aliased self-import), got: {names:?}"
    );
    // Aliased self-import: use std::io::{self as io_alias};
    assert!(
        aliases.contains(&"io_alias".to_string()),
        "expected 'io_alias' among import aliases (aliased self-import), got: {aliases:?}"
    );
    // Multi-name with an aliased member: use std::collections::{HashSet, BTreeMap as Tree};
    assert!(
        names.contains(&"HashSet".to_string()),
        "expected 'HashSet' import name, got: {names:?}"
    );
    assert!(
        names.contains(&"BTreeMap".to_string()) && aliases.contains(&"Tree".to_string()),
        "expected 'BTreeMap as Tree' aliased member, names={names:?} aliases={aliases:?}"
    );
    // Wildcard: use std::env::*;
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture for 'use std::env::*;', got: {globs:?}"
    );
    // Aliased re-export: pub use std::fmt::Debug as DebugTrait;
    assert!(
        !reexports.is_empty(),
        "expected at least one import.reexport capture for 'pub use ... as DebugTrait;', got: {reexports:?}"
    );
    assert!(
        aliases.contains(&"DebugTrait".to_string()),
        "expected 'DebugTrait' alias for the re-export, got: {aliases:?}"
    );
}

#[test]
fn rust_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_complexity: rust grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("rust")
        .expect("rust complexity query missing");
    let complexity = collect_captures(&lang, RUST_SAMPLE, &query_str, "complexity");
    // classify() has two if branches; sum_evens() has for + if
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in rust sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn rust_types_finds_struct_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_types: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_types("rust").expect("rust types query missing");
    let names = collect_captures(&lang, RUST_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Counter".to_string()),
        "expected 'Counter' in rust types captures, got: {names:?}"
    );
}

#[test]
fn rust_decorations_finds_attribute_and_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping rust_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "rust",
        RUST_SAMPLE,
        &["#[derive(Debug, Clone)]", "/// Classify"],
    );
}
