//! Query fixture tests for kotlin.
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
// Kotlin
// ---------------------------------------------------------------------------

const KOTLIN_SAMPLE: &str = include_str!("fixtures/kotlin/sample.kt");

const KOTLIN_VARIANTS: &str = include_str!("fixtures/kotlin/variants.kt");

// --- Dimension 4: real-world fixture coverage (sample.kt) -------------------

#[test]
fn kotlin_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_tags: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let names = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in kotlin tags, got: {names:?}"
    );
    // Sealed class hierarchy + interface implemented WITHOUT parens
    // (`class Circle(val r: Double) : Shape` — the near-ubiquitous Kotlin
    // idiom that was previously entirely unmatched).
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' interface reference (no-paren delegation) in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Figure".to_string()),
        "expected 'Figure' sealed class in kotlin tags, got: {names:?}"
    );
    // Secondary constructor delegating to the primary via `this(...)`.
    assert!(
        names.contains(&"this".to_string()),
        "expected 'this' constructor delegation reference in kotlin tags, got: {names:?}"
    );
    // Extension function and suspend function are both ordinary
    // function_declarations — must still surface as @definition.function.
    assert!(
        names.contains(&"shout".to_string()),
        "expected extension function 'shout' in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"fetchData".to_string()),
        "expected suspend function 'fetchData' in kotlin tags, got: {names:?}"
    );
    // Named companion object.
    assert!(
        names.contains(&"Repository".to_string()),
        "expected 'Repository' class in kotlin tags, got: {names:?}"
    );
}

#[test]
fn kotlin_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_calls: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("kotlin")
        .expect("kotlin calls query missing");
    let calls = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"println".to_string()) || calls.contains(&"enqueue".to_string()),
        "expected 'println' or 'enqueue' call in kotlin sample, got: {calls:?}"
    );
    // Trailing-lambda call: `listOf(1, 2, 3).map { it * 2 }`.
    assert!(
        calls.contains(&"map".to_string()),
        "expected trailing-lambda 'map' call in kotlin sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"filter".to_string()),
        "expected lambda-with-arrow 'filter' call in kotlin sample, got: {calls:?}"
    );
    // `Repository(name, 16)` secondary-constructor `this(...)` delegation —
    // a distinct `constructor_delegation_call` node, not `call_expression`,
    // previously entirely unmatched.
    assert!(
        calls.contains(&"this".to_string()),
        "expected 'this' constructor-delegation call in kotlin sample, got: {calls:?}"
    );
}

#[test]
fn kotlin_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_imports: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("kotlin")
        .expect("kotlin imports query missing");
    let paths = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("LinkedList") || p.contains("java")),
        "expected 'java.util.LinkedList' in kotlin import paths, got: {paths:?}"
    );
    // `import kotlin.math.max as mathMax` — aliased import must still
    // report its path (and, per the completeness test below, must not
    // also be double-counted by the plain-import pattern).
    assert!(
        paths.iter().any(|p| p.contains("max")),
        "expected 'kotlin.math.max' aliased import path in kotlin sample, got: {paths:?}"
    );
}

#[test]
fn kotlin_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_complexity: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("kotlin")
        .expect("kotlin complexity query missing");
    let complexity = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "complexity");
    // classify()'s when-arms, sumEvens()'s if, dequeue()'s if, the
    // when(figure) is-branches, and the try/catch all contribute.
    assert!(
        complexity.len() >= 5,
        "expected at least 5 complexity nodes in kotlin sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn kotlin_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_types: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("kotlin")
        .expect("kotlin types query missing");
    let refs = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Point" || r == "Double" || r == "Int"),
        "expected 'Point', 'Double', or 'Int' in kotlin type references, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.kt) -

/// Every type-defining declaration kind (class, interface, enum, sealed
/// class, object, type alias) must be found as a tags AND types definition
/// with the correct capture kind.
#[test]
fn kotlin_tags_completeness_type_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_type_declaration_kinds: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_type_declaration_kinds: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);

    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(
        find_def_kind("PlainClass"),
        Some("definition.class"),
        "expected PlainClass as definition.class, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("PlainObject"),
        Some("definition.class"),
        "expected PlainObject as definition.class, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("PlainAlias"),
        Some("definition.type"),
        "expected PlainAlias as definition.type, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("PlainInterface"),
        Some("definition.class"),
        "expected PlainInterface as definition.class (same node kind as class_declaration), got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("Direction"),
        Some("definition.class"),
        "expected enum class Direction as definition.class, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("SealedBase"),
        Some("definition.class"),
        "expected sealed class SealedBase as definition.class, got pairs: {pairs:?}"
    );
    // Enum entries are @definition.constant, not @definition.class.
    assert_eq!(
        find_def_kind("NORTH"),
        Some("definition.constant"),
        "expected enum entry NORTH as definition.constant, got pairs: {pairs:?}"
    );
}

