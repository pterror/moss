//! Query fixture tests for cpp.
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
// C++
// ---------------------------------------------------------------------------

const CPP_SAMPLE: &str = include_str!("fixtures/cpp/sample.cpp");

const CPP_VARIANTS: &str = include_str!("fixtures/cpp/variants.cpp");

// --- Dimension 4: real-world fixture coverage (sample.cpp) ------------------

#[test]
fn cpp_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let names = collect_captures(&lang, CPP_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in cpp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()) && names.contains(&"sum_evens".to_string()),
        "expected 'classify' and 'sum_evens' functions in cpp tags, got: {names:?}"
    );
    // Namespace container — previously zero namespace tags coverage at all.
    assert!(
        names.contains(&"shapes".to_string()),
        "expected 'shapes' namespace in cpp tags, got: {names:?}"
    );
    // Polymorphic base + derived class, with a destructor *declared* inline
    // (`virtual ~Shape();`, no body) and *defined* out-of-line — previously
    // entirely untagged (destructor_name declarator variant).
    assert!(
        names.contains(&"Shape".to_string()) && names.contains(&"Circle".to_string()),
        "expected 'Shape' and 'Circle' classes in cpp tags, got: {names:?}"
    );
    let destructor_defs = names.iter().filter(|n| n.contains("~Shape")).count();
    assert_eq!(
        destructor_defs, 2,
        "expected exactly 2 '~Shape' function_declarator matches (the inline prototype \
         declaration plus the out-of-line definition — function_declarator has no body \
         constraint, so a prototype and its definition both match, exactly like every other \
         function/method in this query), got {destructor_defs}: {names:?}"
    );
    // Operator overload — previously entirely untagged (operator_name
    // declarator variant).
    assert!(
        names.iter().any(|n| n == "operator+="),
        "expected 'operator+=' overload in cpp tags, got: {names:?}"
    );
}

#[test]
fn cpp_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let calls = collect_captures(&lang, CPP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"push".to_string()) && calls.contains(&"pop".to_string()),
        "expected 'push' and 'pop' method calls in cpp sample, got: {calls:?}"
    );
    // Smart-pointer + polymorphism idiom: make_unique<Circle>(...), a
    // template-argument call.
    assert!(
        calls.iter().any(|c| c.contains("make_unique")),
        "expected 'make_unique<...>' templated call in cpp sample, got: {calls:?}"
    );
    // Plain template-argument call: identity<int>(21) — direct analogue of
    // Rust's turbofish gap.
    assert!(
        calls.iter().any(|c| c.contains("identity")),
        "expected 'identity<int>' templated call in cpp sample, got: {calls:?}"
    );
}

#[test]
fn cpp_imports_finds_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_imports: cpp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cpp")
        .expect("cpp imports query missing");
    let paths = collect_captures(&lang, CPP_SAMPLE, &query_str, "import.path");
    // Raw capture text still carries the angle brackets (`<iostream>`); the
    // Rust-side extraction layer strips them, not the query itself.
    assert!(
        paths.iter().any(|p| p.contains("iostream")) && paths.iter().any(|p| p.contains("vector")),
        "expected 'iostream' and 'vector' in cpp import paths, got: {paths:?}"
    );
    // `using namespace std::literals;` — previously zero `using` coverage at
    // all in cpp.imports.scm (only #include was tracked).
    assert!(
        paths.iter().any(|p| p.contains("literals")),
        "expected 'using namespace std::literals' import path in cpp sample, got: {paths:?}"
    );
}

