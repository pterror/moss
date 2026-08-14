//! Query fixture tests for lua.
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

const LUA_SAMPLE: &str = include_str!("fixtures/lua/sample.lua");

#[test]
fn lua_tags_finds_functions_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_tags: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lua").expect("lua tags query missing");
    let pairs = collect_tag_pairs(&lang, LUA_SAMPLE, &query_str);
    // function Stack.new() — name: dot_index_expression -> @definition.method
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "new"),
        "expected 'new' (dot-index function) as definition.method, got: {pairs:?}"
    );
    // function Stack:push(value) — name: method_index_expression -> @definition.method
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "push"),
        "expected 'push' from 'function Stack:push()' as definition.method, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "is_empty"),
        "expected 'is_empty' from 'function Stack:is_empty()' as definition.method, \
         got: {pairs:?}"
    );
    // function classify(n) — name: identifier -> @definition.function
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "classify"),
        "expected 'classify' as definition.function, got: {pairs:?}"
    );
    // local function dispatch(event, ...) — name: identifier -> @definition.function
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "dispatch"),
        "expected 'dispatch' as definition.function, got: {pairs:?}"
    );
    // handlers.on_push = function(item) ... end — assignment-based dot-index
    // definition -> @definition.method
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "on_push"),
        "expected 'on_push' (dot-index assignment) as definition.method, got: {pairs:?}"
    );
    // handlers["on_pop"] = function() ... end — bracket-index assignment target
    // has no static name and must NOT appear as a definition of any kind.
    assert!(
        !pairs.iter().any(|(_, n)| n == "on_pop"),
        "bracket-index assignment target 'on_pop' has no static name and must not \
         be captured as a definition, got: {pairs:?}"
    );
}

#[test]
fn lua_calls_finds_call_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_calls: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lua").expect("lua calls query missing");
    let calls = collect_captures(&lang, LUA_SAMPLE, &query_str, "call");

    // Stack.new() — plain identifier call via dot-index qualifier (field: identifier)
    assert!(
        calls.iter().any(|c| c == "new"),
        "expected 'new' call in lua sample, got: {calls:?}"
    );
    // s:push(1) — method call
    assert!(
        calls.iter().any(|c| c == "push"),
        "expected 'push' call in lua sample, got: {calls:?}"
    );
    // print(classify(-3)) — plain identifier call
    assert!(
        calls.iter().any(|c| c == "classify"),
        "expected 'classify' call in lua sample, got: {calls:?}"
    );
    // handlers["on_pop"]() — computed/bracket call (dispatch-table idiom)
    assert!(
        calls.iter().any(|c| c.contains("on_pop")),
        "expected computed bracket call on 'on_pop' in lua sample, got: {calls:?}"
    );
}

#[test]
fn lua_imports_finds_require_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_imports: lua grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("lua")
        .expect("lua imports query missing");
    let paths = collect_captures(&lang, LUA_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("json")),
        "expected 'json' require path in lua sample, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("utils.string")),
        "expected 'utils.string' require path in lua sample, got: {paths:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.lua) -

const LUA_VARIANTS: &str = include_str!("fixtures/lua/variants.lua");

/// Every grammar-legal variant of `function_call.name` that lua.calls.scm
/// claims to support (identifier, method_index_expression,
/// dot_index_expression, bracket_index_expression, parenthesized_expression,
/// function_call) must actually match, with the right capture kind.
#[test]
fn lua_calls_completeness_all_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_calls_completeness: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lua").expect("lua calls query missing");
    let caps = collect_captures_full(&lang, LUA_VARIANTS, &query_str);

    // (capture_name, kind, text) triples, one per documented name-field variant.
    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "plain_global"),     // name: identifier
        ("call", "identifier", "method_fn"),        // name: method_index_expression
        ("call", "identifier", "dotted_fn"),        // name: dot_index_expression
        ("call", "function_call", "get_handler()"), // name: function_call (chained)
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in lua.calls.scm \
             output for variants.lua, got: {caps:?}"
        );
    }

    // bracket_index_expression callees: whole-node text captured as best-effort @call.
    let bracket_calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "call" && k == "bracket_index_expression")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        bracket_calls.contains(&"dispatch[\"key\"]"),
        "expected bracket_index_expression call 'dispatch[\"key\"]', got: {bracket_calls:?}"
    );
    assert!(
        bracket_calls.contains(&"dispatch[1]"),
        "expected bracket_index_expression call 'dispatch[1]', got: {bracket_calls:?}"
    );

    // parenthesized_expression callee (IIFE-style call target).
    let paren_calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "call" && k == "parenthesized_expression")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        paren_calls.contains(&"(plain_global)"),
        "expected parenthesized_expression call '(plain_global)', got: {paren_calls:?}"
    );

    // @call.qualifier must carry the receiver/table text for
    // method/dot/bracket calls, never the call name itself.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"NS"),
        "expected 'NS' qualifier for method/dot calls, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"dispatch"),
        "expected 'dispatch' qualifier for bracket calls, got: {qualifiers:?}"
    );
}