/// Every grammar-legal shape of `delegation_specifier` (superclass call
/// with parens, bare interface reference with no parens, and `by`
/// delegation) must produce a @reference.class capture with the right name.
#[test]
fn kotlin_tags_completeness_delegation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_delegation_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_delegation_variants: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);

    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    // delegation_specifier -> constructor_invocation -> user_type (superclass
    // call with parens/args).
    assert!(
        ref_class_names.contains(&"OpenBase"),
        "expected 'OpenBase' (constructor-invocation delegation) in kotlin tags, got: {ref_class_names:?}"
    );
    // delegation_specifier -> user_type directly (bare interface reference,
    // no parens) — the most common Kotlin idiom, previously unmatched.
    assert!(
        ref_class_names.contains(&"PlainInterface"),
        "expected 'PlainInterface' (bare delegation, no parens) in kotlin tags, got: {ref_class_names:?}"
    );
    // delegation_specifier -> explicit_delegation -> user_type (`by`
    // interface delegation), previously unmatched.
    assert!(
        ref_class_names.contains(&"SuperBase"),
        "expected 'SuperBase' (bare delegation before secondary ctor) in kotlin tags, got: {ref_class_names:?}"
    );

    // Re-run the tags query but only look at ExplicitDelegationVariant's
    // line to disambiguate the `by`-delegation form specifically (the name
    // "PlainInterface" is reused above for the paren-less form).
    let full_captures = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);
    let by_delegation = full_captures
        .iter()
        .find(|(cap, _, text, _)| cap == "reference.class" && text.contains(" by impl"));
    assert!(
        by_delegation.is_some(),
        "expected a @reference.class capture spanning the `by impl` explicit_delegation, got: {full_captures:?}"
    );
}

/// `this(...)` / `super(...)` secondary-constructor delegation
/// (`constructor_delegation_call`, a distinct node kind from
/// `call_expression`) must produce a @reference.call in tags and a @call in
/// calls, with the correct capture kind (an anonymous keyword token, not
/// `simple_identifier`).
#[test]
fn kotlin_tags_completeness_constructor_delegation_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_constructor_delegation_calls: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_constructor_delegation_calls: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);
    assert!(
        pairs.contains(&("reference.call".to_string(), "this".to_string())),
        "expected 'this' constructor-delegation @reference.call, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("reference.call".to_string(), "super".to_string())),
        "expected 'super' constructor-delegation @reference.call, got: {pairs:?}"
    );
}

/// Every grammar-legal call shape (plain call, navigation/method call,
/// `this`/`super` constructor delegation) must produce a @call with the
/// correct capture kind.
#[test]
fn kotlin_calls_completeness_call_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_calls_completeness_call_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_calls_completeness_call_variants: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("kotlin")
        .expect("kotlin calls query missing");
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);

    let find_kind = |name: &str| -> Vec<&str> {
        full.iter()
            .filter(|(cap, _, text, _)| cap == "call" && text == name)
            .map(|(_, kind, _, _)| kind.as_str())
            .collect()
    };
    assert!(
        find_kind("println").contains(&"simple_identifier"),
        "expected plain call 'println' as simple_identifier, got: {full:?}"
    );
    assert!(
        find_kind("add").contains(&"simple_identifier"),
        "expected navigation call 'add' as simple_identifier, got: {full:?}"
    );
    assert!(
        find_kind("map").contains(&"simple_identifier"),
        "expected trailing-lambda call 'map' as simple_identifier, got: {full:?}"
    );
    // "this"/"super" constructor delegation: captured node kind is the
    // anonymous keyword token itself, not simple_identifier — distinct
    // extraction depth signal from ordinary calls.
    assert!(
        find_kind("this").contains(&"this"),
        "expected 'this' constructor-delegation call captured as kind 'this', got: {full:?}"
    );
    assert!(
        find_kind("super").contains(&"super"),
        "expected 'super' constructor-delegation call captured as kind 'super', got: {full:?}"
    );
}

