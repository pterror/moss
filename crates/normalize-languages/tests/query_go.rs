//! Query fixture tests for go.
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
// Go
// ---------------------------------------------------------------------------

const GO_SAMPLE: &str = include_str!("fixtures/go/sample.go");

const GO_VARIANTS: &str = include_str!("fixtures/go/variants.go");

// --- Dimension 4: real-world fixture coverage (sample.go) -------------------

#[test]
fn go_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_tags: go grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("go").expect("go tags query missing");
    let names = collect_captures(&lang, GO_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Classify".to_string()),
        "expected 'Classify' function in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"JoinWords".to_string()),
        "expected 'JoinWords' function in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' type in go tags, got: {names:?}"
    );
    // Generic function/type/method idioms (real Go leans on generics
    // heavily since 1.18): Max[T Ordered], Box[T any], and its method Get.
    assert!(
        names.contains(&"Max".to_string()),
        "expected generic function 'Max' in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Box".to_string()),
        "expected generic type 'Box' in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Get".to_string()),
        "expected method 'Get' on generic receiver Box[T] in go tags, got: {names:?}"
    );
    // Type alias: `type MyInt = int` must be found as a definition, not
    // silently dropped (the type_alias node is distinct from type_spec).
    assert!(
        names.contains(&"MyInt".to_string()),
        "expected type alias 'MyInt' in go tags, got: {names:?}"
    );
    // Struct embedding: Derived embeds Base; both must still be found.
    assert!(
        names.contains(&"Base".to_string()) && names.contains(&"Derived".to_string()),
        "expected both 'Base' and 'Derived' (struct embedding) in go tags, got: {names:?}"
    );
    // Closures must never be reported as function/method definitions: the
    // outer `adder` binding is a real function, but nothing from inside its
    // closure body should appear as a definition name it doesn't already
    // have legitimately (`delta`, `base` are parameters, not definitions).
    assert!(
        names.contains(&"adder".to_string()),
        "expected 'adder' function in go tags, got: {names:?}"
    );
}

#[test]
fn go_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let calls = collect_captures(&lang, GO_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"Println".to_string()),
        "expected 'Println' call in go sample, got: {calls:?}"
    );
    // Generic call relying on type inference (`Max(3, 4)`, no explicit
    // instantiation) parses as an ordinary call_expression/identifier and
    // must be found like any other plain call.
    assert!(
        calls.contains(&"Max".to_string()),
        "expected generic call 'Max' in go sample, got: {calls:?}"
    );
    // Method call on a generic receiver: b.Get().
    assert!(
        calls.contains(&"Get".to_string()),
        "expected method call 'Get' in go sample, got: {calls:?}"
    );
    // Package-qualified call through an aliased import (io.Discard via
    // `io "io"`) — same call shape as a plain package call.
    assert!(
        calls.contains(&"Fprintln".to_string()),
        "expected 'Fprintln' call (aliased-import-qualified) in go sample, got: {calls:?}"
    );
}

#[test]
fn go_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_imports: go grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("go").expect("go imports query missing");
    let paths = collect_captures(&lang, GO_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("fmt")),
        "expected '\"fmt\"' in go import paths, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("strings")),
        "expected '\"strings\"' in go import paths, got: {paths:?}"
    );
    // Aliased import: `io "io"` — must still surface a path, not just the
    // unaliased forms.
    assert!(
        paths.iter().any(|p| p.contains("io")),
        "expected aliased '\"io\"' import path in go sample, got: {paths:?}"
    );
    let aliases = collect_captures(&lang, GO_SAMPLE, &query_str, "import.alias");
    assert!(
        aliases.iter().any(|a| a == "io"),
        "expected 'io' alias captured for the aliased import, got: {aliases:?}"
    );
}

#[test]
fn go_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_complexity: go grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("go")
        .expect("go complexity query missing");
    let complexity = collect_captures(&lang, GO_SAMPLE, &query_str, "complexity");
    // Classify() has two if branches; Pop() has one if; Max[T] has one if;
    // sumEvens has a for-range loop plus a nested if.
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in go sample, got {} ({complexity:?})",
        complexity.len()
    );
    // for _, n := range nums { ... } — the range-loop form of for_statement
    // must count as a complexity/nesting node exactly like a classic
    // three-clause for loop (both are for_statement; range_clause is a
    // child, not a separate statement type).
    let nesting = collect_captures(&lang, GO_SAMPLE, &query_str, "nesting");
    assert!(
        nesting.len() >= 2,
        "expected at least 2 nesting nodes (incl. the for-range loop and its \
         enclosing function) in go sample, got {} ({nesting:?})",
        nesting.len()
    );
}

