//! Query fixture tests for php.
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
// PHP
// ---------------------------------------------------------------------------

const PHP_SAMPLE: &str = include_str!("fixtures/php/sample.php");

const PHP_VARIANTS: &str = include_str!("fixtures/php/variants.php");

// --- Dimension 4: real-world fixture coverage (sample.php) ------------------

#[test]
fn php_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_tags: php grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("php").expect("php tags query missing");
    let names = collect_captures(&lang, PHP_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in php tags, got: {names:?}"
    );
    // Trait, interface, enum containers must also surface as definitions.
    assert!(
        names.contains(&"Loggable".to_string()),
        "expected 'Loggable' trait in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Comparable".to_string()),
        "expected 'Comparable' interface in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Direction".to_string()),
        "expected 'Direction' enum in php tags, got: {names:?}"
    );

    // References: extends/implements and constructor calls must now surface
    // too (previously entirely absent — see php.tags.scm's "References"
    // section for the field-by-field verification).
    let pairs = collect_tag_pairs(&lang, PHP_SAMPLE, &query_str);
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.class" && n == "Stack"),
        "expected 'extends Stack' (BoundedStack) as reference.class, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "Comparable"),
        "expected 'implements Comparable' as reference.implementation, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.class" && n == "Stack"),
        "expected 'new Stack()' as reference.class, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "push"),
        "expected '$stack->push(...)' as reference.call, got: {pairs:?}"
    );
}

#[test]
fn php_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_calls: php grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("php").expect("php calls query missing");
    let calls = collect_captures(&lang, PHP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"classify".to_string())
            || calls.contains(&"array_push".to_string())
            || calls.contains(&"empty".to_string()),
        "expected a function call in php sample, got: {calls:?}"
    );
    // Static method call (BoundedStack::class is a constant-fetch, not a
    // call — check parent::push(...) and parent::__construct() instead,
    // real scoped_call_expression sites in the sample).
    assert!(
        calls.contains(&"push".to_string()),
        "expected 'parent::push(...)'/'$stack->push(...)' method call, got: {calls:?}"
    );
    // Namespace-qualified function call.
    assert!(
        calls.iter().any(|c| c.contains("classify")),
        "expected '\\App\\Collections\\classify(3)' namespaced call, got: {calls:?}"
    );
}

#[test]
fn php_imports_finds_use_declarations() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_imports: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("php")
        .expect("php imports query missing");
    let paths = collect_captures(&lang, PHP_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("User") || p.contains("Collection") || p.contains("App")),
        "expected namespace path in php import paths, got: {paths:?}"
    );
    // Bare single-segment `use Countable;`/`use Traversable;` (no
    // namespace separator) — previously dropped entirely.
    assert!(
        paths.contains(&"Countable".to_string()),
        "expected bare 'use Countable;', got: {paths:?}"
    );
    // `require_once __DIR__ . '/bootstrap.php';` — the string-literal
    // suffix of a concatenation; require_expression/require_once_expression
    // were previously entirely unmatched (only include* was handled).
    assert!(
        paths.iter().any(|p| p.contains("bootstrap.php")),
        "expected 'require_once ... bootstrap.php' path, got: {paths:?}"
    );
}

#[test]
fn php_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_complexity: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("php")
        .expect("php complexity query missing");
    let complexity = collect_captures(&lang, PHP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in php sample, got {} ({complexity:?})",
        complexity.len()
    );
    // `match ($this) { ... }` arms in Direction::opposite() and
    // `describeDirection` must count too.
    let caps = collect_captures_full(&lang, PHP_SAMPLE, &query_str);
    assert!(
        caps.iter()
            .any(|(cn, k, _, _)| cn == "complexity" && k == "match_conditional_expression"),
        "expected at least one match_conditional_expression @complexity, got: {caps:?}"
    );
    // `$n % 2 === 0 && $n > 0` — the `&&` must count as its own branch.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "complexity"
            && k == "binary_expression"
            && t.contains("&&")),
        "expected the '&&' in sumEvens to count as @complexity, got: {caps:?}"
    );
}

#[test]
fn php_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_types: php grammar .so not found");
        return;
    };
    let query_str = loader.get_types("php").expect("php types query missing");
    let refs = collect_captures(&lang, PHP_SAMPLE, &query_str, "type");
    assert!(
        refs.contains(&"Direction".to_string()) || refs.contains(&"mixed".to_string()),
        "expected a type reference (Direction/mixed/etc) in php sample, got: {refs:?}"
    );
}