/// Every grammar-legal variant of `function_declaration.name` (identifier,
/// dot_index_expression, method_index_expression) plus the assignment-based
/// definition forms (identifier, dot_index_expression) must produce a @name
/// capture with the correct definition kind; the dynamic
/// bracket_index_expression assignment target must NOT.
#[test]
fn lua_tags_completeness_all_definition_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_tags_completeness: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lua").expect("lua tags query missing");
    let pairs = collect_tag_pairs(&lang, LUA_VARIANTS, &query_str);

    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "plain_global"),
        "expected 'plain_global' (function_declaration, name: identifier) as \
         definition.function, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "plain_local"),
        "expected 'plain_local' (local function, name: identifier) as \
         definition.function, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "dotted_fn"),
        "expected 'dotted_fn' (function_declaration, name: dot_index_expression) as \
         definition.method, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "method_fn"),
        "expected 'method_fn' (function_declaration, name: method_index_expression) as \
         definition.method, got: {pairs:?}"
    );
    // Bug fix: assignment-based function expression with a bare identifier
    // target was previously silently dropped (only the dot_index_expression
    // target was handled).
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "ident_fn"),
        "expected 'ident_fn' (assignment_statement, variable_list.name: identifier) as \
         definition.function, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "dotted_assign_fn"),
        "expected 'dotted_assign_fn' (assignment_statement, variable_list.name: \
         dot_index_expression) as definition.method, got: {pairs:?}"
    );
    // NEGATIVE: dynamic bracket-index assignment target has no static name.
    assert!(
        !pairs.iter().any(|(_, n)| n == "dynamic_key"),
        "bracket-index assignment target must not produce a definition, got: {pairs:?}"
    );
}

/// Bug fix: lua.tags.scm never had @reference.call at all (lua.calls.scm
/// handled call extraction separately, but tags never mirrored it) — the same
/// class of gap documented as bug #5 in docs/query-testing-methodology.md's
/// Rust worked example ("scoped calls existed in rust.calls.scm but were
/// never ported to rust.tags.scm").
#[test]
fn lua_tags_finds_call_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_tags_call_refs: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_tags_call_refs: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lua").expect("lua tags query missing");
    let pairs = collect_tag_pairs(&lang, LUA_VARIANTS, &query_str);

    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "plain_global"),
        "expected 'plain_global' call as reference.call, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "method_fn"),
        "expected 'method_fn' call as reference.call, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "dotted_fn"),
        "expected 'dotted_fn' call as reference.call, got: {pairs:?}"
    );
    // Bracket-dispatched calls report the subscripted container's name as a
    // best-effort approximation (matches python.tags.scm's convention).
    let dispatch_refs = pairs
        .iter()
        .filter(|(k, n)| k == "reference.call" && n == "dispatch")
        .count();
    assert!(
        dispatch_refs >= 2,
        "expected at least 2 'dispatch' reference.call entries (dispatch[\"key\"](), \
         dispatch[1]()), got {dispatch_refs}: {pairs:?}"
    );
}

/// Negative cases: constructs that must never appear in @call/@name captures.
#[test]
fn lua_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_calls_negative: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lua").expect("lua calls query missing");
    let caps = collect_captures_full(&lang, LUA_VARIANTS, &query_str);
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

    // Only the call site `adder(1)(2)` should register "adder" as a call —
    // its definition site (the `local adder = function(x) ... end` LHS) must
    // not.
    let adder_calls = call_texts.iter().filter(|t| **t == "adder").count();
    assert_eq!(
        adder_calls, 1,
        "expected exactly 1 call to 'adder' (the call site, not the definition), \
         got {adder_calls}: {call_texts:?}"
    );
}

/// Negative case: a plain variable reference passed to something other than
/// `require(...)` must never be captured as an import path.
#[test]
fn lua_imports_negative_case_non_require_not_matched() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_imports_negative: lua grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("lua")
        .expect("lua imports query missing");
    let paths = collect_captures(&lang, LUA_VARIANTS, &query_str, "import.path");
    // All three require() forms (paren, bareword, long-bracket string) must match.
    assert!(
        paths.iter().any(|p| p.contains("paren_module")),
        "expected 'paren_module' require path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("bareword_module")),
        "expected 'bareword_module' (bareword require, no parens) path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("bracket_module")),
        "expected 'bracket_module' (long-bracket string require) path, got: {paths:?}"
    );
    // `local dyn = some_module_var` must never contribute an import path.
    assert!(
        !paths.iter().any(|p| p.contains("module_var")),
        "non-require variable reference must not be captured as an import, got: {paths:?}"
    );
}

/// lua.complexity.scm: every branch/loop construct plus `and`/`or` must
/// contribute to @complexity; only branch/loop constructs (not `and`/`or`)
/// contribute to @nesting.
#[test]
fn lua_complexity_finds_all_branch_and_loop_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_complexity: lua grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("lua")
        .expect("lua complexity query missing");
    let caps = collect_captures_full(&lang, LUA_VARIANTS, &query_str);

    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    for kind in [
        "if_statement",
        "elseif_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
        "and",
        "or",
    ] {
        assert!(
            complexity_kinds.contains(&kind),
            "expected a @complexity capture of kind '{kind}' in variants.lua, \
             got kinds: {complexity_kinds:?}"
        );
    }
    // for_statement must fire for both the numeric and generic clause forms.
    let for_count = complexity_kinds
        .iter()
        .filter(|k| **k == "for_statement")
        .count();
    assert!(
        for_count >= 2,
        "expected at least 2 for_statement @complexity captures (numeric + generic \
         clause), got {for_count}"
    );

    let nesting_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "nesting")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    for kind in [
        "if_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
    ] {
        assert!(
            nesting_kinds.contains(&kind),
            "expected a @nesting capture of kind '{kind}' in variants.lua, got kinds: \
             {nesting_kinds:?}"
        );
    }
    // "and"/"or" contribute to @complexity but never to @nesting.
    assert!(
        !nesting_kinds.contains(&"and") && !nesting_kinds.contains(&"or"),
        "'and'/'or' must never contribute to @nesting, got kinds: {nesting_kinds:?}"
    );
}

#[test]
fn lua_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping lua_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "lua",
        LUA_SAMPLE,
        &["-- Simple stack implementation"],
    );
}