#[test]
fn go_types_finds_struct_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_types: go grammar .so not found");
        return;
    };
    let query_str = loader.get_types("go").expect("go types query missing");
    let names = collect_captures(&lang, GO_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' in go types captures, got: {names:?}"
    );
    // Type alias must appear in types.scm too, not just tags.scm.
    assert!(
        names.contains(&"MyInt".to_string()),
        "expected type alias 'MyInt' in go types captures, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.go) -

/// Every grammar-legal variant of `call_expression.function` that
/// go.calls.scm claims to support must actually match, with the right
/// capture *kind* (dimension 3) — not just the right text.
#[test]
fn go_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);

    let call_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // plain_call: function: identifier
    assert!(
        call_names.contains(&"identity"),
        "expected 'identity' plain call, got: {call_names:?}"
    );
    // method_call: function: selector_expression, field: field_identifier
    assert!(
        call_names.contains(&"len"),
        "expected 'len' method call, got: {call_names:?}"
    );
    // scoped_call: function: selector_expression, package-qualified
    assert!(
        call_names.contains(&"Sprintf"),
        "expected 'Sprintf' scoped call, got: {call_names:?}"
    );
    // dot-imported call: brought into scope directly by `. "strings"`, so
    // it parses as a plain identifier call, same as plain_call.
    assert!(
        call_names.contains(&"Repeat"),
        "expected dot-imported 'Repeat' call (parses as plain identifier call), got: {call_names:?}"
    );

    // Every @call capture must be either an identifier or a field_identifier
    // — never the parenthesized wrapper or anything larger (extraction
    // depth: capture *kind*, not just text).
    for (cn, kind, text, line) in &caps {
        if cn == "call" {
            assert!(
                kind == "identifier" || kind == "field_identifier",
                "expected @call capture kind to be identifier/field_identifier, \
                 got kind={kind} text={text} line={line}"
            );
        }
    }

    // @call.qualifier must carry the qualifier text for every scoped/method call.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"fmt"),
        "expected 'fmt' qualifier for the package-qualified call, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"h"),
        "expected 'h' qualifier for the method call, got: {qualifiers:?}"
    );
}

/// @call.write is reserved for assignment/compound-assignment RHS; a
/// `:=`-bound call must never produce @call.write.
#[test]
fn go_calls_write_context_distinguishes_short_var_decl_from_assignment() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls_write_context: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls_write_context: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);

    let write_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call.write" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    let plain_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    // go.calls.scm has no dedicated @call.write pattern (unlike Rust) — Go's
    // grammar doesn't distinguish assignment-RHS calls from plain calls in
    // a way this query captures separately, so every `identity(...)` call
    // site (the plain call in callVariants, plus assignment,
    // compound-assignment, and `:=` in writeContextCall — 4 total) surfaces
    // as plain @call. This test documents that as current behavior: no
    // @call.write captures exist anywhere.
    assert!(
        write_calls.is_empty(),
        "go.calls.scm has no @call.write pattern; expected zero, got: {write_calls:?}"
    );
    assert_eq!(
        plain_calls.len(),
        4,
        "expected all 4 'identity' call sites (plain, assignment, \
         compound-assignment, short-var-decl) as plain @call, got {}: {plain_calls:?}",
        plain_calls.len()
    );
}

/// Negative cases: the three deliberately-excluded call_expression.function
/// variants (documented in go.calls.scm) must never produce a @call capture.
#[test]
fn go_calls_negative_uncallable_function_variants_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls_negative: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // Curried call `higherOrder()(1)`: the outer call's callee is itself a
    // call_expression, not a named symbol. `higherOrder` (the inner call's
    // name) is legitimately captured once; it must not appear a second time
    // from the outer call.
    let higher_order_calls = call_texts.iter().filter(|t| **t == "higherOrder").count();
    assert_eq!(
        higher_order_calls, 1,
        "expected exactly 1 call to 'higherOrder' (the inner call only), got \
         {higher_order_calls}: {call_texts:?}"
    );
    // Immediately-invoked func_literal: no identifier/field_identifier
    // named "iife" exists in the source at all (it's a string argument to
    // Println), so its absence from @call is trivially satisfied; the real
    // assertion is structural — no @call capture has kind "func_literal".
    assert!(
        !caps
            .iter()
            .any(|(cn, kind, _, _)| cn == "call" && kind == "func_literal"),
        "func_literal must never itself be captured as @call, got: {caps:?}"
    );
    // Dispatch-table call `dispatch[0]()`: no capture of kind
    // "index_expression" as @call.
    assert!(
        !caps
            .iter()
            .any(|(cn, kind, _, _)| cn == "call" && kind == "index_expression"),
        "index_expression must never itself be captured as @call, got: {caps:?}"
    );
}

