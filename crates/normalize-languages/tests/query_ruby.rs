//! Query fixture tests for ruby.
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
// Ruby
// ---------------------------------------------------------------------------

const RUBY_SAMPLE: &str = include_str!("fixtures/ruby/sample.rb");

const RUBY_VARIANTS: &str = include_str!("fixtures/ruby/variants.rb");

// --- Dimension 4: real-world fixture coverage (sample.rb) -------------------

#[test]
fn ruby_tags_finds_class_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_tags: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ruby").expect("ruby tags query missing");
    let names = collect_captures(&lang, RUBY_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in ruby tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' method in ruby tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sum_if".to_string()),
        "expected 'sum_if' method in ruby tags, got: {names:?}"
    );
    // Mixin module + namespaced include: real Ruby leans heavily on modules
    // as mixins, frequently with namespaced module names (ActiveSupport::Concern-
    // style). The module itself and its own method must both be found.
    assert!(
        names.contains(&"Loggable".to_string()),
        "expected 'Loggable' module in ruby tags, got: {names:?}"
    );
    assert!(
        names.contains(&"log".to_string()),
        "expected 'log' method nested in module Loggable, got: {names:?}"
    );
    // Inheritance: BoundedStack < Stack.
    assert!(
        names.contains(&"BoundedStack".to_string()),
        "expected 'BoundedStack' class in ruby tags, got: {names:?}"
    );
    // Struct.new-based value class with a block body: the block's own method
    // definition ('distance') must still be found even though its container
    // is a constant assignment, not a `class` node.
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' method inside Struct.new block, got: {names:?}"
    );
    // `class << self; def empty; end; end` — the method inside a singleton-
    // class reopening must be found as a plain method definition (see
    // ruby.tags.scm's comment on why the singleton_class container itself
    // is not captured).
    assert!(
        names.contains(&"empty".to_string()),
        "expected 'empty' class method (via class << self) in ruby tags, got: {names:?}"
    );
}

#[test]
fn ruby_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_calls: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ruby").expect("ruby calls query missing");
    let calls = collect_captures(&lang, RUBY_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"push".to_string()) || calls.contains(&"pop".to_string()),
        "expected 'push' or 'pop' call in ruby sample, got: {calls:?}"
    );
    // Bare Kernel-style call whose callee is a constant, not an identifier:
    // Integer("5") — this is the gap fixed on top of the shallow baseline.
    assert!(
        calls.contains(&"Integer".to_string()),
        "expected 'Integer' bare-constant call in ruby sample, got: {calls:?}"
    );
    // Safe navigation: label&.upcase must still be found as an ordinary call.
    assert!(
        calls.contains(&"upcase".to_string()),
        "expected 'upcase' call via safe navigation in ruby sample, got: {calls:?}"
    );
    // `super()`/`super` (implicit-args form) inside BoundedStack — the plain
    // `super` keyword is a distinct node from `call` in this grammar and is
    // legitimately absent from @call; not asserted here (see ruby.calls.scm
    // for why explicit-operator/self/super call forms are out of scope).
}

#[test]
fn ruby_imports_finds_require() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_imports: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("ruby")
        .expect("ruby imports query missing");
    let paths = collect_captures(&lang, RUBY_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"json".to_string()),
        "expected 'json' in ruby import paths, got: {paths:?}"
    );
    // `require_relative 'support/helpers'`
    assert!(
        paths.iter().any(|p| p.contains("helpers")),
        "expected require_relative path in ruby import paths, got: {paths:?}"
    );
    // `include ActiveSupport::Concern` — namespaced include argument
    // (scope_resolution), the real-world-common case the bare-constant-only
    // pattern silently dropped.
    assert!(
        paths.iter().any(|p| p.contains("ActiveSupport")),
        "expected namespaced 'include ActiveSupport::Concern' in ruby import paths, got: {paths:?}"
    );
}

