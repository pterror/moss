//! Query fixture tests for swift.
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
// Swift
// ---------------------------------------------------------------------------

const SWIFT_SAMPLE: &str = include_str!("fixtures/swift/sample.swift");

const SWIFT_VARIANTS: &str = include_str!("fixtures/swift/variants.swift");

// --- Dimension 4: real-world fixture coverage (sample.swift) ----------------

#[test]
fn swift_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let names = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in swift tags, got: {names:?}"
    );
    // Protocol + protocol extension: the protocol itself and its
    // requirement/associatedtype must all be found.
    assert!(
        names.contains(&"Greetable".to_string()),
        "expected 'Greetable' protocol in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"greet".to_string()),
        "expected 'greet' protocol requirement in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Payload".to_string()),
        "expected 'Payload' associatedtype in swift tags, got: {names:?}"
    );
    // Generic function with a constraint (`<T: Comparable>`) must still be
    // found like any other function.
    assert!(
        names.contains(&"largest".to_string()),
        "expected 'largest' generic function in swift tags, got: {names:?}"
    );
    // Enum with associated values: both the enum and its cases.
    assert!(
        names.contains(&"NetworkResult".to_string()),
        "expected 'NetworkResult' enum in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"success".to_string()) && names.contains(&"cancelled".to_string()),
        "expected enum cases 'success'/'cancelled' in swift tags, got: {names:?}"
    );
    // Extension: previously entirely invisible (name field is
    // user_type-wrapped, not a bare type_identifier).
    assert!(
        names.contains(&"Coordinate".to_string()),
        "expected 'Coordinate' class in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"magnitude".to_string()),
        "expected 'magnitude' computed property (declared in an extension) \
         in swift tags, got: {names:?}"
    );
    // Standard-operator overload declared inside an extension.
    assert!(
        names.contains(&"==".to_string()),
        "expected '==' operator overload in swift tags, got: {names:?}"
    );
    // Member properties: onComplete (var) on Downloader.
    assert!(
        names.contains(&"onComplete".to_string()),
        "expected 'onComplete' member property in swift tags, got: {names:?}"
    );
}

#[test]
fn swift_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_calls: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("swift")
        .expect("swift calls query missing");
    let calls = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"print".to_string()) || calls.contains(&"push".to_string()),
        "expected 'print' or 'push' call in swift sample, got: {calls:?}"
    );
    // Trailing closure call (`numbers.map { ... }`) — the call_suffix's
    // lambda_literal content doesn't change the callee shape.
    assert!(
        calls.contains(&"map".to_string()),
        "expected trailing-closure 'map' call in swift sample, got: {calls:?}"
    );
    // Force-unwrap call: `onComplete!()`.
    assert!(
        calls.contains(&"onComplete".to_string()),
        "expected force-unwrap 'onComplete' call in swift sample, got: {calls:?}"
    );
}

#[test]
fn swift_imports_finds_module_imports() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_imports: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("swift")
        .expect("swift imports query missing");
    let paths = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Foundation") || p.contains("Swift")),
        "expected 'Foundation' or 'Swift' in swift import paths, got: {paths:?}"
    );
}