#[test]
fn cpp_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_complexity: cpp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("cpp")
        .expect("cpp complexity query missing");
    let complexity = collect_captures(&lang, CPP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in cpp sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn cpp_types_finds_type_identifiers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_types: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_types("cpp").expect("cpp types query missing");
    let refs = collect_captures(&lang, CPP_SAMPLE, &query_str, "type");
    assert!(
        refs.iter().any(|r| r == "Stack"),
        "expected 'Stack' in cpp type references, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.cpp)

/// Every grammar-legal variant of union/class-specialization/namespace
/// definitions that cpp.tags.scm claims to support must produce a capture
/// with the correct kind, not just the right text.
#[test]
fn cpp_tags_completeness_union_specialization_namespace() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags_completeness: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);

    // Bare union definition — same struct/union asymmetry bug as C.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "union_specifier"
            && t.contains("PlainUnion")),
        "expected 'PlainUnion' union_specifier as definition.class, got: {caps:?}"
    );
    // Explicit template specialization: `template <> class TemplateClass<int>`
    // — name is wrapped in template_type, previously unmatched entirely.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "class_specifier"
            && t.contains("TemplateClass<int>")),
        "expected explicit specialization 'TemplateClass<int>' as definition.class, got: {caps:?}"
    );
    let names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        names.iter().filter(|n| **n == "TemplateClass").count() >= 2,
        "expected 'TemplateClass' name from both the primary template and its \
         specialization, got: {names:?}"
    );
    // Namespaces: plain, nested plain, and nested path-form
    // (`namespace deep::path::here`) — previously zero namespace tags
    // coverage of any kind.
    assert!(
        names.contains(&"outer_ns") && names.contains(&"inner_ns"),
        "expected 'outer_ns' and nested 'inner_ns' namespaces, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("deep") && n.contains("path") && n.contains("here")),
        "expected 'deep::path::here' nested_namespace_specifier name, got: {names:?}"
    );
}

/// Destructors and operator overloads — inline, out-of-line (plain class),
/// and out-of-line (template class) — must all be tagged as
/// @definition.method with the correct name, none of which were captured at
/// all before this fix.
#[test]
fn cpp_tags_completeness_destructors_and_operators() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags_completeness_dtor_op: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags_completeness_dtor_op: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);
    // @name carries just the destructor_name/operator_name node text (e.g.
    // "~WithSpecialMembers", "operator="); @definition.method carries the
    // whole function_declarator (e.g. "~WithSpecialMembers()"), which is
    // deliberately not what's asserted on here since the goal is verifying
    // the captured *name*, dimension 3.
    let method_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // Inline destructor + inline operator overload.
    assert!(
        method_names.contains(&"~WithSpecialMembers"),
        "expected inline destructor '~WithSpecialMembers' as name, got: {method_names:?}"
    );
    assert!(
        method_names.contains(&"operator="),
        "expected inline operator overload 'operator=' as name, got: {method_names:?}"
    );
    // Out-of-line destructor + operator overload (plain class).
    assert!(
        method_names.contains(&"~OutOfLineMembers"),
        "expected out-of-line destructor '~OutOfLineMembers' as name, got: {method_names:?}"
    );
    assert!(
        method_names.contains(&"operator+="),
        "expected out-of-line operator overload 'operator+=' as name, got: {method_names:?}"
    );
    // Out-of-line method on a template class, where the qualifier scope
    // itself carries template arguments (`OutOfLineTemplateMethods<T>::get`).
    assert!(
        method_names.contains(&"get"),
        "expected out-of-line template-class method 'get' as definition.method, got: {method_names:?}"
    );
}