#[test]
fn ruby_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_complexity: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("ruby")
        .expect("ruby complexity query missing");
    let complexity = collect_captures(&lang, RUBY_SAMPLE, &query_str, "complexity");
    // classify() alone now contributes if + elsif (2); pop's rescue,
    // build_report/with_yield's statement modifiers, and describe's
    // case_match/in_clause pattern-match all add further complexity nodes.
    assert!(
        complexity.len() >= 8,
        "expected at least 8 complexity nodes in ruby sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn ruby_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_types: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_types("ruby").expect("ruby types query missing");
    // Ruby types.scm captures @type.reference (superclass/scope resolution)
    let refs = collect_captures(&lang, RUBY_SAMPLE, &query_str, "type");
    // BoundedStack < Stack (plain superclass) and Stack's own `rescue
    // StandardError` are both real type references now present in the
    // enriched sample.
    assert!(
        refs.contains(&"Stack".to_string()),
        "expected 'Stack' superclass reference in ruby sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.rb) -

/// Every grammar-legal variant of `call.method` that ruby.calls.scm claims to
/// support (identifier, constant) must actually match, with the right
/// capture kind (dimension 3), not just the right text.
#[test]
fn ruby_calls_completeness_method_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_calls_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ruby").expect("ruby calls query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"), // plain_call: method: identifier
        ("call", "identifier", "length"),   // method_call_with_receiver: method: identifier
        ("call", "constant", "Integer"),    // bare_constant_call: method: constant
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in ruby.calls.scm \
             output for variants.rb, got: {caps:?}"
        );
    }

    // @call.qualifier must carry the receiver text, not the call name.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"v"),
        "expected 'v' qualifier for the receiver-qualified call, got: {qualifiers:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn ruby_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_calls_negative: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ruby").expect("ruby calls query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` IS a call (method: identifier "field"); it must appear
    // exactly once — the negative guard here is against it appearing twice
    // (once for the call, once spuriously for the bare-identifier catch-all
    // pattern, which is reserved for calls with no explicit `call` node).
    let field_calls = call_texts.iter().filter(|t| **t == "field").count();
    assert_eq!(
        field_calls, 1,
        "expected exactly 1 'field' call (holder.field), got {field_calls}: {call_texts:?}"
    );
    // `bound = read_via_call` reads a local variable; the local reference
    // itself must never be captured as a call.
    assert!(
        !call_texts.contains(&"read_via_call") || {
            // `read_via_call` also appears once as an actual call-site
            // target name earlier (`holder.field`'s result assignment does
            // not call anything named read_via_call) — guard that only the
            // legitimate call-producing text ever lands here.
            call_texts.iter().filter(|t| **t == "read_via_call").count() == 0
        },
        "local variable 'read_via_call' must never be captured as a call, got: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `class.name`/`method.name` that
/// ruby.tags.scm claims to support must produce the correct definition kind.
#[test]
fn ruby_tags_completeness_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_tags_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ruby").expect("ruby tags query missing");
    let query = Query::new(&lang, &query_str).expect("query compilation failed");
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(RUBY_VARIANTS, None).expect("parse failed");
    let source_bytes = RUBY_VARIANTS.as_bytes();

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
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);

    // class.name: constant (Plain), scope_resolution (Namespaced::Deep).
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.class" && n == "Plain"),
        "expected 'Plain' class (name: constant), got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.class" && n == "Deep"),
        "expected 'Deep' class (name: scope_resolution, from Namespaced::Deep), got: {pairs:?}"
    );

    // method.name: identifier (build), operator (+), setter (name=).
    let def_method_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.method")
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        def_method_names.contains(&"+"),
        "expected 'def +' operator method (name: operator), got: {def_method_names:?}"
    );
    assert!(
        def_method_names.contains(&"name="),
        "expected 'def name=' setter method (name: setter), got: {def_method_names:?}"
    );
    assert!(
        def_method_names.contains(&"build"),
        "expected 'build' method nested inside 'class << self', got: {def_method_names:?}"
    );

    // Bare Kernel-style call captured as @reference.call with kind constant.
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "reference.call" && k == "constant" && t == "Integer"),
        "expected 'Integer' bare-constant call as reference.call, got: {caps:?}"
    );
}

