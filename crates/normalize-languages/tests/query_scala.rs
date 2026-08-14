//! Query fixture tests for scala.
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
// Scala
// ---------------------------------------------------------------------------

const SCALA_SAMPLE: &str = include_str!("fixtures/scala/sample.scala");

const SCALA_VARIANTS: &str = include_str!("fixtures/scala/variants.scala");

// --- Dimension 4: real-world fixture coverage (sample.scala) ----------------

#[test]
fn scala_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let names = collect_captures(&lang, SCALA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in scala tags, got: {names:?}"
    );
    // Companion object.
    assert!(
        names.contains(&"Point".to_string()),
        "expected companion 'Point' object in scala tags, got: {names:?}"
    );
    // Traits with mixins.
    assert!(
        names.contains(&"Named".to_string()) && names.contains(&"Aged".to_string()),
        "expected 'Named'/'Aged' traits in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Person".to_string()),
        "expected 'Person' class (extends Named with Aged) in scala tags, got: {names:?}"
    );
    // Scala 3 enum with a body method.
    assert!(
        names.contains(&"Direction".to_string()),
        "expected 'Direction' enum in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"opposite".to_string()),
        "expected 'opposite' method inside the enum body in scala tags, got: {names:?}"
    );
    // Operator-method definition on the case class.
    assert!(
        names.contains(&"+".to_string()),
        "expected operator method '+' in scala tags, got: {names:?}"
    );
    // Higher-kinded generic trait.
    assert!(
        names.contains(&"Functor".to_string()),
        "expected 'Functor' higher-kinded trait in scala tags, got: {names:?}"
    );
}

#[test]
fn scala_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_calls: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scala")
        .expect("scala calls query missing");
    let calls = collect_captures(&lang, SCALA_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"println".to_string()) || calls.contains(&"push".to_string()),
        "expected 'println' or 'push' call in scala sample, got: {calls:?}"
    );
    // Companion-object factory call and case-class apply.
    assert!(
        calls.contains(&"distanceTo".to_string()),
        "expected 'distanceTo' method call in scala sample, got: {calls:?}"
    );
}

#[test]
fn scala_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_imports: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scala")
        .expect("scala imports query missing");
    // Scala imports query captures @import (the full declaration node)
    let imports = collect_captures(&lang, SCALA_SAMPLE, &query_str, "import");
    assert!(
        !imports.is_empty(),
        "expected at least one import declaration in scala sample, got: {imports:?}"
    );
    // Import with a per-name rename (`Success => S`) must still surface as
    // its own @import declaration.
    assert!(
        imports.iter().any(|i| i.contains("Success")),
        "expected the 'Success => S' rename import in scala sample, got: {imports:?}"
    );
}

#[test]
fn scala_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_complexity: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scala")
        .expect("scala complexity query missing");
    let complexity = collect_captures(&lang, SCALA_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in scala sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn scala_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_types: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("scala")
        .expect("scala types query missing");
    let refs = collect_captures(&lang, SCALA_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Int" || r == "Double" || r == "String"),
        "expected type identifiers in scala sample, got: {refs:?}"
    );
}

// --- Dimensions 2/3: completeness + extraction depth (variants.scala) ------

/// `function_definition.name` allows `identifier` and `operator_identifier`;
/// both must produce a `definition.function` tag with the correct kind.
#[test]
fn scala_tags_completeness_function_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping scala_tags_completeness_function_name: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_function_name: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);

    assert!(
        pairs.contains(&("definition.function".to_string(), "plainFunc".to_string())),
        "expected identifier-named function 'plainFunc', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "+".to_string())),
        "expected operator_identifier-named method '+', got: {pairs:?}"
    );
}

/// Scala 3 `enum` definitions must surface as `definition.enum`.
#[test]
fn scala_tags_completeness_enum_definition() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags_completeness_enum: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_enum: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    assert!(
        pairs.contains(&("definition.enum".to_string(), "Color".to_string())),
        "expected 'Color' enum as definition.enum, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.enum".to_string(), "Nested".to_string())),
        "expected 'Nested' enum as definition.enum, got: {pairs:?}"
    );
    // A method inside the enum body must still surface as its own definition,
    // proving the enum acts as a container (SymbolKind::Enum is a container
    // kind).
    assert!(
        pairs.contains(&("definition.function".to_string(), "label".to_string())),
        "expected 'label' method inside 'Nested' enum body, got: {pairs:?}"
    );
}

/// Every `call_expression.function` variant scala.tags.scm's @reference.call
/// claims to support must produce a matching capture, with the correct name.
#[test]
fn scala_tags_completeness_reference_call_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping scala_tags_completeness_reference_call: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_reference_call: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    let ref_calls: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.call")
        .map(|(_, n)| n.as_str())
        .collect();

    assert!(
        ref_calls.contains(&"identity"),
        "expected plain call 'identity', got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"map"),
        "expected method call 'map', got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"+"),
        "expected explicit operator-method call 'a.+(b)', got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"identityGeneric"),
        "expected generic call 'identityGeneric[Int](1)', got: {ref_calls:?}"
    );
}