#[test]
fn swift_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_complexity: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("swift")
        .expect("swift complexity query missing");
    let complexity = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in swift sample, got {} ({complexity:?})",
        complexity.len()
    );
    // `largest<T: Comparable>` uses `guard ... else { return nil }` — must
    // count toward complexity like any other branch.
    let source_from_guard = SWIFT_SAMPLE.contains("guard var best = items.first");
    assert!(
        source_from_guard,
        "fixture must contain the guard statement this test relies on"
    );
    assert!(
        complexity.len() >= 8,
        "expected guard_statement/switch_entry/conjunction/disjunction to be \
         counted (sample has >=1 guard, a 4-case switch, and no boolean \
         operators yet at this count baseline), got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn swift_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_types: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("swift")
        .expect("swift types query missing");
    let refs = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Int" || r == "String" || r == "Bool"),
        "expected primitive type references in swift sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.swift) -

/// Every grammar-legal variant of declaration `name` fields that
/// swift.tags.scm claims to support must actually match, with the right
/// capture *kind* (dimension 3) — not just the right text.
#[test]
fn swift_tags_completeness_all_declaration_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags_completeness: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let pairs = collect_tag_pairs(&lang, SWIFT_VARIANTS, &query_str);

    // plain_name / custom_operator / standard-operator-overload function names.
    assert!(
        pairs.contains(&(
            "definition.function".to_string(),
            "plainFunction".to_string()
        )),
        "expected plain function name, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "+++".to_string())),
        "expected custom_operator overload '+++', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "==".to_string())),
        "expected standard-operator overload '==', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "+=".to_string())),
        "expected compound-assignment operator overload '+=', got: {pairs:?}"
    );

    // plain class name vs. extension (user_type-wrapped) name.
    assert!(
        pairs.contains(&("definition.class".to_string(), "PlainClass".to_string())),
        "expected plain class name, got: {pairs:?}"
    );
    // "PlainClass" appears twice: once for the class itself (type_identifier)
    // and once for its extension (user_type -> type_identifier) — both must
    // be present as separate matches.
    let plain_class_defs = pairs
        .iter()
        .filter(|(k, n)| k == "definition.class" && n == "PlainClass")
        .count();
    assert_eq!(
        plain_class_defs, 2,
        "expected 2 'PlainClass' definitions (class + extension), got {plain_class_defs}: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.class".to_string(), "Array".to_string())),
        "expected extension-of-generic-stdlib-type name 'Array', got: {pairs:?}"
    );

    // Enum cases: single, associated-value, and comma-separated multi-name.
    for case_name in ["ready", "failed", "paused", "cancelled"] {
        assert!(
            pairs.contains(&("definition.constant".to_string(), case_name.to_string())),
            "expected enum case '{case_name}', got: {pairs:?}"
        );
    }

    // Member let/var + computed property (class_body), and enum_class_body
    // variant of the same ancestor restriction.
    assert!(
        pairs.contains(&("definition.constant".to_string(), "readOnly".to_string())),
        "expected member 'let readOnly', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "mutable".to_string())),
        "expected member 'var mutable', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "computed".to_string())),
        "expected computed property 'computed', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "isA".to_string())),
        "expected enum computed property 'isA' (enum_class_body variant), got: {pairs:?}"
    );

    // Protocol requirements: property, method, associatedtype.
    assert!(
        pairs.contains(&("definition.var".to_string(), "label".to_string())),
        "expected protocol property requirement 'label', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.method".to_string(), "describe".to_string())),
        "expected protocol method requirement 'describe', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.type".to_string(), "Value".to_string())),
        "expected protocol associatedtype 'Value', got: {pairs:?}"
    );
}

/// Local `let`/`var` declarations inside function bodies share a node kind
/// (property_declaration) with member-level properties, but must never be
/// captured as @definition.constant/@definition.var — verified with exact
/// zero counts, not just "absent from a name list" (a false positive that
/// happened to collide with another name would otherwise hide the bug).
#[test]
fn swift_tags_negative_local_declarations_not_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags_negative: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let pairs = collect_tag_pairs(&lang, SWIFT_VARIANTS, &query_str);

    for local_name in [
        "localReadOnly",
        "localMutable",
        "notAMember",
        "alsoNotAMember",
    ] {
        let count = pairs
            .iter()
            .filter(|(k, n)| {
                (k == "definition.constant" || k == "definition.var") && n == local_name
            })
            .count();
        assert_eq!(
            count, 0,
            "local declaration '{local_name}' must never be captured as a \
             member constant/var, got {count} match(es): {pairs:?}"
        );
    }
}

/// Every grammar-legal variant of `call_expression.function` (plus the
/// distinct postfix_expression/constructor_expression callee shapes) that
/// swift.calls.scm claims to support must actually match, with the right
/// capture kind.
#[test]
fn swift_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_calls_completeness: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("swift")
        .expect("swift calls query missing");
    let caps = collect_captures_full(&lang, SWIFT_VARIANTS, &query_str);

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
    // method_call: function: navigation_expression -> simple_identifier
    assert!(
        call_names.contains(&"get"),
        "expected 'get' method call, got: {call_names:?}"
    );
    // force-unwrap call: function: postfix_expression(target, operation: bang)
    assert!(
        call_names.contains(&"completion"),
        "expected force-unwrap 'completion' call, got: {call_names:?}"
    );
    // optional-chaining call: plain identifier callee, same as plain_call.
    let completion_calls = call_names.iter().filter(|n| **n == "completion").count();
    assert_eq!(
        completion_calls, 2,
        "expected 2 'completion' calls (force-unwrap + optional-chaining), \
         got {completion_calls}: {call_names:?}"
    );
    // generic type instantiation call: constructor_expression, constructed_type:
    // (user_type (type_identifier)).
    assert!(
        call_names.contains(&"GenericBox"),
        "expected generic-instantiation call 'GenericBox', got: {call_names:?}"
    );
    assert!(
        call_names.contains(&"Optional"),
        "expected generic-instantiation call 'Optional', got: {call_names:?}"
    );

    // Every @call capture must be one of the node kinds the query actually
    // targets — never the parenthesized wrapper or anything larger
    // (extraction depth: capture kind, not just text).
    for (cn, kind, text, line) in &caps {
        if cn == "call" {
            assert!(
                kind == "simple_identifier" || kind == "type_identifier",
                "expected @call capture kind to be simple_identifier/type_identifier, \
                 got kind={kind} text={text} line={line}"
            );
        }
    }

    // @call.qualifier must carry the qualifier text for the method call.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"b"),
        "expected 'b' qualifier for the method call, got: {qualifiers:?}"
    );
}

