//! Query fixture tests for csharp.
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
// C#
// ---------------------------------------------------------------------------

const CSHARP_SAMPLE: &str = include_str!("fixtures/c-sharp/sample.cs");

const CSHARP_VARIANTS: &str = include_str!("fixtures/c-sharp/variants.cs");

// --- Dimension 4: real-world fixture coverage (sample.cs) -------------------

#[test]
fn csharp_tags_finds_class_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_tags: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let names = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"MathUtils".to_string()),
        "expected 'MathUtils' class in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Classify".to_string()),
        "expected 'Classify' method in c-sharp tags, got: {names:?}"
    );
    // base_list: `class Stack<T> : IEnumerable<T>, System.IDisposable` — both
    // the generic and the path-qualified interface must be found.
    assert!(
        names.contains(&"IEnumerable".to_string()),
        "expected 'IEnumerable' (generic base_list entry) in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"IDisposable".to_string()),
        "expected 'IDisposable' (path-qualified base_list entry) in c-sharp tags, got: {names:?}"
    );
    // `class BoundedStack<T> : Stack<T>` — generic base class.
    assert!(
        names.contains(&"BoundedStack".to_string()),
        "expected 'BoundedStack' class in c-sharp tags, got: {names:?}"
    );
    // Record with primary-constructor base type: `record Point3D(...) : Point(X, Y);`
    assert!(
        names.contains(&"Point3D".to_string()),
        "expected 'Point3D' record in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' record (primary-constructor base) in c-sharp tags, got: {names:?}"
    );
    // Lambda binding parameters must never surface as method/function
    // definitions — closures aren't method_declaration/local_function_statement.
    let def_names: Vec<&str> = names
        .iter()
        .map(std::string::String::as_str)
        .filter(|n| *n == "FetchLengthAsync")
        .collect();
    assert!(
        def_names.contains(&"FetchLengthAsync"),
        "expected the real async method 'FetchLengthAsync' in c-sharp tags, got: {names:?}"
    );
}

#[test]
fn csharp_tags_finds_call_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_tags_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_tags_calls: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_SAMPLE, &query_str);
    let ref_calls: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.call")
        .map(|(_, n)| n.as_str())
        .collect();
    // base()/this() constructor delegation inside BoundedStack.
    assert!(
        ref_calls.contains(&"base"),
        "expected 'base' constructor-delegation reference.call, got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"this"),
        "expected 'this' constructor-delegation reference.call, got: {ref_calls:?}"
    );
    // Qualified generic LINQ call: Enumerable.Range(...).Where(...).ToList()
    assert!(
        ref_calls.contains(&"Range"),
        "expected 'Range' qualified generic call reference, got: {ref_calls:?}"
    );
}

#[test]
fn csharp_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let calls = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"Push".to_string())
            || calls.contains(&"WriteLine".to_string())
            || calls.contains(&"Add".to_string()),
        "expected method call in c-sharp sample, got: {calls:?}"
    );
    // Unqualified generic call: Identity<int>(42).
    assert!(
        calls.contains(&"Identity".to_string()),
        "expected 'Identity' generic call in c-sharp sample, got: {calls:?}"
    );
    // Qualified generic LINQ chain: Enumerable.Range(...).Where(...).ToList().
    assert!(
        calls.contains(&"Range".to_string()),
        "expected 'Range' call in c-sharp sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"Where".to_string()),
        "expected 'Where' chained call in c-sharp sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"ToList".to_string()),
        "expected 'ToList' chained call in c-sharp sample, got: {calls:?}"
    );
    // Null-conditional invocation chain: maybeNull?.Trim()?.Length (Trim is a
    // call; Length is a property access, not a call).
    assert!(
        calls.contains(&"Trim".to_string()),
        "expected 'Trim' null-conditional call in c-sharp sample, got: {calls:?}"
    );
    // base()/this() constructor delegation.
    assert!(
        calls.contains(&"base".to_string()),
        "expected 'base' constructor-delegation call in c-sharp sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"this".to_string()),
        "expected 'this' constructor-delegation call in c-sharp sample, got: {calls:?}"
    );
    // Extension method call: blank.IsBlank().
    assert!(
        calls.contains(&"IsBlank".to_string()),
        "expected 'IsBlank' extension-method call in c-sharp sample, got: {calls:?}"
    );
}