/// Every grammar-legal `import_header` shape (plain, aliased, wildcard)
/// must produce exactly one @import per statement — no duplicates. The
/// plain-import pattern was previously unconstrained and also matched
/// every aliased/wildcard import.
#[test]
fn kotlin_imports_completeness_no_duplicate_matches() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_imports_completeness_no_duplicate_matches: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_imports_completeness_no_duplicate_matches: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("kotlin")
        .expect("kotlin imports query missing");
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);
    let import_paths: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.path")
        .map(|(_, _, text, _)| text.as_str())
        .collect();

    // variants.kt has exactly 3 import statements (plain, aliased,
    // wildcard); each must contribute exactly one @import.path.
    assert_eq!(
        import_paths,
        vec!["java.util.ArrayList", "java.util.HashMap", "kotlin.math"],
        "expected exactly one @import.path per import statement (no duplicates), got: {import_paths:?}"
    );

    let aliases: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.alias")
        .map(|(_, _, text, _)| text.as_str())
        .collect();
    assert_eq!(
        aliases,
        vec!["JHashMap"],
        "expected exactly one @import.alias, got: {aliases:?}"
    );

    let globs: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.glob")
        .map(|(_, _, text, _)| text.as_str())
        .collect();
    assert_eq!(
        globs,
        vec!["*"],
        "expected exactly one @import.glob, got: {globs:?}"
    );
}

/// Type-defining declarations must produce @definition.type, and the
/// blanket @type.reference pattern must not double-count qualified/generic
/// type usages (the fixed duplicate-match bug).
#[test]
fn kotlin_types_completeness_definitions_and_no_duplicates() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_types_completeness_definitions_and_no_duplicates: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_types_completeness_definitions_and_no_duplicates: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_types("kotlin")
        .expect("kotlin types query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);

    // collect_tag_pairs pairs the @name leaf (just the identifier) with its
    // @definition.type container, not the container's own (much larger)
    // span — the query's outer @definition.type capture spans the whole
    // declaration (e.g. "class PlainClass"), so asserting equality against
    // the outer capture's text (as collect_captures_full alone would) is
    // wrong; the leaf name is what a consumer actually wants.
    for expected in ["PlainClass", "PlainObject", "PlainAlias"] {
        assert!(
            pairs.contains(&("definition.type".to_string(), expected.to_string())),
            "expected '{expected}' among @definition.type captures, got: {pairs:?}"
        );
    }

    // "PlainClass" appears at exactly 4 distinct source lines in
    // variants.kt (the class declaration itself, `plainType: PlainClass?`,
    // `List<PlainClass>` generic argument, and the callable-reference
    // negative case) — each must produce exactly one @type.reference,
    // not two, even though the generic-argument occurrence is wrapped in
    // a `user_type` (the redundant pattern that caused the duplicate).
    let plain_class_ref_lines: Vec<usize> = full
        .iter()
        .filter(|(cap, _, text, _)| cap == "type.reference" && text == "PlainClass")
        .map(|(_, _, _, line)| *line)
        .collect();
    let mut sorted_lines = plain_class_ref_lines.clone();
    sorted_lines.sort_unstable();
    let mut deduped_lines = sorted_lines.clone();
    deduped_lines.dedup();
    assert_eq!(
        sorted_lines, deduped_lines,
        "expected no duplicate @type.reference lines for 'PlainClass' (found the same line twice), got: {plain_class_ref_lines:?}"
    );
    assert_eq!(
        plain_class_ref_lines.len(),
        4,
        "expected exactly 4 'PlainClass' @type.reference occurrences (decl, plain annotation, generic argument, callable-reference), got {}: {plain_class_ref_lines:?}",
        plain_class_ref_lines.len()
    );
}