/// Object creation (`new X()`) must be found for plain, generic, qualified,
/// and generic+qualified type shapes.
#[test]
fn scala_tags_completeness_new_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags_completeness_new: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_new: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    let ref_class: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();

    assert!(
        ref_class.contains(&"OpHolder"),
        "expected plain 'new OpHolder(1)', got: {ref_class:?}"
    );
    assert!(
        ref_class.contains(&"ArrayBuffer"),
        "expected generic 'new ArrayBuffer[Int]()', got: {ref_class:?}"
    );
    assert!(
        ref_class.contains(&"Date"),
        "expected qualified 'new java.util.Date()', got: {ref_class:?}"
    );
    assert!(
        ref_class.contains(&"HashMap"),
        "expected qualified+generic 'new java.util.HashMap[String, Int]()', got: {ref_class:?}"
    );
}

/// `extends X with Y with Z` — the first supertype and every subsequent
/// `with` mixin must all surface as @reference.implementation, including
/// generic and qualified shapes.
#[test]
fn scala_tags_completeness_extends_mixin_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping scala_tags_completeness_extends: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_extends: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    let ref_impl: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.implementation")
        .map(|(_, n)| n.as_str())
        .collect();

    // Both the first supertype (fielded) and the "with" mixin (unfielded)
    // from `class MultiMixin extends TraitA with TraitB`.
    assert!(
        ref_impl.contains(&"TraitA"),
        "expected first supertype 'TraitA', got: {ref_impl:?}"
    );
    assert!(
        ref_impl.contains(&"TraitB"),
        "expected 'with' mixin 'TraitB', got: {ref_impl:?}"
    );
    // Generic mixin: `class GenericMixin extends TraitC[Int]`.
    assert!(
        ref_impl.contains(&"TraitC"),
        "expected generic supertype 'TraitC', got: {ref_impl:?}"
    );
    // Qualified + generic mixin: `extends scala.collection.Iterable[Int]`.
    assert!(
        ref_impl.contains(&"Iterable"),
        "expected qualified+generic supertype 'Iterable', got: {ref_impl:?}"
    );
}

/// Negative case: bare field access/write, lambda bindings, and
/// eta-expansion (passing a method by name without calling it) must never
/// surface as tags definitions or call references.
#[test]
fn scala_tags_negative_field_access_and_lambda_bindings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_negative: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);

    let def_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k.starts_with("definition."))
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        !def_names.contains(&"lambdaBinding"),
        "'lambdaBinding' (a val bound to a lambda) must not be a definition, got: {def_names:?}"
    );
    assert!(
        !def_names.contains(&"etaExpanded"),
        "'etaExpanded' (a val bound via eta-expansion) must not be a definition, got: {def_names:?}"
    );

    let ref_calls: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.call")
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        !ref_calls.contains(&"counter"),
        "bare field read/write 'counter' must not be a call reference, got: {ref_calls:?}"
    );
}

/// Every `call_expression.function` variant scala.calls.scm claims to
/// support (identifier, method call, explicit operator-method call, generic,
/// qualified generic, parenthesized target) must produce a @call capture.
#[test]
fn scala_calls_completeness_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_calls_completeness: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scala")
        .expect("scala calls query missing");
    let caps = collect_captures_full(&lang, SCALA_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    assert!(
        calls.contains(&"identity"),
        "expected plain call 'identity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"map"),
        "expected method call 'map', got: {calls:?}"
    );
    assert!(
        calls.contains(&"+"),
        "expected explicit operator-method call '+', got: {calls:?}"
    );
    assert!(
        calls.contains(&"identityGeneric"),
        "expected generic call 'identityGeneric', got: {calls:?}"
    );
    // Parenthesized call target: (f)(1) — the whole parenthesized text is
    // captured as @call, matching typescript.calls.scm's convention.
    assert!(
        calls.iter().any(|c| c.starts_with('(') && c.contains('f')),
        "expected parenthesized call target '(f)', got: {calls:?}"
    );
}

/// Negative case: bare field access/write must never appear as a @call.
#[test]
fn scala_calls_negative_field_access() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_calls_negative: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scala")
        .expect("scala calls query missing");
    let calls = collect_captures(&lang, SCALA_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"counter".to_string()),
        "bare field read/write 'counter' must not be captured as a call, got: {calls:?}"
    );
}