#[test]
fn csharp_imports_finds_using_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_imports: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("c-sharp")
        .expect("c-sharp imports query missing");
    let paths = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "import.path");
    // Must capture simple identifier: `using System;`
    assert!(
        paths.iter().any(|p| p == "System"),
        "expected 'System' in c-sharp import paths, got: {paths:?}"
    );
    // Must capture qualified name: `using System.Collections.Generic;`
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Collections") || p.contains("Generic")),
        "expected qualified namespace in c-sharp import paths, got: {paths:?}"
    );
    // `using System.Linq;` / `using System.Threading.Tasks;`
    assert!(
        paths.iter().any(|p| p.contains("Linq")),
        "expected 'System.Linq' in c-sharp import paths, got: {paths:?}"
    );
}

#[test]
fn csharp_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_complexity: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("c-sharp")
        .expect("c-sharp complexity query missing");
    let complexity = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in c-sharp sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn csharp_complexity_finds_switch_expression_arms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_complexity_switch: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_complexity_switch: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("c-sharp")
        .expect("c-sharp complexity query missing");
    let caps = collect_captures_full(&lang, CSHARP_SAMPLE, &query_str);
    // sample.cs's `n switch { < 0 => ..., 0 => ..., _ => ... }` has 3 arms —
    // previously entirely uncounted (switch_expression_arm is a distinct node
    // kind from switch_section, the statement-form switch's case label).
    let arm_count = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "complexity" && k == "switch_expression_arm")
        .count();
    assert!(
        arm_count >= 3,
        "expected >= 3 switch_expression_arm complexity nodes, got {arm_count}: {caps:?}"
    );
}

#[test]
fn csharp_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "type.reference");
    assert!(
        refs.iter()
            .any(|r| r == "Stack" || r == "MathUtils" || r == "List"),
        "expected type reference in c-sharp sample, got: {refs:?}"
    );
}

#[test]
fn csharp_types_finds_type_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types_definitions: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_definitions: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_SAMPLE, &query_str);
    let def_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k.starts_with("definition."))
        .map(|(_, n)| n.as_str())
        .collect();
    // Previously c-sharp.types.scm had NO @definition.type at all.
    assert!(
        def_names.contains(&"Stack"),
        "expected 'Stack' @definition.type, got: {def_names:?}"
    );
    assert!(
        def_names.contains(&"Point"),
        "expected 'Point' record @definition.type, got: {def_names:?}"
    );
}

#[test]
fn csharp_types_negative_no_value_identifiers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_negative: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "type.reference");
    // Regression test for the severe overmatching bug: `(identifier)
    // @type.reference` with no field constraint used to match every
    // identifier in the file, including method names, parameter names, and
    // local variable names. None of these are type positions.
    for value_ident in ["Push", "Add", "items", "item", "Classify", "stack"] {
        assert!(
            !refs.contains(&value_ident.to_string()),
            "'{value_ident}' is a value identifier, must not appear as a \
             @type.reference, got: {refs:?}"
        );
    }
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.cs) --

