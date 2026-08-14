//! Query fixture tests for elixir.
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
// Elixir
// ---------------------------------------------------------------------------

const ELIXIR_SAMPLE: &str = include_str!("fixtures/elixir/sample.ex");

const ELIXIR_VARIANTS: &str = include_str!("fixtures/elixir/variants.ex");

// --- Dimension 4: real-world fixture coverage (sample.ex) -------------------

#[test]
fn elixir_tags_finds_modules_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    let names = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in elixir tags, got: {names:?}"
    );
    assert!(
        names.contains(&"push".to_string()) || names.contains(&"pop".to_string()),
        "expected 'push' or 'pop' in elixir tags, got: {names:?}"
    );
    // Guard clauses: `def double(n) when is_integer(n) or is_float(n)` — the
    // gap fixed on top of the shallow baseline. Previously silently dropped
    // any guarded function head entirely.
    assert!(
        names.contains(&"double".to_string()),
        "expected 'double' (guarded function head) in elixir tags, got: {names:?}"
    );
    // defguard
    assert!(
        names.contains(&"is_percentage".to_string()),
        "expected 'is_percentage' (defguard) in elixir tags, got: {names:?}"
    );
    // defp with guard
    assert!(
        names.contains(&"clamp".to_string()),
        "expected 'clamp' (guarded defp) in elixir tags, got: {names:?}"
    );
    // defprotocol / defimpl
    assert!(
        names.contains(&"Sized".to_string()),
        "expected 'Sized' protocol in elixir tags, got: {names:?}"
    );
    // Nested module name is a single dotted alias token
    assert!(
        names.contains(&"Stack.Namespaced".to_string()),
        "expected 'Stack.Namespaced' nested module in elixir tags, got: {names:?}"
    );
    // defmacro / defmacrop
    assert!(
        names.contains(&"trace".to_string()),
        "expected 'trace' defmacro in elixir tags, got: {names:?}"
    );
    assert!(
        names.contains(&"double_expr".to_string()),
        "expected 'double_expr' guarded defmacrop in elixir tags, got: {names:?}"
    );
}

#[test]
fn elixir_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_calls: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let calls = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"defmodule".to_string()) || calls.contains(&"def".to_string()),
        "expected 'defmodule' or 'def' call in elixir sample, got: {calls:?}"
    );
    // Anonymous-function invocation `predicate.(x)` — the gap fixed on top
    // of the shallow baseline (dot with no `right` field).
    assert!(
        calls.contains(&"predicate".to_string()),
        "expected 'predicate' anon-fn invocation call in elixir sample, got: {calls:?}"
    );
}

#[test]
fn elixir_imports_finds_alias_and_import() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_imports: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elixir")
        .expect("elixir imports query missing");
    let paths = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Enum")),
        "expected 'Enum' in elixir import paths, got: {paths:?}"
    );
    // Multi-alias form `alias Stack.{Namespaced}` — the gap fixed on top of
    // the shallow baseline (dot with a tuple right-hand side).
    assert!(
        paths.iter().any(|p| p.contains("Namespaced")),
        "expected 'Namespaced' from multi-alias 'alias Stack.{{Namespaced}}', got: {paths:?}"
    );
}

#[test]
fn elixir_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_complexity: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elixir")
        .expect("elixir complexity query missing");
    let complexity = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in elixir sample, got {} ({complexity:?})",
        complexity.len()
    );
    // The previous blanket `(call) @complexity` counted every function call
    // (including ordinary calls like `Enum.reduce`, `IO.inspect`) as a
    // decision point. With the fix, plain non-branching calls must NOT
    // contribute — only the scoped set of branching macros / stab_clause
    // arms / boolean operators listed in elixir.complexity.scm should.
    // `sum_evens/1`'s body is a single non-branching call with no control
    // flow at all, so the *total* complexity count for the whole sample
    // must be far smaller than "one point per call", which the previous
    // version produced (every `def`/`defmodule`/ordinary call counted).
    let total_calls_in_sample = ELIXIR_SAMPLE.matches('(').count();
    assert!(
        complexity.len() < total_calls_in_sample,
        "expected complexity count ({}) to be well below the raw call count \
         ({total_calls_in_sample}) now that ordinary calls are excluded",
        complexity.len()
    );
}

#[test]
fn elixir_types_finds_module_aliases() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_types: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("elixir")
        .expect("elixir types query missing");
    let refs = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r.contains("Enum") || r.contains("Stack") || r.contains("MathUtils")),
        "expected module alias references in elixir sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.ex) -

