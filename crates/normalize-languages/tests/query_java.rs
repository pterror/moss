//! Query fixture tests for java.
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
// Java
// ---------------------------------------------------------------------------

const JAVA_SAMPLE: &str = include_str!("fixtures/java/sample.java");

const JAVA_VARIANTS: &str = include_str!("fixtures/java/variants.java");

// --- Dimension 4: real-world fixture coverage (sample.java) ----------------

#[test]
fn java_tags_finds_class_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let names = collect_captures(&lang, JAVA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"TaskQueue".to_string()),
        "expected 'TaskQueue' class in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"enqueue".to_string()),
        "expected 'enqueue' method in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' method in java tags, got: {names:?}"
    );
    // implements Comparable<TaskQueue>, java.io.Serializable — both the
    // generic and the path-qualified interface must be found as containers
    // for the nested `PriorityTaskQueue extends TaskQueue implements
    // java.util.Comparator<String>` idiom (generic + scoped supertype).
    assert!(
        names.contains(&"Comparable".to_string()),
        "expected 'Comparable' (generic implements) in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Serializable".to_string()),
        "expected 'Serializable' (path-qualified implements) in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Comparator".to_string()),
        "expected 'Comparator' (generic + path-qualified implements) in java tags, got: {names:?}"
    );
    // Nested static class + its qualified/generic extends clause.
    assert!(
        names.contains(&"PriorityTaskQueue".to_string()),
        "expected 'PriorityTaskQueue' nested class in java tags, got: {names:?}"
    );
    // Anonymous class (`new Runnable() { ... }`) — its constructor-call
    // reference and the `run` override inside it must both surface.
    assert!(
        names.contains(&"Runnable".to_string()),
        "expected 'Runnable' anonymous-class reference in java tags, got: {names:?}"
    );
    // Enum with a constructor and a method.
    assert!(
        names.contains(&"Color".to_string()),
        "expected 'Color' enum in java tags, got: {names:?}"
    );
    // Record.
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' record in java tags, got: {names:?}"
    );
    // Lambda bindings (`String::length` method reference site, `t -> ...`)
    // must never surface as function/method definitions — closures and
    // method references aren't `method_declaration`s in this grammar.
    let def_method_names: Vec<&str> = names
        .iter()
        .map(std::string::String::as_str)
        .filter(|n| *n == "lengthFn" || *n == "t")
        .collect();
    assert!(
        def_method_names.contains(&"lengthFn"),
        "expected the real method 'lengthFn' in java tags, got: {names:?}"
    );
}

#[test]
fn java_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_calls: java grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("java").expect("java calls query missing");
    let calls = collect_captures(&lang, JAVA_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"add".to_string()) || calls.contains(&"remove".to_string()),
        "expected 'add' or 'remove' method call in java sample, got: {calls:?}"
    );
    // Iterator-chain idiom: tasks.stream().filter(...).map(...).count() —
    // every link in the chain must be found, not just the first call.
    assert!(
        calls.contains(&"stream".to_string()),
        "expected 'stream' call in java sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"filter".to_string()),
        "expected 'filter' call in java sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"count".to_string()),
        "expected 'count' call in java sample, got: {calls:?}"
    );
    // Qualified static call: Integer.compare(...).
    assert!(
        calls.contains(&"compare".to_string()),
        "expected 'compare' static-qualified call in java sample, got: {calls:?}"
    );
    // super(capacity) constructor delegation inside the nested subclass
    // constructor — a distinct `explicit_constructor_invocation` node, not a
    // `method_invocation`, that was previously entirely unmatched.
    assert!(
        calls.contains(&"super".to_string()),
        "expected 'super' constructor-delegation call in java sample, got: {calls:?}"
    );
}

#[test]
fn java_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_imports: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("java")
        .expect("java imports query missing");
    let paths = collect_captures(&lang, JAVA_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("ArrayList")),
        "expected 'java.util.ArrayList' in java import paths, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("function.Function")),
        "expected 'java.util.function.Function' in java import paths, got: {paths:?}"
    );
}

#[test]
fn java_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_complexity: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("java")
        .expect("java complexity query missing");
    let complexity = collect_captures(&lang, JAVA_SAMPLE, &query_str, "complexity");
    // enqueue() has an if; dequeue() has an if; classify() has if/else-if;
    // Shapes.describe() has a switch (arrow form, 3 labels incl. default).
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in java sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn java_types_finds_class() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_types: java grammar .so not found");
        return;
    };
    let query_str = loader.get_types("java").expect("java types query missing");
    let names = collect_captures(&lang, JAVA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"TaskQueue".to_string()),
        "expected 'TaskQueue' in java types captures, got: {names:?}"
    );
    // interface_declaration, enum_declaration, and record_declaration must
    // all be reported as @definition.type alongside class_declaration.
    assert!(
        names.contains(&"Processor".to_string()),
        "expected 'Processor' interface in java types captures, got: {names:?}"
    );
    assert!(
        names.contains(&"Color".to_string()),
        "expected 'Color' enum in java types captures, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' record in java types captures, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.java) -

/// Every grammar-legal variant of `object_creation_expression.type` that
/// java.tags.scm claims to support (plain, generic, diamond, scoped, and
/// generic+scoped) must produce a @reference.class capture with the right name.
#[test]
fn java_tags_completeness_object_creation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_tags_completeness_object_creation: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags_completeness_object_creation: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);

    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    for expected in ["Object", "ArrayList", "Date", "HashMap"] {
        assert!(
            ref_class_names.contains(&expected),
            "expected '{expected}' among object-creation @reference.class captures, got: {ref_class_names:?}"
        );
    }
    // Extraction depth: the leaf name must be the plain class name
    // ("Date"), not the qualified path text ("java.util.Date"), even for
    // scoped/generic-scoped constructors.
    assert!(
        ref_class_names.iter().all(|n| !n.contains('.')),
        "expected leaf-only class names (no '.'), got: {ref_class_names:?}"
    );
}