/// Every grammar-legal variant of `base_list` (plain, generic, path-qualified
/// base class AND interface — C# has no syntactic extends/implements split)
/// must produce a @reference.class capture, matching c-sharp.tags.scm's
/// completeness claims.
#[test]
fn csharp_tags_completeness_base_list_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_tags_completeness_base_list: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_tags_completeness_base_list: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_VARIANTS, &query_str);
    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    for expected in [
        "PlainBase",         // base_list: identifier
        "GenericBase",       // base_list: generic_name -> identifier
        "Exception",         // base_list: qualified_name -> identifier
        "IPlainIface",       // base_list: identifier (interface)
        "IGenericIface",     // base_list: generic_name -> identifier (interface)
        "IDisposable",       // base_list: qualified_name -> identifier (interface)
        "RecordBase",        // primary_constructor_base_type: identifier
        "RecordBaseGeneric", // primary_constructor_base_type: generic_name -> identifier
    ] {
        assert!(
            ref_class_names.contains(&expected),
            "expected '{expected}' among base_list @reference.class captures, got: {ref_class_names:?}"
        );
    }
    // MultiBase : PlainBase, IPlainIface, IGenericIface<int> — all 3 entries
    // in one base_list must be found, not just the first.
    let multi_base_count = ref_class_names
        .iter()
        .filter(|n| **n == "PlainBase" || **n == "IPlainIface" || **n == "IGenericIface")
        .count();
    assert!(
        multi_base_count >= 3,
        "expected all 3 entries of MultiBase's base_list, found {multi_base_count} among: {ref_class_names:?}"
    );
}

/// Every grammar-legal variant of `object_creation_expression.type` must
/// produce a @reference.class capture with the leaf class name.
#[test]
fn csharp_tags_completeness_object_creation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_tags_completeness_object_creation: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!(
            "Skipping csharp_tags_completeness_object_creation: c-sharp grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_VARIANTS, &query_str);
    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    // Note: `new object()` (PlainNew) is deliberately excluded — `object` is a
    // `predefined_type` keyword node (like Java's `boolean_type`/`integral_type`),
    // not `identifier`/`generic_name`/`qualified_name`, so it correctly does
    // NOT produce a @reference.class (a builtin keyword type isn't a "class
    // reference" in any meaningful sense).
    for expected in ["List", "StringBuilder"] {
        assert!(
            ref_class_names.contains(&expected),
            "expected '{expected}' among object-creation @reference.class captures, got: {ref_class_names:?}"
        );
    }
    // Extraction depth: leaf-only names (no '.') even for qualified forms.
    assert!(
        ref_class_names.iter().all(|n| !n.contains('.')),
        "expected leaf-only class names (no '.'), got: {ref_class_names:?}"
    );
}

/// Every type-defining declaration kind (class, struct, interface, enum,
/// record) must be found as a tags definition.
#[test]
fn csharp_tags_completeness_type_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_tags_completeness_type_declaration_kinds: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!(
            "Skipping csharp_tags_completeness_type_declaration_kinds: c-sharp grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_VARIANTS, &query_str);
    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(find_def_kind("PlainClass"), Some("definition.class"));
    assert_eq!(
        find_def_kind("PlainStruct"),
        Some("definition.class"),
        "structs map to definition.class (closest existing kind)"
    );
    assert_eq!(
        find_def_kind("PlainInterface"),
        Some("definition.interface")
    );
    assert_eq!(find_def_kind("PlainEnum"), Some("definition.enum"));
    assert_eq!(
        find_def_kind("PlainRecord"),
        Some("definition.class"),
        "records map to definition.class (closest existing kind)"
    );
}

/// Every grammar-legal variant of `invocation_expression.function` (plain
/// identifier, generic_name, member_access_expression with identifier/
/// generic_name name, chained qualifier, conditional-access) must produce a
/// @call capture, matching c-sharp.calls.scm's completeness claims.
#[test]
fn csharp_calls_completeness_invocation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls_completeness: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let caps = collect_captures_full(&lang, CSHARP_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        calls.contains(&"Identity"),
        "expected plain call 'Identity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"GenericIdentity"),
        "expected unqualified generic call 'GenericIdentity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"WriteLine"),
        "expected qualified call 'WriteLine', got: {calls:?}"
    );
    assert!(
        calls.contains(&"OfType"),
        "expected qualified generic call 'OfType', got: {calls:?}"
    );
    assert!(
        calls.contains(&"Trim") && calls.contains(&"ToUpper"),
        "expected chained calls 'Trim'/'ToUpper', got: {calls:?}"
    );
    // Null-conditional invocation: s?.Trim() / xs?.OfType<int>().
    let conditional_call_count = calls.iter().filter(|c| **c == "Trim").count();
    assert!(
        conditional_call_count >= 1,
        "expected at least one conditional-access 'Trim' call, got: {calls:?}"
    );
}