/// Every grammar-legal variant of def/defp/defmacro/defmacrop/defguard/
/// defguardp/defdelegate name extraction that elixir.tags.scm claims to
/// support, with the correct definition kind (dimension 3).
#[test]
fn elixir_tags_completeness_def_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    let pairs = collect_tag_pairs(&lang, ELIXIR_VARIANTS, &query_str);

    let function_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.function")
        .map(|(_, n)| n.as_str())
        .collect();
    let macro_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.macro")
        .map(|(_, n)| n.as_str())
        .collect();
    let module_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.module")
        .map(|(_, n)| n.as_str())
        .collect();
    let interface_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.interface")
        .map(|(_, n)| n.as_str())
        .collect();
    let impl_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.implementation")
        .map(|(_, n)| n.as_str())
        .collect();

    for expected in [
        "plain_call",
        "plain_noargs",
        "guarded_call",
        "guarded_noargs",
        "private_plain",
        "private_guarded",
        "guard_expr",
        "guardp_expr",
        "delegated_call",
        "delegated_noargs",
    ] {
        assert!(
            function_names.contains(&expected),
            "expected '{expected}' in elixir @definition.function, got: {function_names:?}"
        );
    }
    for expected in [
        "macro_plain",
        "macro_guarded",
        "macrop_plain",
        "macrop_guarded",
    ] {
        assert!(
            macro_names.contains(&expected),
            "expected '{expected}' in elixir @definition.macro, got: {macro_names:?}"
        );
    }
    assert!(
        module_names.contains(&"Plain") && module_names.contains(&"Deep.Nested"),
        "expected 'Plain' and 'Deep.Nested' modules, got: {module_names:?}"
    );
    assert!(
        interface_names.contains(&"PlainProtocol"),
        "expected 'PlainProtocol' defprotocol, got: {interface_names:?}"
    );
    assert!(
        impl_names.contains(&"PlainProtocol"),
        "expected 'PlainProtocol' defimpl reference, got: {impl_names:?}"
    );
}

/// Every grammar-legal variant of call.target (identifier, dot-with-right,
/// dot-without-right, call) that elixir.calls.scm claims to support.
#[test]
fn elixir_calls_completeness_target_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_calls_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let caps = collect_captures_full(&lang, ELIXIR_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // target: identifier (local call)
    assert!(
        calls.contains(&"identity"),
        "expected 'identity' local call, got: {calls:?}"
    );
    // target: dot, right: identifier (remote call)
    assert!(
        calls.contains(&"identity")
            && caps
                .iter()
                .any(|(cn, _, t, _)| cn == "call.qualifier" && t == "Kernel"),
        "expected 'Kernel.identity' remote call with qualifier, got: {caps:?}"
    );
    // target: dot, no right (anonymous-function invocation)
    assert!(
        calls.contains(&"add_one"),
        "expected 'add_one' anon-fn invocation ('add_one.(5)'), got: {calls:?}"
    );

    // target: call (dynamic/macro-generated call, e.g. `unquote(x)(1, 2)`
    // inside a `quote` block) — best-effort partial capture of the inner
    // call's own text.
    let query_str2 = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let dyn_source = "defmodule M do\n  defmacro build(name) do\n    quote do\n      unquote(name)(1, 2)\n    end\n  end\nend\n";
    let dyn_caps = collect_captures_full(&lang, dyn_source, &query_str2);
    assert!(
        dyn_caps
            .iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "call" && t == "unquote(name)"),
        "expected dynamic call-target 'unquote(name)' captured as @call, got: {dyn_caps:?}"
    );
}