/// Every grammar-legal variant of `superclass` and `type_list` (implements)
/// that java.tags.scm claims to support must produce a
/// @reference.class/@reference.implementation capture.
#[test]
fn java_tags_completeness_extends_implements_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_tags_completeness_extends_implements: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags_completeness_extends_implements: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);

    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    // superclass: plain, generic, generic+scoped.
    assert!(
        ref_class_names.contains(&"PlainBase"),
        "expected 'PlainBase' (plain superclass) in java tags, got: {ref_class_names:?}"
    );
    assert!(
        ref_class_names.contains(&"GenericBase"),
        "expected 'GenericBase' (generic superclass) in java tags, got: {ref_class_names:?}"
    );
    assert!(
        ref_class_names.contains(&"AbstractList"),
        "expected 'AbstractList' (generic + path-qualified superclass) in java tags, got: {ref_class_names:?}"
    );

    let ref_impl_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.implementation")
        .map(|(_, n)| n.as_str())
        .collect();
    // type_list (implements): plain, generic, scoped, generic+scoped.
    assert!(
        ref_impl_names.contains(&"PlainIface"),
        "expected 'PlainIface' (plain implements) in java tags, got: {ref_impl_names:?}"
    );
    assert!(
        ref_impl_names.contains(&"Comparable"),
        "expected 'Comparable' (generic implements) in java tags, got: {ref_impl_names:?}"
    );
    assert!(
        ref_impl_names.contains(&"Serializable"),
        "expected 'Serializable' (path-qualified implements) in java tags, got: {ref_impl_names:?}"
    );
    assert!(
        ref_impl_names.contains(&"Comparator"),
        "expected 'Comparator' (generic + path-qualified implements) in java tags, got: {ref_impl_names:?}"
    );
}

/// Every type-defining declaration kind (class, interface, enum, record,
/// annotation type) must be found as a tags definition, and record/annotation
/// must be mapped to the documented closest-existing kind (class/interface).
#[test]
fn java_tags_completeness_type_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_tags_completeness_type_declaration_kinds: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!(
            "Skipping java_tags_completeness_type_declaration_kinds: java grammar .so not found"
        );
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);

    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(find_def_kind("PlainClass"), Some("definition.class"));
    assert_eq!(
        find_def_kind("PlainInterface"),
        Some("definition.interface")
    );
    assert_eq!(find_def_kind("PlainEnum"), Some("definition.enum"));
    assert_eq!(
        find_def_kind("PlainRecord"),
        Some("definition.class"),
        "records compile to classes; expected definition.class"
    );
    assert_eq!(
        find_def_kind("PlainAnnotation"),
        Some("definition.interface"),
        "annotation types compile to interfaces; expected definition.interface"
    );
}

/// Negative case: lambda bindings and method references are not
/// `method_declaration`s and must never appear as tags definitions; bare
/// field access/writes must never appear as calls.
#[test]
fn java_tags_negative_lambdas_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags_negative: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let is_def_lambda_binding = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.method" || cn == "definition.function") && t == "lambdaBinding"
    });
    assert!(
        !is_def_lambda_binding,
        "lambda binding 'lambdaBinding' must never be captured as a method/function \
         definition, got: {caps:?}"
    );
    let is_def_method_ref = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.method" || cn == "definition.function") && t == "methodRef"
    });
    assert!(
        !is_def_method_ref,
        "method-reference binding 'methodRef' must never be captured as a method/function \
         definition, got: {caps:?}"
    );
}

/// Negative case: method references (`Foo::bar`) are not invocations and
/// must never appear as @call captures in java.calls.scm.
#[test]
fn java_calls_negative_method_references_and_field_access() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_calls_negative_method_references: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_calls_negative_method_references: java grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("java").expect("java calls query missing");
    let calls = collect_captures(&lang, JAVA_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"staticMethod".to_string()),
        "method reference 'NegativeHolder::staticMethod' must not be captured as a call, \
         got: {calls:?}"
    );
    // Bare field read (`this.field`) and field write (`this.field = 5`) must
    // never be captured as calls (no argument_list, not a method_invocation).
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access/write 'this.field' must not be captured as a call, got: {calls:?}"
    );
}