/// Negative cases: call_expression.function variants with no stable,
/// nameable callee (curried calls, IIFEs, bracket type-literal calls) must
/// never produce a @call capture.
#[test]
fn swift_calls_negative_uncallable_function_variants_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_calls_negative: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("swift")
        .expect("swift calls query missing");
    let caps = collect_captures_full(&lang, SWIFT_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // NEGATIVE: curried call `makeAdder()(1)` — the INNER call (`makeAdder()`)
    // is a real plain_call and must be captured once; the OUTER call (whose
    // callee is the inner call_expression's *result*) must not add a second
    // 'makeAdder' capture.
    let make_adder_calls = call_texts.iter().filter(|t| **t == "makeAdder").count();
    assert_eq!(
        make_adder_calls, 1,
        "expected exactly 1 'makeAdder' call (the inner call only, not the \
         curried outer call), got {make_adder_calls}: {call_texts:?}"
    );
    // NEGATIVE: IIFE `{ (x: Int) -> Int in x * 2 }(5)` — anonymous callee —
    // and the bracket type-literal call `[Int](repeating:count:)` must
    // produce no capture at all. No text assertion is possible for either
    // (there is no name to accidentally capture); instead assert the total
    // capture count matches exactly the full expected set of named calls
    // across variants.swift, so a stray capture from either would be caught.
    let expected_calls = [
        "Vector",
        "reduce",
        "print",
        "identity",
        "Box",
        "get",
        "Optional",
        "completion",
        "completion",
        "GenericBox",
        "Optional",
        "makeAdder",
        "print",
    ];
    let mut actual_sorted = call_texts.clone();
    actual_sorted.sort_unstable();
    let mut expected_sorted: Vec<&str> = expected_calls.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(
        actual_sorted, expected_sorted,
        "expected exactly the named calls in variants.swift, got: {call_texts:?}"
    );
}

/// guard_statement / switch_entry / conjunction_expression / disjunction_expression
/// must all be counted individually — completeness + extraction-depth check
/// against the dedicated complexityVariants function in variants.swift.
#[test]
fn swift_complexity_completeness_guard_switch_and_boolean_operators() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_complexity_completeness: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("swift")
        .expect("swift complexity query missing");
    let caps = collect_captures_full(&lang, SWIFT_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();

    assert!(
        complexity_kinds.contains(&"guard_statement"),
        "expected guard_statement to count toward complexity, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"conjunction_expression"),
        "expected conjunction_expression (&&) to count toward complexity, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"disjunction_expression"),
        "expected disjunction_expression (||) to count toward complexity, got: {complexity_kinds:?}"
    );
    // complexityVariants has 4 switch_entry nodes (1, 2-3, where-guarded, default).
    let switch_entry_count = complexity_kinds
        .iter()
        .filter(|k| **k == "switch_entry")
        .count();
    assert_eq!(
        switch_entry_count, 4,
        "expected 4 switch_entry complexity nodes, got {switch_entry_count}: {complexity_kinds:?}"
    );
}

/// `let`-bound vs `var`-bound member properties must land in distinct
/// capture kinds (@definition.constant vs @definition.var) — the closest
/// analog in this query to a read/write or definition/reference
/// distinction, since Swift's tags query has no separate reference captures.
#[test]
fn swift_tags_distinguishes_let_and_var_member_properties() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags_let_var: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags_let_var: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let pairs = collect_tag_pairs(&lang, SWIFT_VARIANTS, &query_str);

    assert!(
        pairs.contains(&("definition.constant".to_string(), "readOnly".to_string())),
        "expected 'readOnly' as @definition.constant, got: {pairs:?}"
    );
    assert!(
        !pairs.contains(&("definition.var".to_string(), "readOnly".to_string())),
        "'readOnly' (a `let`) must not ALSO appear as @definition.var, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "mutable".to_string())),
        "expected 'mutable' as @definition.var, got: {pairs:?}"
    );
    assert!(
        !pairs.contains(&("definition.constant".to_string(), "mutable".to_string())),
        "'mutable' (a `var`) must not ALSO appear as @definition.constant, got: {pairs:?}"
    );
}

#[test]
fn swift_decorations_finds_attribute_and_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping swift_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "swift",
        SWIFT_SAMPLE,
        &["@discardableResult", "/// Classify"],
    );
}