// --- Dimension 2/3: completeness matrix + extraction depth (variants.php) --

/// Every grammar-legal variant of `function_call_expression.function` /
/// `scoped_call_expression.name` / `member_call_expression.name` /
/// `nullsafe_member_call_expression.name` that php.calls.scm claims to
/// support, asserted by capture kind (not just text).
#[test]
fn php_calls_completeness_all_callee_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_calls_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("php").expect("php calls query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let calls: Vec<(&str, &str)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, k, t, _)| (k.as_str(), t.as_str()))
        .collect();

    // function_call_expression.function variants.
    assert!(
        calls.contains(&("name", "helperFn")),
        "expected plain function call (function: name), got: {calls:?}"
    );
    // `$fn();` drills into variable_name to capture the *variable's own*
    // name ("fn") — the AST has no notion of the string value ("helperFn")
    // the variable happens to hold, only the identifier being called.
    assert!(
        calls.contains(&("name", "fn")),
        "expected variable function call ($fn(), function: variable_name -> \
         name, capturing the variable's own name) got: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "qualified_name" && t.contains("classify")),
        "expected namespaced function call (function: qualified_name), got: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "relative_name" && t.contains("helperFn")),
        "expected relative-namespace function call (function: relative_name), got: {calls:?}"
    );

    // scoped_call_expression.name variants.
    assert!(
        calls.contains(&("name", "on")),
        "expected static method call (scoped_call name: name), got: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "variable_name" && *t == "$method"),
        "expected dynamic static method call (scoped_call name: variable_name), got: {calls:?}"
    );

    // member_call_expression.name variants.
    assert!(
        calls.contains(&("name", "next")),
        "expected nullsafe method call (name: name), got: {calls:?}"
    );

    // object_creation_expression is NOT a call (see php.calls.scm comment):
    // must never contribute a @call capture.
    assert!(
        !calls.iter().any(|(_, t)| *t == "Widget"),
        "constructor invocation must not be captured as @call, got: {calls:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn php_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_calls_negative: php grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("php").expect("php calls query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // A property read ($this->field) must never appear as a call.
    assert!(
        !call_texts.contains(&"field"),
        "property read must not be captured as @call, got: {call_texts:?}"
    );
    // Anonymous class instantiation contributes no @call/name capture.
    assert!(
        !call_texts.iter().any(|t| t.contains("implements Shape")),
        "anonymous class body must never leak into @call text, got: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `object_creation_expression`/
/// `base_clause`/`class_interface_clause` that php.tags.scm's new
/// @reference.class/@reference.implementation patterns claim to support.
#[test]
fn php_tags_completeness_reference_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_tags_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("php").expect("php tags query missing");
    // @reference.class/@reference.implementation are attached to the
    // *container* node (object_creation_expression/base_clause/
    // class_interface_clause), not the field-variant node itself — so
    // `tags_matches_by_kind` (which correlates each match's anchor capture
    // with that match's @name capture) is required here, not
    // `collect_captures_full` filtered by capture name (that would report
    // every reference.class hit as kind "object_creation_expression"/
    // "base_clause", the container's own kind, not the variant).
    let class_refs = tags_matches_by_kind(&lang, PHP_VARIANTS, &query_str, "reference.class");
    let class_ref_pairs: Vec<(&str, &str)> = class_refs
        .iter()
        .map(|(k, t)| (k.as_str(), t.as_str()))
        .collect();

    assert!(
        class_ref_pairs
            .iter()
            .any(|(k, t)| *k == "qualified_name" && t.contains("User")),
        "expected 'new \\App\\Models\\User()' (object_creation: qualified_name), got: {class_ref_pairs:?}"
    );
    assert!(
        class_ref_pairs
            .iter()
            .any(|(k, t)| *k == "relative_name" && t.contains("Widget")),
        "expected 'new namespace\\Widget()' (object_creation: relative_name), got: {class_ref_pairs:?}"
    );
    assert!(
        class_ref_pairs
            .iter()
            .any(|(k, t)| *k == "variable_name" && *t == "$cls"),
        "expected 'new $cls()' (object_creation: variable_name), got: {class_ref_pairs:?}"
    );
    // 'new Widget()' (object_creation: name) and 'extends Widget'
    // (base_clause: name) both produce identical ("name", "Widget") pairs
    // by design — assert at least 2 occurrences so a regression that drops
    // either pattern is still caught.
    let widget_name_refs = class_ref_pairs
        .iter()
        .filter(|(k, t)| *k == "name" && *t == "Widget")
        .count();
    assert!(
        widget_name_refs >= 2,
        "expected both 'new Widget()' (object_creation) and 'extends Widget' \
         (base_clause) to each produce a ('name', 'Widget') reference.class \
         capture, got {widget_name_refs}: {class_ref_pairs:?}"
    );

    let impl_refs =
        tags_matches_by_kind(&lang, PHP_VARIANTS, &query_str, "reference.implementation");
    let impl_ref_pairs: Vec<(&str, &str)> = impl_refs
        .iter()
        .map(|(k, t)| (k.as_str(), t.as_str()))
        .collect();
    assert!(
        impl_ref_pairs.contains(&("name", "Shape")),
        "expected 'implements Shape' (class_interface_clause: name), got: {impl_ref_pairs:?}"
    );
    assert!(
        impl_ref_pairs.contains(&("name", "Colored")),
        "expected 'implements Colored' (class_interface_clause: name), got: {impl_ref_pairs:?}"
    );
}

/// Negative case: anonymous class instantiation must never produce a
/// @reference.class capture with fabricated name text (no name field
/// exists for `anonymous_class`).
#[test]
fn php_tags_negative_anonymous_class_has_no_reference() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_tags_negative: php grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("php").expect("php tags query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let anon_leaks = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "reference.class" && t.contains("implements Shape"))
        .count();
    assert_eq!(
        anon_leaks, 0,
        "anonymous_class body must never leak into a @reference.class capture, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `namespace_use_declaration`/
/// `use_declaration`/`require*`/`include*` that php.imports.scm claims to
/// support.
#[test]
fn php_imports_completeness_directive_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_imports_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("php")
        .expect("php imports query missing");
    let paths = collect_captures(&lang, PHP_VARIANTS, &query_str, "import.path");

    assert!(
        paths.iter().any(|p| p.contains("User")),
        "expected 'use App\\Models\\User;' (qualified_name, no alias), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("Order")),
        "expected 'use App\\Models\\Order as OrderModel;' path, got: {paths:?}"
    );
    assert!(
        paths.contains(&"Exception".to_string()),
        "expected bare 'use Exception;' (name, no alias), got: {paths:?}"
    );
    assert!(
        paths.contains(&"Throwable".to_string()),
        "expected bare 'use Throwable as T;' path, got: {paths:?}"
    );
    assert!(
        paths.contains(&"Loggable".to_string()) && paths.contains(&"Cacheable".to_string()),
        "expected grouped 'use App\\Traits\\{{Loggable, Cacheable as Cache}};' members, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("bootstrap.php")),
        "expected 'require_once ... bootstrap.php', got: {paths:?}"
    );
    assert!(
        paths.contains(&"config.php".to_string()),
        "expected 'require config.php', got: {paths:?}"
    );
    assert!(
        paths.contains(&"legacy.php".to_string()),
        "expected 'include legacy.php', got: {paths:?}"
    );
    assert!(
        paths.contains(&"once.php".to_string()),
        "expected 'include_once once.php', got: {paths:?}"
    );
    // Trait composition (use_declaration, distinct from namespace imports).
    assert!(
        paths.contains(&"GreetingTrait".to_string())
            && paths.contains(&"FarewellTrait".to_string()),
        "expected trait composition 'use GreetingTrait;'/'use FarewellTrait, GreetingTrait;', \
         got: {paths:?}"
    );
}

/// Aliased imports must not double-count @import.path (the alias-form and
/// bare-form patterns previously both fired for every aliased `use`).
#[test]
fn php_imports_negative_alias_does_not_double_count() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_imports_negative: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("php")
        .expect("php imports query missing");
    let paths = collect_captures(&lang, PHP_VARIANTS, &query_str, "import.path");
    let order_count = paths.iter().filter(|p| p.contains("Order")).count();
    assert_eq!(
        order_count, 1,
        "'use App\\Models\\Order as OrderModel;' must produce exactly 1 \
         @import.path capture, got {order_count}: {paths:?}"
    );
    let throwable_count = paths.iter().filter(|p| **p == "Throwable").count();
    assert_eq!(
        throwable_count, 1,
        "'use Throwable as T;' must produce exactly 1 @import.path capture, \
         got {throwable_count}: {paths:?}"
    );
}

/// Every grammar-legal variant of `named_type`'s children (name,
/// qualified_name, relative_name) that php.types.scm claims to support.
#[test]
fn php_types_completeness_named_type_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_types_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader.get_types("php").expect("php types query missing");
    let refs = collect_captures(&lang, PHP_VARIANTS, &query_str, "type");

    assert!(
        refs.contains(&"int".to_string()),
        "expected primitive_type 'int', got: {refs:?}"
    );
    assert!(
        refs.contains(&"Widget".to_string()),
        "expected named_type -> name 'Widget', got: {refs:?}"
    );
    assert!(
        refs.iter().any(|t| t.contains("User")),
        "expected named_type -> qualified_name '\\App\\Models\\User', got: {refs:?}"
    );
    assert!(
        refs.iter().any(|t| t.contains("namespace\\Widget")),
        "expected named_type -> relative_name 'namespace\\Widget', got: {refs:?}"
    );
    // Union type members (int|string) — each must appear.
    assert!(
        refs.contains(&"string".to_string()),
        "expected union_type member 'string', got: {refs:?}"
    );
}

/// Negative case: union type members must not be double-counted (a
/// redundant union_type-specific pattern previously duplicated every
/// match the unanchored named_type rule already produced).
#[test]
fn php_types_negative_union_type_not_double_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_types_negative: php grammar .so not found");
        return;
    };
    let query_str = loader.get_types("php").expect("php types query missing");
    let refs = collect_captures(&lang, PHP_VARIANTS, &query_str, "type");
    // `Shape&Colored $f` (intersection_type) is the only site each of these
    // two names appears in a type position in variants.php — unlike
    // "Widget"/"int", which legitimately appear at several distinct
    // parameter sites, so a >1 count for either of these specifically
    // indicates the same named_type node was captured twice, not two
    // different real sites.
    for name in ["Shape", "Colored"] {
        let count = refs.iter().filter(|t| t.as_str() == name).count();
        assert_eq!(
            count, 1,
            "'{name}' (from the 'Shape&Colored' intersection_type) must \
             produce exactly 1 @type.reference capture, got {count}: {refs:?}"
        );
    }
}