/// Negative case: `class << self`'s singleton_class container has no name
/// field (its value is the bare `self` keyword) and must never itself
/// produce a @definition.class/@definition.module capture with a
/// fabricated name.
#[test]
fn ruby_tags_negative_singleton_class_has_no_definition() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_tags_negative: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ruby").expect("ruby tags query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);
    let self_named_defs = caps
        .iter()
        .filter(|(cn, _, t, _)| {
            (cn == "definition.class" || cn == "definition.module") && t == "self"
        })
        .count();
    assert_eq!(
        self_named_defs, 0,
        "singleton_class ('class << self') must never produce a definition named \
         'self', got: {caps:?}"
    );
}

/// Every grammar-legal variant of require/require_relative/load/using/
/// include/extend/prepend that ruby.imports.scm claims to support.
#[test]
fn ruby_imports_completeness_directive_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_imports_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("ruby")
        .expect("ruby imports query missing");
    let paths = collect_captures(&lang, RUBY_VARIANTS, &query_str, "import.path");

    assert!(
        paths.contains(&"json".to_string()),
        "expected require 'json', got: {paths:?}"
    );
    assert!(
        paths.contains(&"other".to_string()),
        "expected require_relative 'other', got: {paths:?}"
    );
    assert!(
        paths.contains(&"plain.rb".to_string()),
        "expected load 'plain.rb', got: {paths:?}"
    );
    assert!(
        paths.contains(&"RefinementModule".to_string()),
        "expected 'using RefinementModule', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Comparable".to_string()),
        "expected 'include Comparable' (bare constant), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("ActiveSupport")),
        "expected 'include ActiveSupport::Concern' (scope_resolution), got: {paths:?}"
    );
    assert!(
        paths.contains(&"Forwardable".to_string()),
        "expected 'extend Forwardable' (bare constant), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("MyLib")),
        "expected 'extend MyLib::Extensions' (scope_resolution), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("MyModule")),
        "expected 'prepend MyModule::Prependable' (scope_resolution), got: {paths:?}"
    );
}

/// Every grammar-legal variant of statement-modifier and pattern-match
/// complexity nodes that ruby.complexity.scm claims to support, plus elsif
/// branch counting.
#[test]
fn ruby_complexity_completeness_modifier_and_pattern_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_complexity_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("ruby")
        .expect("ruby complexity query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();

    for kind in [
        "if_modifier",
        "unless_modifier",
        "while_modifier",
        "until_modifier",
        "rescue_modifier",
        "elsif",
        "case_match",
        "in_clause",
    ] {
        assert!(
            complexity_kinds.contains(&kind),
            "expected a @complexity capture of kind '{kind}' in variants.rb, \
             got kinds: {complexity_kinds:?}"
        );
    }

    // elsif_chain has two elsif branches — both must count independently,
    // not be folded into a single complexity point for the whole chain.
    let elsif_count = complexity_kinds.iter().filter(|k| **k == "elsif").count();
    assert_eq!(
        elsif_count, 2,
        "expected exactly 2 'elsif' complexity nodes (elsif_chain has two), got {elsif_count}"
    );
}

/// Every grammar-legal variant of `superclass` that ruby.types.scm claims to
/// support: plain constant, namespaced (scope_resolution), and dynamic/
/// computed (call) superclasses.
#[test]
fn ruby_types_completeness_superclass_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_types_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_types("ruby").expect("ruby types query missing");
    let refs = collect_captures(&lang, RUBY_VARIANTS, &query_str, "type");

    // PlainSuper < Plain
    assert!(
        refs.contains(&"Plain".to_string()),
        "expected 'Plain' superclass reference, got: {refs:?}"
    );
    // NamespacedSuper < Outer2::Nested — covered by the generic
    // scope_resolution pattern, not a dedicated superclass one.
    assert!(
        refs.contains(&"Outer2".to_string()) && refs.contains(&"Nested".to_string()),
        "expected 'Outer2'/'Nested' from the namespaced superclass, got: {refs:?}"
    );
    // DynamicSuper < Struct.new(:x, :y) — best-effort receiver capture.
    assert!(
        refs.contains(&"Struct".to_string()),
        "expected 'Struct' receiver reference from the dynamic superclass, got: {refs:?}"
    );
}

#[test]
fn ruby_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ruby_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "ruby",
        RUBY_SAMPLE,
        &["# A simple stack data structure"],
    );
}