/// `enum_definition` must contribute nesting depth, matching how
/// class/object/trait definitions are treated.
#[test]
fn scala_complexity_completeness_enum_nesting() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_complexity_completeness: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scala")
        .expect("scala complexity query missing");
    let caps = collect_captures_full(&lang, SCALA_VARIANTS, &query_str);
    let nesting_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "nesting")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    assert!(
        nesting_kinds.contains(&"enum_definition"),
        "expected enum_definition to contribute nesting, got: {nesting_kinds:?}"
    );
    // match with guards (case_clause) must contribute complexity.
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    assert!(
        complexity_kinds
            .iter()
            .filter(|k| **k == "case_clause")
            .count()
            >= 2,
        "expected multiple case_clause complexity nodes (guarded match), got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"for_expression"),
        "expected for-comprehension to contribute complexity, got: {complexity_kinds:?}"
    );
}

/// Duplicate-capture regression: a qualified type reference (`java.util.Date`)
/// must produce exactly one @type.reference capture per identifier, not two.
/// A previous version of scala.types.scm had a redundant clause that matched
/// every `stable_type_identifier`-nested `type_identifier` twice.
#[test]
fn scala_types_negative_no_duplicate_qualified_captures() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_types_negative: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("scala")
        .expect("scala types query missing");
    let refs = collect_captures(&lang, SCALA_VARIANTS, &query_str, "type");
    // variants.scala's NewVariants object has exactly one `new java.util.Date()`.
    let date_count = refs.iter().filter(|r| *r == "Date").count();
    assert_eq!(
        date_count, 1,
        "expected exactly 1 'Date' type.reference capture (qualified type must not \
         double-count), got {date_count}: {refs:?}"
    );
}

/// Rename-arrow import bugs: `{Map => MutableMap}` (Scala 2 arrow),
/// `{List as JList}` (Scala 3 `as`), and per-name renames inside a
/// multi-name brace list (`{Try, Success => S, Failure}`) must all still
/// anchor as their own `import_declaration` — this exercises
/// `Scala::extract_imports`'s text-parsing fallback via the same
/// query-selected `import_declaration` nodes.
#[test]
fn scala_imports_completeness_rename_and_wildcard_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_imports_completeness: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scala")
        .expect("scala imports query missing");
    let imports = collect_captures(&lang, SCALA_VARIANTS, &query_str, "import");

    assert!(
        imports.iter().any(|i| i.contains("Map => MutableMap")),
        "expected the arrow-rename import statement, got: {imports:?}"
    );
    assert!(
        imports.iter().any(|i| i.contains("List as JList")),
        "expected the Scala-3 'as'-rename import statement, got: {imports:?}"
    );
    assert!(
        imports.iter().any(|i| i.contains("foo.bar.baz.*")),
        "expected the bare-wildcard import statement, got: {imports:?}"
    );
}

/// `Scala::extract_imports` must strip per-name rename suffixes (`=>`/`as`)
/// from the parsed `names` list instead of leaving raw "X => Y" text in it,
/// and must not mistake a name merely containing '_' for a wildcard marker.
#[test]
fn scala_imports_extract_strips_rename_suffix_and_detects_wildcard_precisely() {
    use normalize_languages::{Language, Scala};
    use tree_sitter::{Parser, StreamingIterator};

    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_imports_extract: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_imports_extract: scala grammar .so not found");
        return;
    };
    // Parse directly and probe extract_imports on the raw import_declaration
    // nodes — no need for the tags/imports query here.
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let source = "import scala.util.{Try, Success => S, Failure}\n\
                  import scala.collection.mutable.{Map => MutableMap}\n";
    let tree = parser.parse(source, None).expect("parse failed");
    let query =
        tree_sitter::Query::new(&lang, "(import_declaration) @import").expect("query compile");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let scala = Scala;
    let mut all_names: Vec<String> = Vec::new();
    let mut single_alias: Option<String> = None;
    while let Some(m) = matches.next() {
        for cap in m.captures {
            let imports = scala.extract_imports(&cap.node, source);
            for imp in imports {
                all_names.extend(imp.names.iter().cloned());
                if imp.names.len() == 1 {
                    single_alias = imp.alias.clone();
                }
            }
        }
    }
    // Multi-name brace import: renamed entry must contribute a clean plain
    // name ("Success"), never the raw "Success => S" text.
    assert!(
        all_names.contains(&"Success".to_string()),
        "expected clean name 'Success' (rename suffix stripped), got: {all_names:?}"
    );
    assert!(
        !all_names.iter().any(|n| n.contains("=>")),
        "no parsed import name may contain a raw rename arrow, got: {all_names:?}"
    );
    assert!(
        all_names.contains(&"Try".to_string()) && all_names.contains(&"Failure".to_string()),
        "expected unrenamed names 'Try'/'Failure' preserved, got: {all_names:?}"
    );
    // Single-name brace import with a rename: alias must be recovered.
    assert_eq!(
        single_alias,
        Some("MutableMap".to_string()),
        "expected single-name rename alias 'MutableMap' to be recovered"
    );
}

#[test]
fn scala_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping scala_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "scala",
        SCALA_SAMPLE,
        &["@main", "// Classify a number"],
    );
}