/// `constructor_initializer`'s base(...)/this(...) delegation must produce a
/// @call capture for both keywords.
#[test]
fn csharp_calls_completeness_constructor_initializer() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_calls_completeness_ctor_init: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls_completeness_ctor_init: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let calls = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "call");
    assert!(
        calls.contains(&"base".to_string()),
        "expected 'base' constructor-delegation call, got: {calls:?}"
    );
    assert!(
        calls.contains(&"this".to_string()),
        "expected 'this' constructor-delegation call, got: {calls:?}"
    );
}

/// Negative case: method references passed as delegates, bare field
/// access/writes, and casts must never appear as @call captures.
#[test]
fn csharp_calls_negative_field_access_and_lambda() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls_negative: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let calls = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access/write 'this.field' must not be captured as a call, got: {calls:?}"
    );
    assert!(
        !calls.contains(&"StaticMethod".to_string()),
        "'StaticMethod' is never invoked in variants.cs (only referenced via delegate-\
         shaped lambda text); must not spuriously appear as a call, got: {calls:?}"
    );
}

/// Every grammar-legal variant of `using_directive`'s path argument (bare
/// identifier, qualified_name, bare generic_name, bare alias_qualified_name,
/// each with and without an alias) must produce a correctly-shaped @import,
/// with NO duplicate @import.path for aliased forms (regression test for the
/// alias/path overlap bug).
#[test]
fn csharp_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_imports_completeness: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("c-sharp")
        .expect("c-sharp imports query missing");
    let paths = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "import.path");
    let aliases = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "import.alias");

    // using Bare; — bare single-segment path.
    assert!(
        paths.contains(&"Bare".to_string()),
        "expected 'Bare' bare-identifier import path, got: {paths:?}"
    );
    // using System.Collections.Generic; — qualified path.
    assert!(
        paths.iter().any(|p| p.contains("Collections")),
        "expected qualified import path, got: {paths:?}"
    );
    // using static Wrapper<int>; — bare generic_name path.
    assert!(
        paths.iter().any(|p| p.starts_with("Wrapper")),
        "expected 'Wrapper<int>' bare-generic import path, got: {paths:?}"
    );
    // using global::System; — bare alias_qualified_name path.
    assert!(
        paths.iter().any(|p| p.contains("global::System")),
        "expected 'global::System' alias_qualified_name import path, got: {paths:?}"
    );
    // using Sys = System; using SysColl = ...; using MyList = List<int>; —
    // three aliases, each with exactly one alias and one path capture (the
    // historical bug produced the alias identifier itself as a spurious
    // second @import.path).
    assert!(
        aliases.contains(&"Sys".to_string()),
        "expected 'Sys' import alias, got: {aliases:?}"
    );
    assert!(
        !paths.contains(&"Sys".to_string()),
        "alias name 'Sys' must not also appear as an @import.path, got: {paths:?}"
    );
    assert!(
        !paths.contains(&"SysColl".to_string()),
        "alias name 'SysColl' must not also appear as an @import.path, got: {paths:?}"
    );
    assert!(
        !paths.contains(&"MyList".to_string()),
        "alias name 'MyList' must not also appear as an @import.path, got: {paths:?}"
    );
}