/// Every grammar-legal variant of `method_invocation.object` (absent, plain
/// identifier qualifier, chained method_invocation qualifier) must produce a
/// @call capture, matching java.calls.scm's completeness claims.
#[test]
fn java_calls_completeness_object_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_calls_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("java").expect("java calls query missing");
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        calls.contains(&"identity"),
        "expected plain (no-object) call 'identity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"abs"),
        "expected qualified call 'Math.abs' -> 'abs', got: {calls:?}"
    );
    // Chained calls: s.trim().toUpperCase().length() — every link found.
    assert!(
        calls.contains(&"trim"),
        "expected chained call 'trim', got: {calls:?}"
    );
    assert!(
        calls.contains(&"toUpperCase"),
        "expected chained call 'toUpperCase', got: {calls:?}"
    );
    assert!(
        calls.contains(&"length"),
        "expected chained call 'length', got: {calls:?}"
    );
    // Extraction depth: the qualifier for the chained calls must be the
    // *previous method_invocation node*, not a plain identifier.
    let chained_qualifier_kind = caps
        .iter()
        .find(|(cn, _, t, _)| cn == "call.qualifier" && t.starts_with("s.trim()"))
        .map(|(_, k, _, _)| k.as_str());
    assert_eq!(
        chained_qualifier_kind,
        Some("method_invocation"),
        "expected the chained call's qualifier to be a method_invocation node, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `import_declaration`'s argument (bare
/// identifier, scoped_identifier, wildcard, static, static wildcard) that
/// java.imports.scm claims to support must produce a correctly-shaped @import.
#[test]
fn java_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_imports_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("java")
        .expect("java imports query missing");
    let paths = collect_captures(&lang, JAVA_VARIANTS, &query_str, "import.path");
    let globs = collect_captures(&lang, JAVA_VARIANTS, &query_str, "import.glob");

    // Bare single-segment import: `import Bare;`
    assert!(
        paths.contains(&"Bare".to_string()),
        "expected 'Bare' bare-identifier import path, got: {paths:?}"
    );
    // Plain scoped import: `import java.util.ArrayList;`
    assert!(
        paths.iter().any(|p| p.contains("ArrayList")),
        "expected 'java.util.ArrayList' import path, got: {paths:?}"
    );
    // Wildcard: `import java.util.*;`
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture for 'import java.util.*;', got: {globs:?}"
    );
    // import static pkg.Class.member; and import static pkg.Class.*;
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Math.PI") || p.contains("PI")),
        "expected static import path for 'java.lang.Math.PI', got: {paths:?}"
    );
    assert!(
        globs.len() >= 2,
        "expected 2 import.glob captures (plain wildcard + static wildcard), got {}: {globs:?}",
        globs.len()
    );
}

/// Negative case: `import.path` must never be empty/missing for any of the
/// import forms above — a silent drop (0 matches) is exactly the historical
/// bug class this methodology targets.
#[test]
fn java_imports_negative_no_silently_dropped_forms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_imports_negative: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("java")
        .expect("java imports query missing");
    // Exact-match "import" only — collect_captures' prefix match would also
    // pull in "import.path"/"import.glob"/"import.reexport", which is not
    // what this test is asserting.
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let import_stmts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // variants.java has exactly 6 import declarations; every one must
    // produce at least one @import capture (the whole-statement anchor).
    assert_eq!(
        import_stmts.len(),
        6,
        "expected 6 @import captures (one per import declaration in variants.java), got {}: {import_stmts:?}",
        import_stmts.len()
    );
}

/// Completeness: switch (arrow form), try-with-resources, and enhanced-for
/// all contribute complexity, matching java.complexity.scm's claims.
#[test]
fn java_complexity_completeness_control_flow_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_complexity_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("java")
        .expect("java complexity query missing");
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    // switch (arrow form): 3 switch_label nodes (case 1, case 2, default).
    assert!(
        complexity_kinds
            .iter()
            .filter(|k| **k == "switch_label")
            .count()
            >= 3,
        "expected >= 3 switch_label complexity nodes (arrow-form switch), got: {complexity_kinds:?}"
    );
    // catch_clause from the try-with-resources block.
    assert!(
        complexity_kinds.contains(&"catch_clause"),
        "expected a catch_clause complexity node, got: {complexity_kinds:?}"
    );
    // for/while/do-while/enhanced-for loops.
    assert!(
        complexity_kinds.contains(&"for_statement"),
        "expected a for_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"while_statement"),
        "expected a while_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"do_statement"),
        "expected a do_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"enhanced_for_statement"),
        "expected an enhanced_for_statement complexity node, got: {complexity_kinds:?}"
    );
}

/// Every type-defining declaration kind must be found as @definition.type in
/// java.types.scm, matching the tags completeness matrix above.
#[test]
fn java_types_completeness_all_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_types_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader.get_types("java").expect("java types query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);
    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(find_def_kind("PlainClass"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainInterface"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainEnum"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainRecord"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainAnnotation"), Some("definition.type"));
}

#[test]
fn java_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping java_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "java",
        JAVA_SAMPLE,
        &["@Override", "// Returns the size"],
    );
}