/// Negative cases: constructs that must never appear (or must appear exactly
/// once, not duplicated) in @call captures.
#[test]
fn elixir_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_calls_negative: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let caps = collect_captures_full(&lang, ELIXIR_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder2.other_field` IS a remote call (target: dot, right: identifier
    // "other_field"); it must appear exactly once — the negative guard here
    // is against the anon-invocation `!right` pattern ALSO firing on it (it
    // has a `right`, so it must not double-match).
    let field_calls = call_texts.iter().filter(|t| **t == "other_field").count();
    assert_eq!(
        field_calls, 1,
        "expected exactly 1 'other_field' call (holder2.other_field), got {field_calls}: {call_texts:?}"
    );

    // A bare local-variable read (`bound = plain_arithmetic`) must never be
    // captured as a call.
    assert!(
        !call_texts.contains(&"bound"),
        "local variable 'bound' must never be captured as a call, got: {call_texts:?}"
    );
}

/// Every grammar-legal variant of alias/import/use/require argument shape
/// (plain alias, multi-alias tuple, dot-qualified) that elixir.imports.scm
/// claims to support.
#[test]
fn elixir_imports_completeness_directive_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_imports_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elixir")
        .expect("elixir imports query missing");
    let paths = collect_captures(&lang, ELIXIR_VARIANTS, &query_str, "import.path");

    // Plain forms
    assert!(
        paths.contains(&"Plain".to_string()),
        "expected plain 'alias Plain', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Deep.Nested".to_string()),
        "expected 'alias Deep.Nested, as: DN', got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("Kernel")),
        "expected 'import Kernel, only: [...]', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Logger".to_string()),
        "expected 'require Logger', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Application".to_string()),
        "expected 'use Application', got: {paths:?}"
    );
    // Multi-alias tuple form: `alias Deep.{Nested}`
    assert!(
        paths.iter().filter(|p| *p == "Nested").count() >= 1,
        "expected 'Nested' from multi-alias 'alias Deep.{{Nested}}', got: {paths:?}"
    );
    // Dot-qualified single form: `alias __MODULE__.Plain`
    assert!(
        paths.iter().any(|p| p == "Plain"),
        "expected 'Plain' from dot-qualified 'alias __MODULE__.Plain', got: {paths:?}"
    );
}

/// Every grammar-legal variant of branching complexity node that
/// elixir.complexity.scm claims to support, plus the correctness fix:
/// ordinary (non-branching) calls and arithmetic operators must NOT count.
#[test]
fn elixir_complexity_completeness_branch_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping elixir_complexity_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_complexity_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elixir")
        .expect("elixir complexity query missing");
    let caps = collect_captures_full(&lang, ELIXIR_VARIANTS, &query_str);
    let complexity_kinds: Vec<&(String, String, String, usize)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .collect();
    let complexity_call_kws: Vec<&str> = complexity_kinds
        .iter()
        .filter(|(_, k, _, _)| k == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    for kw in [
        "if x > 0",
        "unless x > 0",
        "case x",
        "cond do",
        "with {:ok",
        "for x <- list",
        "try do",
        "receive do",
    ] {
        let head = kw.split_whitespace().next().unwrap();
        assert!(
            complexity_call_kws.iter().any(|t| t.starts_with(head)),
            "expected a branching '{head}' call in @complexity, got: {complexity_call_kws:?}"
        );
    }

    // stab_clause arms count independently: branch_case has 3 arms.
    let stab_count = complexity_kinds
        .iter()
        .filter(|(_, k, _, _)| k == "stab_clause")
        .count();
    assert!(
        stab_count >= 3,
        "expected at least 3 'stab_clause' complexity nodes (branch_case alone has 3), got {stab_count}"
    );

    // Boolean operators count; arithmetic/comparison operators must not.
    let bool_ops: Vec<&str> = complexity_kinds
        .iter()
        .filter(|(_, k, _, _)| k == "binary_operator")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        bool_ops.iter().any(|t| t.contains("&&"))
            && bool_ops.iter().any(|t| t.contains(" and "))
            && bool_ops.iter().any(|t| t.contains(" or ")),
        "expected &&/and/or boolean-operator complexity nodes, got: {bool_ops:?}"
    );
}

/// Negative cases: ordinary (non-branching) calls and arithmetic/comparison
/// operators must never contribute to @complexity — the correctness bug
/// fixed on top of the previous blanket `(call) @complexity` / `(binary_
/// operator) @complexity`.
#[test]
fn elixir_complexity_negative_ordinary_calls_and_arithmetic_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_complexity_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_complexity_negative: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elixir")
        .expect("elixir complexity query missing");
    let source = "defmodule N do\n  def f(x) do\n    plain_arithmetic = 1 + 2 - 3 * 4 / 5\n    plain_comparison = 1 == 2\n    identity(x)\n    {plain_arithmetic, plain_comparison}\n  end\nend\n";
    let caps = collect_captures_full(&lang, source, &query_str);
    let complexity_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        complexity_texts.is_empty(),
        "expected 0 complexity nodes for a function with only ordinary calls and \
         arithmetic/comparison operators (no branching), got: {complexity_texts:?}"
    );
}

/// Negative case for tags: `defguard`'s always-guarded form must not also
/// spuriously produce a plain (unguarded) @definition.function match — the
/// unguarded def/defp patterns require `arguments -> call`/`identifier`
/// directly, which a guarded head's `binary_operator` wrapper never
/// satisfies, so no double-counting should occur.
#[test]
fn elixir_tags_negative_guarded_head_not_double_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags_negative: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    let names = collect_captures(&lang, ELIXIR_VARIANTS, &query_str, "name");
    let guarded_count = names.iter().filter(|n| *n == "guarded_call").count();
    assert_eq!(
        guarded_count, 1,
        "expected exactly 1 'guarded_call' definition (not double-counted \
         across guarded/unguarded patterns), got {guarded_count}: {names:?}"
    );
}

#[test]
fn elixir_tags_no_args_function() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags_no_args_function: elixir grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    // Test the no-args function form: "def name do ... end"
    let source = r#"defmodule Foo do
  def initialize do
    :ok
  end

  def greet(name) do
    name
  end
end"#;
    let names = collect_captures(&lang, source, &query_str, "name");
    assert!(
        names.contains(&"initialize".to_string()),
        "expected 'initialize' (no-args def), got: {names:?}"
    );
    assert!(
        names.contains(&"greet".to_string()),
        "expected 'greet' (with-args def), got: {names:?}"
    );
}

#[test]
fn elixir_decorations_finds_module_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elixir_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "elixir",
        ELIXIR_SAMPLE,
        &["@doc"],
    );
}
