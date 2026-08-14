//! Query fixture tests for jinja2.
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
// Jinja2 (live grammar — uses ~/.config/normalize/grammars/jinja2.so)
// ---------------------------------------------------------------------------

const JINJA2_SAMPLE: &str = include_str!("fixtures/jinja2/sample.jinja2");

#[test]
fn jinja2_tags_finds_macros() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jinja2_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_tags: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("jinja2")
        .expect("jinja2 tags query missing");
    let names = collect_captures(&lang, JINJA2_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"render_form".to_string()),
        "expected 'render_form' macro in jinja2 tags, got: {names:?}"
    );
    assert!(
        names.contains(&"render_nav".to_string()),
        "expected 'render_nav' macro in jinja2 tags, got: {names:?}"
    );
}

#[test]
fn jinja2_imports_finds_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jinja2_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_imports: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("jinja2")
        .expect("jinja2 imports query missing");
    let paths = collect_captures(&lang, JINJA2_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("base.html")),
        "expected 'base.html' in jinja2 import paths, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("helpers.html")),
        "expected 'helpers.html' in jinja2 import paths, got: {paths:?}"
    );
}

#[test]
fn jinja2_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jinja2_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_complexity: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("jinja2")
        .expect("jinja2 complexity query missing");
    let complexity = collect_captures(&lang, JINJA2_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in jinja2 sample (for/if/elif), got {} ({complexity:?})",
        complexity.len()
    );
}

// ==================== jinja2 query tests ====================
// Query-completeness sweep per docs/query-testing-methodology.md.
// Covers: jinja2.{tags,calls,complexity,imports,cfg}.scm field-constraint
// completeness against grammars/jinja2/src/node-types.json, verified via
// `normalize syntax ast` / `normalize syntax query` (not from memory).

const JINJA2_VARIANTS: &str = include_str!("fixtures/jinja2/variants.jinja2");

#[test]
fn jinja2_sample_finds_real_world_idioms() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!(
            "Skipping jinja2_sample_finds_real_world_idioms: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_sample_finds_real_world_idioms: jinja2 grammar .so not found");
        return;
    };

    // Macro with multiple defaulted parameters, macro caller-block
    // ({% call %}), filter chains ({% filter %} and value|filter|filter),
    // for/else, "if not loop.last", "and"/"or" in conditions, ternary
    // expression, set-with-filter, dynamic default() filter call.
    let calls_query = loader
        .get_calls("jinja2")
        .expect("jinja2 calls query missing");
    let calls = collect_captures(&lang, JINJA2_SAMPLE, &calls_query, "call");
    assert!(
        calls.contains(&"default".to_string()),
        "expected 'default' filter call in jinja2 sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"sort".to_string()),
        "expected 'sort' filter call (items|sort) in jinja2 sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"upper".to_string()),
        "expected 'upper' filter call ({{% filter upper %}}) in jinja2 sample, got: {calls:?}"
    );

    let imports_query = loader
        .get_imports("jinja2")
        .expect("jinja2 imports query missing");
    let paths = collect_captures(&lang, JINJA2_SAMPLE, &imports_query, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("optional.html")),
        "expected 'optional.html' (include ... ignore missing) in jinja2 sample imports, got: {paths:?}"
    );

    let complexity_query = loader
        .get_complexity("jinja2")
        .expect("jinja2 complexity query missing");
    let complexity = collect_captures(&lang, JINJA2_SAMPLE, &complexity_query, "complexity");
    assert!(
        complexity.len() >= 6,
        "expected several complexity nodes (for/if/elif/and/or/ternary) in jinja2 sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn jinja2_calls_completeness_all_call_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping jinja2_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_calls_completeness: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("jinja2")
        .expect("jinja2 calls query missing");
    let captures = collect_captures_full(&lang, JINJA2_VARIANTS, &query_str);

    // function: (identifier) -- plain call
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_plain_func"),
        "expected plain function call, got: {captures:?}"
    );
    // function: (attribute_expression) -- single-hop method call
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_method"),
        "expected method call, got: {captures:?}"
    );
    // function: (attribute_expression) nested -- chained attribute access
    // before the call; object: (_) is unconstrained so the whole chain's
    // qualifier must come through as call.qualifier regardless of depth.
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "call.qualifier"
                && kind == "attribute_expression"
                && text == "call_a.call_b"),
        "expected chained attribute qualifier 'call_a.call_b', got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "call" && kind == "identifier" && text == "call_c"),
        "expected chained attribute call 'call_c', got: {captures:?}"
    );
    // filter_item.name, reached via filter_expression's filter_chain
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_filter_name"),
        "expected filter call, got: {captures:?}"
    );
    // filter_item.name, reached via filter_block_statement's filter_chain
    // (structurally distinct parent from filter_expression -- no parent
    // constraint in the query, so both must match identically)
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_filter_via_block"),
        "expected {{% filter %}}-block filter call, got: {captures:?}"
    );
    // filter_item.name, reached via set_block_statement's filter_chain
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_filter_via_set"),
        "expected {{% set ... | filter %}}-block filter call, got: {captures:?}"
    );
    // test_expression.test
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_test_name"),
        "expected test call, got: {captures:?}"
    );
    // call_statement.callee -- call_expression wrapping identifier
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_stmt_macro"),
        "expected {{% call %}}-statement macro call, got: {captures:?}"
    );
    // call_statement.callee -- call_expression wrapping attribute_expression
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "call.qualifier"
                && kind == "identifier"
                && text == "call_stmt_obj"),
        "expected {{% call %}}-statement method qualifier, got: {captures:?}"
    );
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "identifier"
            && text == "call_stmt_method"),
        "expected {{% call %}}-statement method call, got: {captures:?}"
    );

    // Negative: bare attribute access (no call_expression) must not appear
    // as a @call capture.
    assert!(
        !captures
            .iter()
            .any(|(cap, _, text, _)| cap == "call" && text.contains("neg_attr")),
        "bare attribute access must not match @call, got: {captures:?}"
    );
    // Negative: a bare identifier reference must not appear as a @call.
    assert!(
        !captures
            .iter()
            .any(|(cap, _, text, _)| cap == "call" && text == "neg_bare_identifier"),
        "bare identifier reference must not match @call, got: {captures:?}"
    );
}