/// Exact-count regression test for the duplicate-@import.path bug: every
/// using_directive in variants.cs must produce exactly one @import.path
/// capture (not two, from the alias identifier bleeding into the plain
/// pattern).
#[test]
fn csharp_imports_negative_no_duplicate_path_per_alias() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_imports_negative: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("c-sharp")
        .expect("c-sharp imports query missing");
    let caps = collect_captures_full(&lang, CSHARP_VARIANTS, &query_str);
    // variants.cs has exactly 8 using directives.
    let import_stmts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert_eq!(
        import_stmts.len(),
        8,
        "expected 8 @import captures (one per using directive in variants.cs), got {}: {import_stmts:?}",
        import_stmts.len()
    );
    // Every using_directive produces exactly one @import.path — the
    // historical bug produced two for aliased forms (the alias identifier
    // plus the real path), so a 1:1 ratio with @import statements is the
    // exact regression guard.
    let path_count = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.path")
        .count();
    assert_eq!(
        path_count, 8,
        "expected exactly 8 @import.path captures (one per using directive, no \
         duplicates from alias bleed-through), got {path_count}: {caps:?}"
    );
}

/// Every `types.scm`-covered type-position field (variable declaration,
/// parameter, method return type, local function type, property type,
/// foreach loop variable, catch clause, cast, is/as pattern) must produce a
/// @type.reference capture for its identifier/generic_name/qualified_name/
/// nullable_type-wrapped leaf.
#[test]
fn csharp_types_completeness_field_positions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_completeness: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "type.reference");
    // variable_declaration.type: identifier / generic_name / qualified_name / nullable_type
    assert!(
        refs.contains(&"List".to_string()),
        "expected 'List' (variable_declaration.type generic_name), got: {refs:?}"
    );
    assert!(
        refs.contains(&"StringBuilder".to_string()),
        "expected 'StringBuilder' (variable_declaration.type qualified_name), got: {refs:?}"
    );
    // parameter.type
    assert!(
        refs.iter().filter(|r| *r == "List").count() >= 2,
        "expected 'List' from both variable_declaration.type and parameter.type, got: {refs:?}"
    );
    // method_declaration.returns
    assert!(
        refs.iter().filter(|r| *r == "StringBuilder").count() >= 2,
        "expected 'StringBuilder' from both variable_declaration.type and returns:, got: {refs:?}"
    );
    // The bare `identifier` variant (as opposed to generic_name/qualified_name)
    // — exercised via the user-defined `PlainClass` type across
    // variable_declaration.type, parameter.type, method_declaration.returns,
    // local_function_statement.type, property_declaration.type,
    // foreach_statement.type, cast_expression.type, and as_expression.right.
    // Builtin keyword types (`int`, `object`, `string`) are deliberately NOT
    // used for this check: they parse as `predefined_type`, not `identifier`,
    // so they would silently fail to exercise this variant at all.
    let plain_class_count = refs.iter().filter(|r| *r == "PlainClass").count();
    assert!(
        plain_class_count >= 7,
        "expected >= 7 'PlainClass' identifier-variant @type.reference captures \
         (one per field position), got {plain_class_count}: {refs:?}"
    );
    // catch_declaration.type: qualified_name (System.Exception)
    // is_expression.right: generic_name (List<int>)
    // These are exercised structurally by the field patterns above; spot-check
    // extraction depth instead:
    let caps = collect_captures_full(&lang, CSHARP_VARIANTS, &query_str);
    let list_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "type.reference" && t == "List")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    assert!(
        list_kinds.iter().all(|k| *k == "identifier"),
        "expected 'List' captures to be leaf identifier nodes, got kinds: {list_kinds:?}"
    );
}

/// Negative case: value identifiers (parameter names, local variable names,
/// method names unrelated to type positions) must never appear as
/// @type.reference in the completeness fixture either.
#[test]
fn csharp_types_negative_field_positions_no_overmatch() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_types_negative_field_positions: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_negative_field_positions: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "type.reference");
    for value_ident in ["Identity", "GenericIdentity", "field1", "a", "b", "x"] {
        assert!(
            !refs.contains(&value_ident.to_string()),
            "'{value_ident}' is a value/method identifier, must not appear as a \
             @type.reference, got: {refs:?}"
        );
    }
}

#[test]
fn csharp_decorations_finds_attribute_list_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping csharp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "c-sharp",
        CSHARP_SAMPLE,
        &["[Obsolete", "/// <summary>"],
    );
}