/// Every grammar-legal variant of `match_conditional_expression` and the
/// short-circuit boolean operator set that php.complexity.scm claims to
/// support.
#[test]
fn php_complexity_completeness_match_and_boolean_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_complexity_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("php")
        .expect("php complexity query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();

    assert!(
        complexity_kinds.contains(&"match_conditional_expression"),
        "expected match_conditional_expression @complexity, got: {complexity_kinds:?}"
    );

    let bool_ops: Vec<&str> = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "complexity" && k == "binary_expression")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    for op in ["&&", "||", "and", "or", "xor"] {
        assert!(
            bool_ops.iter().any(|t| t.contains(op)),
            "expected a binary_expression @complexity containing operator '{op}', \
             got: {bool_ops:?}"
        );
    }
}

/// Negative cases: `match_default_expression` (the default arm) and a
/// plain arithmetic binary_expression must never count as @complexity.
#[test]
fn php_complexity_negative_default_arm_and_arithmetic_not_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_complexity_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_complexity_negative: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("php")
        .expect("php complexity query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let default_arms = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "complexity" && k == "match_default_expression")
        .count();
    assert_eq!(
        default_arms, 0,
        "match_default_expression must never count as @complexity, got: {caps:?}"
    );
    let arithmetic_hits = caps
        .iter()
        .filter(|(cn, k, t, _)| {
            cn == "complexity" && k == "binary_expression" && t.contains("1 + 2")
        })
        .count();
    assert_eq!(
        arithmetic_hits, 0,
        "plain arithmetic '1 + 2' must never count as @complexity, got: {caps:?}"
    );
}

#[test]
fn php_decorations_finds_attribute_list_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping php_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "php",
        PHP_SAMPLE,
        &["#[Pure]"],
    );
}