#[test]
fn jinja2_imports_completeness_dynamic_paths() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping jinja2_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_imports_completeness: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("jinja2")
        .expect("jinja2 imports query missing");
    let captures = collect_captures_full(&lang, JINJA2_VARIANTS, &query_str);

    // BUG FIXED: path: (string) alone silently dropped every dynamic
    // template path. All four statement types must now capture non-string
    // path expressions with the correct node kind.
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "import.path"
                && kind == "string"
                && text.contains("variant_base.html")),
        "expected string extends path, got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "import.path"
                && kind == "identifier"
                && text == "variant_include_var"),
        "expected identifier (dynamic) include path, got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(cap, kind, _, _)| cap == "import.path" && kind == "concat_expression"),
        "expected concat_expression include path, got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "import.path"
                && kind == "identifier"
                && text == "variant_module_var"),
        "expected identifier path on 'from' statement, got: {captures:?}"
    );

    // Negative: a plain string expression outside an
    // extends/import/from/include statement must not match @import.path.
    assert!(
        !captures
            .iter()
            .any(|(cap, _, text, _)| cap == "import.path" && text.contains("not_a_template_path")),
        "plain string expression must not match @import.path, got: {captures:?}"
    );
}

#[test]
fn jinja2_complexity_completeness_boolean_and_ternary() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!(
            "Skipping jinja2_complexity_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_complexity_completeness: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("jinja2")
        .expect("jinja2 complexity query missing");
    let captures = collect_captures_full(&lang, JINJA2_VARIANTS, &query_str);

    for (kind, text) in [
        ("and_expression", "variant_a and variant_b"),
        ("or_expression", "variant_a or variant_b"),
        (
            "ternary_expression",
            "variant_a if variant_cond_a else variant_b",
        ),
    ] {
        assert!(
            captures
                .iter()
                .any(|(cap, k, t, _)| cap == "complexity" && k == kind && t == text),
            "expected {kind} to produce @complexity, got: {captures:?}"
        );
    }

    // Negative: a plain identifier with no boolean/ternary operator must
    // not itself add a complexity point.
    assert!(
        !captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "complexity"
                && kind == "identifier"
                && text == "neg_plain_boolean"),
        "bare identifier must not match @complexity, got: {captures:?}"
    );
}