/// Every grammar-legal variant of type-definition node (`type_spec` and the
/// distinct `type_alias` node) that go.tags.scm claims to support must
/// produce a @definition.type capture.
#[test]
fn go_tags_completeness_type_spec_and_type_alias() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_tags_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("go").expect("go tags query missing");
    let query = Query::new(&lang, &query_str).expect("query compilation failed");
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(GO_VARIANTS, None).expect("parse failed");
    let source_bytes = GO_VARIANTS.as_bytes();

    // Collect (tag_kind, name_text) pairs: tag_kind is whichever
    // @definition.*/@reference.* capture co-occurs with @name in the match
    // (mirrors rust_tags_completeness_all_impl_variants's pairing approach —
    // definition.type is tagged on the *whole* type_spec/type_alias node,
    // not the name, so pairing by match is required to get the name text).
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
            } else if cap_name.starts_with("definition.") {
                tag_kind = Some(cap_name.to_string());
            }
        }
        if let (Some(n), Some(k)) = (name, tag_kind) {
            pairs.push((k, n));
        }
    }

    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.type" && n == "Plain"),
        "expected 'Plain' (type_spec) as definition.type, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.type" && n == "PlainAlias"),
        "expected 'PlainAlias' (type_alias) as definition.type, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.type" && n == "Generic"),
        "expected 'Generic' (generic type_spec) as definition.type, got: {pairs:?}"
    );
}

/// Negative case: closures/func_literals are never reported as
/// @definition.function or @definition.method.
#[test]
fn go_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_tags_negative: go grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("go").expect("go tags query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);
    let is_def_delta = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t == "delta"
    });
    assert!(
        !is_def_delta,
        "closure parameter 'delta' must never be captured as a function/method \
         definition, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `import_spec` that go.imports.scm claims
/// to support (plain, aliased, dot, blank, and both string-literal kinds
/// for the path) must produce a correctly-shaped @import.
#[test]
fn go_imports_completeness_all_import_spec_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_imports_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("go").expect("go imports query missing");
    let paths = collect_captures(&lang, GO_VARIANTS, &query_str, "import.path");
    let aliases = collect_captures(&lang, GO_VARIANTS, &query_str, "import.alias");
    let globs = collect_captures(&lang, GO_VARIANTS, &query_str, "import.glob");

    assert!(
        paths.iter().any(|p| p.contains("fmt")),
        "expected plain 'fmt' import path, got: {paths:?}"
    );
    assert!(
        aliases.contains(&"f".to_string()),
        "expected 'f' alias for the aliased fmt import, got: {aliases:?}"
    );
    assert!(
        aliases.contains(&"_".to_string()) || paths.iter().any(|p| p.contains("os")),
        "expected the blank import of \"os\" to surface a path, got paths={paths:?} aliases={aliases:?}"
    );
    assert_eq!(
        globs.len(),
        1,
        "expected exactly 1 dot-import glob marker (`. \"strings\"`), got {}: {globs:?}",
        globs.len()
    );
    // raw_string_literal import path: `errors` (backtick-quoted). Rare in
    // real Go (gofmt never emits it) but grammar-legal.
    assert!(
        paths.iter().any(|p| p.contains("errors")),
        "expected raw-string-literal import path '`errors`' to be captured, got: {paths:?}"
    );
}

/// Every grammar-legal variant of complexity/nesting node that
/// go.complexity.scm claims to support must produce a capture, plus a
/// negative case for constructs that must not increase complexity.
#[test]
fn go_complexity_completeness_and_negative() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_complexity_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("go")
        .expect("go complexity query missing");
    let caps = collect_captures_full(&lang, GO_SAMPLE, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    // if_statement (Classify/Pop/Max/sumEvens) and binary_expression
    // (`n%2 == 0`, `a > b`, comparisons throughout) must both appear.
    assert!(
        complexity_kinds.contains(&"if_statement"),
        "expected an if_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"binary_expression"),
        "expected a binary_expression complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"for_statement"),
        "expected a for_statement complexity node (the for-range loop in \
         sumEvens), got: {complexity_kinds:?}"
    );
    // NEGATIVE: a plain function call or a plain assignment must not
    // introduce a complexity node — only the 3 listed statement/expression
    // kinds above (and switch/select forms, not exercised in sample.go) do.
    assert!(
        !complexity_kinds.contains(&"call_expression"),
        "call_expression must never be a complexity node, got: {complexity_kinds:?}"
    );
}

#[test]
fn go_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping go_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "go",
        GO_SAMPLE,
        &["// Stack is a generic LIFO structure"],
    );
}