// --- Negative cases: constructs that must NOT match -------------------------

/// Annotation usages WITH constructor args (`@Deprecated("...")`) must NOT
/// be misclassified as a @reference.class: `constructor_invocation` is
/// also a legal child of `annotation`, not just `delegation_specifier`,
/// and the tags query is deliberately scoped to exclude it.
#[test]
fn kotlin_tags_negative_annotation_args_not_class_reference() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_negative_annotation_args_not_class_reference: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_negative_annotation_args_not_class_reference: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);
    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        !ref_class_names.contains(&"Deprecated"),
        "the @Deprecated(\"...\") annotation must not be misclassified as @reference.class, got: {ref_class_names:?}"
    );
}

/// A top-level `val` (`property_declaration`) must never produce a
/// @definition.* capture: the grammar reuses `property_declaration` for
/// both class-level properties and local `val`/`var` bindings inside
/// function bodies with no reliable way to distinguish them without
/// ancestor traversal (documented in kotlin.tags.scm).
#[test]
fn kotlin_tags_negative_property_declarations_not_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_negative_property_declarations_not_captured: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_negative_property_declarations_not_captured: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let names = collect_captures(&lang, KOTLIN_VARIANTS, &query_str, "name");
    assert!(
        !names.contains(&"topLevelPropertyNegative".to_string()),
        "top-level 'val' must not appear in tags, got: {names:?}"
    );
}

/// An unnamed companion object (`companion object { ... }`, no explicit
/// name) has no `type_identifier` child at all and is architecturally
/// unable to produce a @name capture. This documents the absence rather
/// than asserting new behavior — Kotlin gives it the implicit name
/// "Companion", but the grammar provides no source text to capture that
/// name from, so fabricating it would violate "be honest about
/// capabilities" (CLAUDE.md).
#[test]
fn kotlin_tags_negative_unnamed_companion_object_has_no_name() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_negative_unnamed_companion_object_has_no_name: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_negative_unnamed_companion_object_has_no_name: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);
    // variants.kt's only `companion_object` node is the unnamed one inside
    // `UnnamedCompanionNegative`. Filter by capture *kind* (not just name
    // text — `UnnamedCompanionNegative` the outer class also legitimately
    // produces a @definition.class, but its capture's node kind is
    // `class_declaration`, not `companion_object`) to precisely isolate
    // whether the companion_object pattern fired at all.
    let companion_definitions: Vec<&(String, String, String, usize)> = full
        .iter()
        .filter(|(cap, kind, ..)| cap == "definition.class" && kind == "companion_object")
        .collect();
    assert!(
        companion_definitions.is_empty(),
        "expected no @definition.class capture with kind 'companion_object' for the unnamed companion object, got: {companion_definitions:?}"
    );
}

/// `::foo` / `Type::method` callable references are a distinct node kind
/// (`callable_reference`) from `call_expression` and must never be
/// misclassified as a call.
#[test]
fn kotlin_calls_negative_callable_reference_not_a_call() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_calls_negative_callable_reference_not_a_call: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_calls_negative_callable_reference_not_a_call: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_calls("kotlin")
        .expect("kotlin calls query missing");
    let calls = collect_captures(&lang, KOTLIN_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"hashCode".to_string()),
        "the 'PlainClass::hashCode' callable reference must not appear as a call, got: {calls:?}"
    );
}

#[test]
fn kotlin_tags_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_tags_live: kotlin grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let names = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "name");
    // After fix: should find classes and functions, NOT local val declarations
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function, got: {names:?}"
    );
    // Local val declarations should NOT appear
    assert!(
        !names.contains(&"dx".to_string()),
        "local 'dx' should not appear in tags, got: {names:?}"
    );
    assert!(
        !names.contains(&"total".to_string()),
        "local 'total' should not appear in tags, got: {names:?}"
    );
}

#[test]
fn kotlin_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping kotlin_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "kotlin",
        KOTLIN_SAMPLE,
        &["@JvmStatic", "// Classify a number"],
    );
}