#[test]
fn jinja2_cfg_completeness_for_loop_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping jinja2_cfg_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_cfg_completeness: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader.get_cfg("jinja2").expect("jinja2 cfg query missing");
    let captures = collect_captures_full(&lang, JINJA2_VARIANTS, &query_str);

    // BUG FIXED: the old pattern positionally captured the loop target and
    // the iterable as two `@cfg.loop.condition` captures and never captured
    // `@cfg.loop.body` at all; it also only worked when the iterable was a
    // bare identifier. Non-identifier iterables (filter chains, calls) must
    // now produce a correctly-kinded @cfg.loop.condition.
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "cfg.loop.condition"
                && kind == "filter_expression"
                && text == "variant_items|sort"),
        "expected filter_expression iterable to produce @cfg.loop.condition, got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "cfg.loop.condition"
                && kind == "call_expression"
                && text == "variant_get_items()"),
        "expected call_expression iterable to produce @cfg.loop.condition, got: {captures:?}"
    );
    // Tuple-unpacking target must still parse and yield a @cfg.loop with a
    // plain-identifier iterable (`target` isn't captured by this query at
    // all -- only `iterable`/`body` are -- so this just confirms the tuple
    // target doesn't break the surrounding match).
    assert!(
        captures.iter().any(|(cap, kind, text, _)| {
            cap == "cfg.loop.condition"
                && text == "variant_pairs.items()"
                && (kind == "attribute_expression" || kind == "call_expression")
        }),
        "expected iterable of the tuple-target loop to produce @cfg.loop.condition, got: {captures:?}"
    );

    // BUG FIXED: `body: (_)` alone drops the whole @cfg.loop match for an
    // empty loop body. The field-absence fallback (`!body`) must still
    // produce a @cfg.loop with @cfg.loop.condition and no @cfg.loop.body.
    let loop_matches_without_body = captures
        .iter()
        .filter(|(cap, _, text, _)| {
            cap == "cfg.loop"
                && text.starts_with("{% for variant_x in variant_items %}{% endfor %}")
        })
        .count();
    assert!(
        loop_matches_without_body >= 1,
        "expected empty-body for-loop to still produce @cfg.loop, got: {captures:?}"
    );
    assert!(
        !captures
            .iter()
            .any(|(cap, _, text, _)| cap == "cfg.loop.body" && text.is_empty()),
        "empty-body loop must not produce a spurious empty @cfg.loop.body capture, got: {captures:?}"
    );

    // Non-empty loops must produce a real @cfg.loop.body.
    assert!(
        captures
            .iter()
            .any(|(cap, _, text, _)| cap == "cfg.loop.body" && text.contains("{{ variant_x }}")),
        "expected non-empty for-loop to produce @cfg.loop.body, got: {captures:?}"
    );
}

#[test]
fn jinja2_cfg_completeness_if_branch_variants() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping jinja2_cfg_completeness_if: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_cfg_completeness_if: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader.get_cfg("jinja2").expect("jinja2 cfg query missing");
    let captures = collect_captures_full(&lang, JINJA2_VARIANTS, &query_str);

    // elif_clause as @cfg.branch.else
    assert!(
        captures
            .iter()
            .any(|(cap, kind, _, _)| cap == "cfg.branch.else" && kind == "elif_clause"),
        "expected elif_clause to produce @cfg.branch.else, got: {captures:?}"
    );
    // else_clause as @cfg.branch.else
    assert!(
        captures
            .iter()
            .any(|(cap, kind, _, _)| cap == "cfg.branch.else" && kind == "else_clause"),
        "expected else_clause to produce @cfg.branch.else, got: {captures:?}"
    );
    // Bare if with neither elif nor else must still produce @cfg.branch
    // (the anchored "condition is last child" pattern), but with no
    // @cfg.branch.else in that specific match.
    let bare_if_matches: Vec<_> = captures
        .iter()
        .filter(|(cap, _, text, _)| {
            cap == "cfg.branch" && text.starts_with("{% if neg_only_condition %}")
        })
        .collect();
    assert!(
        !bare_if_matches.is_empty(),
        "expected bare if (no elif/else) to produce @cfg.branch, got: {captures:?}"
    );
}