/// Negative case: a lambda is not a `function_declarator`/`class_specifier`;
/// its parameter/body identifiers must never appear as @definition.function
/// or @definition.method.
#[test]
fn cpp_tags_negative_lambda_is_not_a_definition() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags_negative: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);
    let is_def_add_one = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t.contains("add_one")
    });
    assert!(
        !is_def_add_one,
        "lambda binding 'add_one' must never be captured as a function/method definition, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `field_expression.field` that
/// cpp.calls.scm claims to support (plain field_identifier, template_method,
/// destructor_name, qualified_identifier) must produce a @call capture with
/// the correct kind.
#[test]
fn cpp_calls_completeness_field_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls_completeness: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "field_identifier", "plain_method"), // pre-existing, still works
        ("call", "template_method", "templated_method<int>"), // previously unmatched
        ("call", "destructor_name", "~CallTarget"),   // previously unmatched
    ];
    for (cn, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(n, k, t, _)| n == cn && k == kind && t == text),
            "expected capture ({cn}, kind={kind}, text={text}) in cpp.calls.scm output for \
             variants.cpp, got: {caps:?}"
        );
    }
    // Explicit base-class-qualified call: derived.CallTarget::plain_method()
    // — field is a nested qualified_identifier.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "call"
            && k == "qualified_identifier"
            && t == "CallTarget::plain_method"),
        "expected base-qualified call 'CallTarget::plain_method' (kind=qualified_identifier), got: {caps:?}"
    );
}

/// Every grammar-legal variant of template-argument calls — plain
/// (`identity<int>(5)`) and scoped (`ns::helper<int>(3)`) — must produce a
/// @call capture, the direct C++ analogue of Rust's turbofish gap.
#[test]
fn cpp_calls_completeness_template_argument_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping cpp_calls_completeness_template: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls_completeness_template: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);

    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "template_function" && t == "identity<int>"),
        "expected plain template-argument call 'identity<int>' (kind=template_function), got: {caps:?}"
    );
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "template_function" && t == "helper<int>"),
        "expected scoped template-argument call 'helper<int>' (kind=template_function), got: {caps:?}"
    );
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"call_ns"),
        "expected 'call_ns' qualifier for the scoped template-argument call, got: {qualifiers:?}"
    );
}

/// Negative case: a bare field read must never appear in a @call capture.
#[test]
fn cpp_calls_negative_bare_field_access_is_not_a_call() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls_negative: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let calls = collect_captures(&lang, CPP_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access 'holder->field' must not be captured as a call, got: {calls:?}"
    );
}

/// Every grammar-legal variant of `using`/alias imports that cpp.imports.scm
/// claims to support — `using namespace X;`, `using X::Y;`, `using Alias =
/// Type;`, `namespace alias = X;` (single- and nested-segment) — must
/// produce a correctly-shaped @import, all previously entirely unsupported.
#[test]
fn cpp_imports_completeness_using_and_alias_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_imports_completeness: cpp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cpp")
        .expect("cpp imports query missing");
    let paths = collect_captures(&lang, CPP_VARIANTS, &query_str, "import.path");
    let aliases = collect_captures(&lang, CPP_VARIANTS, &query_str, "import.alias");

    // using namespace detail;
    assert!(
        paths.contains(&"detail".to_string()),
        "expected 'using namespace detail' import path, got: {paths:?}"
    );
    // using ns_target::Thing;
    assert!(
        paths.contains(&"ns_target::Thing".to_string()),
        "expected 'using ns_target::Thing' import path, got: {paths:?}"
    );
    // using IntAlias = int;
    assert!(
        aliases.contains(&"IntAlias".to_string()) && paths.contains(&"int".to_string()),
        "expected type-alias 'IntAlias = int', aliases={aliases:?} paths={paths:?}"
    );
    // namespace short_ns = ns_target;  (single-segment)
    assert!(
        aliases.contains(&"short_ns".to_string()) && paths.contains(&"ns_target".to_string()),
        "expected namespace alias 'short_ns = ns_target', aliases={aliases:?} paths={paths:?}"
    );
    // namespace nested_alias = ns_target::Thing::deeper;  (nested path)
    assert!(
        aliases.contains(&"nested_alias".to_string())
            && paths.iter().any(|p| p == "ns_target::Thing::deeper"),
        "expected namespace alias 'nested_alias = ns_target::Thing::deeper', \
         aliases={aliases:?} paths={paths:?}"
    );
}

#[test]
fn cpp_decorations_finds_attribute_declaration_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping cpp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "cpp",
        CPP_SAMPLE,
        &["[[nodiscard]]", "// Pushes an item onto the stack"],
    );
}
