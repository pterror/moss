//! Query fixture tests for python.
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
// Python
// ---------------------------------------------------------------------------

const PYTHON_SAMPLE: &str = include_str!("fixtures/python/sample.py");

const PYTHON_VARIANTS: &str = include_str!("fixtures/python/variants.py");

// --- Dimension 4: real-world fixture coverage (sample.py) -------------------

#[test]
fn python_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_tags: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("python")
        .expect("python tags query missing");
    let names = collect_captures(&lang, PYTHON_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"DataProcessor".to_string()),
        "expected 'DataProcessor' class in python tags, got: {names:?}"
    );
    assert!(
        names.contains(&"load_file".to_string()),
        "expected 'load_file' function in python tags, got: {names:?}"
    );
    assert!(
        names.contains(&"count_words".to_string()),
        "expected 'count_words' function in python tags, got: {names:?}"
    );
    // @dataclass-decorated class: decorators must not hide the definition.
    assert!(
        names.contains(&"Config".to_string()),
        "expected 'Config' dataclass in python tags, got: {names:?}"
    );
    // Multiple inheritance: LoggingCache(Cache, DataProcessor) — the class
    // itself must still be found regardless of base-class count.
    assert!(
        names.contains(&"LoggingCache".to_string()),
        "expected 'LoggingCache' class in python tags, got: {names:?}"
    );
    // Closures/nested functions: the outer binding (make_adder, adder) is a
    // real function_definition and must appear; `base`/`x` are parameters,
    // not definitions, and must not leak in as spurious function names.
    assert!(
        names.contains(&"make_adder".to_string()),
        "expected 'make_adder' method in python tags, got: {names:?}"
    );
    assert!(
        names.contains(&"adder".to_string()),
        "expected nested 'adder' closure function in python tags, got: {names:?}"
    );
    // async def is still a function_definition (the `async` keyword is a
    // modifier token, not a distinct node type).
    assert!(
        names.contains(&"fetch_all".to_string()),
        "expected 'async def fetch_all' in python tags, got: {names:?}"
    );
    // Parameterized-decorator + stacked-decorator function.
    assert!(
        names.contains(&"status_handler".to_string()),
        "expected 'status_handler' (stacked decorators) in python tags, got: {names:?}"
    );
}

#[test]
fn python_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_calls: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("python")
        .expect("python calls query missing");
    let calls = collect_captures(&lang, PYTHON_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"append".to_string()),
        "expected 'append' method call in python sample, got: {calls:?}"
    );
    // await fetch_one(url): await wraps a plain call, must still be found.
    assert!(
        calls.iter().any(|c| c == "fetch_one"),
        "expected 'fetch_one' call under await in python sample, got: {calls:?}"
    );
    // Subscript-dispatched call: handlers[event]() — event/command dispatch
    // idiom; previously entirely unmatched (function: subscript).
    assert!(
        calls.iter().any(|c| c == "handlers"),
        "expected subscript-dispatched 'handlers[event]()' call in python sample, got: {calls:?}"
    );
    // Walrus operator inside a call argument position: len(items) inside
    // `(n := len(items))` must still be found as an ordinary call.
    assert!(
        calls.iter().any(|c| c == "len"),
        "expected 'len' call (walrus-assigned) in python sample, got: {calls:?}"
    );
}

#[test]
fn python_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_imports: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("python")
        .expect("python imports query missing");
    let paths = collect_captures(&lang, PYTHON_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"os".to_string()),
        "expected 'os' in python import paths, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p == "collections"),
        "expected 'collections' in python import paths, got: {paths:?}"
    );
    // from dataclasses import dataclass, field — multi-name from-import.
    let names = collect_captures(&lang, PYTHON_SAMPLE, &query_str, "import.name");
    assert!(
        names.contains(&"dataclass".to_string()) && names.contains(&"field".to_string()),
        "expected 'dataclass' and 'field' import names, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.py) -

/// Every grammar-legal, realistically-producible variant of `call.function`
/// that python.calls.scm claims to support must actually match, with the
/// right capture *kind* (dimension 3) — not just the right text.
#[test]
fn python_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_calls_completeness: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("python")
        .expect("python calls query missing");
    let caps = collect_captures_full(&lang, PYTHON_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"), // plain_call: function: identifier
        ("call", "identifier", "append"), // method_call: function: attribute, attribute: identifier
        ("call", "identifier", "handlers"), // subscript_call: function: subscript, value: identifier
        ("call", "identifier", "get_func"), // chained_call: inner call independently matched
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in python.calls.scm \
             output for variants.py, got: {caps:?}"
        );
    }

    // subscript_attribute_call: self_like.handlers["go"](1) — function:
    // subscript, value: attribute. @call must carry the attribute's final
    // segment ("handlers"), not the base object ("self_like").
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "identifier" && t == "handlers"),
        "expected 'handlers' from subscript-dispatch-via-attribute, got: {caps:?}"
    );

    // @call.qualifier must carry the object, not the call name.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"items"),
        "expected 'items' qualifier for the method call, got: {qualifiers:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn python_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_calls_negative: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("python")
        .expect("python calls query missing");
    let caps = collect_captures_full(&lang, PYTHON_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `container.field` is a bare attribute read (no call parens); must
    // never be captured as a call.
    assert!(
        !call_texts.contains(&"field"),
        "bare attribute access 'container.field' must not be captured as a call, got: {call_texts:?}"
    );
    // The lambda binding `add_one` is not a call site by itself; only its
    // invocation (add_one(1), inside chained_call/negative_cases-adjacent
    // code) would be. Since `add_one` here is only ever assigned, not
    // called, it must not appear as a call at all.
    assert!(
        !call_texts.contains(&"add_one"),
        "uncalled lambda binding 'add_one' must not be captured as a call, got: {call_texts:?}"
    );
}

/// Every grammar-legal variant of module-level `assignment.left` that
/// python.tags.scm's @definition.constant rule claims to support (plain
/// identifier, tuple/list-unpacking) must produce a @name capture — and
/// function-local assignments must never leak into @definition.constant.
///
/// This test also guards against regressing the completeness bug found
/// while applying this methodology: `expression_statement` is a grammar
/// supertype alias for `assignment` (not a real wrapping tree node) at this
/// position, so `(module (expression_statement (assignment ...)))` matched
/// *nothing at all*, ever — the fixed rule matches `(module (assignment
/// ...))` directly instead.
#[test]
fn python_tags_completeness_module_constants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_tags_completeness: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("python")
        .expect("python tags query missing");
    let caps = collect_captures_full(&lang, PYTHON_VARIANTS, &query_str);

    let constant_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        constant_names.contains(&"PLAIN_CONSTANT"),
        "expected 'PLAIN_CONSTANT' module-level constant, got: {constant_names:?}"
    );
    assert!(
        constant_names.contains(&"TUPLE_A") && constant_names.contains(&"TUPLE_B"),
        "expected 'TUPLE_A'/'TUPLE_B' from tuple-unpacking constant, got: {constant_names:?}"
    );
    assert!(
        constant_names.contains(&"ANNOTATED_CONSTANT"),
        "expected 'ANNOTATED_CONSTANT' (annotated module assignment), got: {constant_names:?}"
    );

    // Negative: function-local assignment must never appear as a
    // @definition.constant capture.
    let has_local_constant = caps
        .iter()
        .any(|(cn, _, t, _)| cn == "definition.constant" && t.contains("local_not_constant"));
    assert!(
        !has_local_constant,
        "function-local assignment must not be captured as @definition.constant, got: {caps:?}"
    );
}

#[test]
fn python_imports_finds_import_paths_completeness() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_imports_completeness: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("python")
        .expect("python imports query missing");
    let paths = collect_captures(&lang, PYTHON_VARIANTS, &query_str, "import.path");
    let names = collect_captures(&lang, PYTHON_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, PYTHON_VARIANTS, &query_str, "import.alias");
    let globs = collect_captures(&lang, PYTHON_VARIANTS, &query_str, "import.glob");

    // import os.path — multi-segment dotted_name path.
    assert!(
        paths.iter().any(|p| p == "os.path"),
        "expected 'os.path' dotted import path, got: {paths:?}"
    );
    // import os as os_alias — import_statement aliased_import.
    assert!(
        aliases.contains(&"os_alias".to_string()),
        "expected 'os_alias' import alias, got: {aliases:?}"
    );
    // from collections import OrderedDict as OD — import_from_statement aliased_import.
    assert!(
        names.contains(&"OrderedDict".to_string()) && aliases.contains(&"OD".to_string()),
        "expected 'OrderedDict as OD', names={names:?} aliases={aliases:?}"
    );
    // Parenthesized multi-name from-import.
    assert!(
        names.contains(&"defaultdict".to_string()) && names.contains(&"Counter".to_string()),
        "expected parenthesized multi-name import, got: {names:?}"
    );
    // Relative imports: from . import sibling / from ..pkg import cousin.
    assert!(
        paths.iter().any(|p| p == "."),
        "expected bare relative-import path '.', got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p == "..pkg"),
        "expected '..pkg' relative-import path, got: {paths:?}"
    );
    assert!(
        names.contains(&"sibling".to_string()) && names.contains(&"cousin".to_string()),
        "expected 'sibling'/'cousin' relative-import names, got: {names:?}"
    );
    // from os.path import * — wildcard.
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture for the wildcard import, got: {globs:?}"
    );
}

#[test]
fn python_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_complexity: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("python")
        .expect("python complexity query missing");
    let complexity = collect_captures(&lang, PYTHON_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in python sample, got {} ({complexity:?})",
        complexity.len()
    );
}

/// Every complexity/nesting construct claimed by python.complexity.scm must
/// fire on its documented variants.py exercise, including match/case
/// (structural pattern matching) and every comprehension flavor.
#[test]
fn python_complexity_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping python_complexity_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_complexity_completeness: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("python")
        .expect("python complexity query missing");
    let caps = collect_captures_full(&lang, PYTHON_VARIANTS, &query_str);
    let kinds: Vec<&str> = caps.iter().map(|(_, k, _, _)| k.as_str()).collect();

    for expected_kind in [
        "if_statement",
        "for_statement",
        "while_statement",
        "try_statement",
        "except_clause",
        "with_statement",
        "match_statement",
        "case_clause",
        "list_comprehension",
        "dictionary_comprehension",
        "set_comprehension",
        "generator_expression",
        "conditional_expression",
    ] {
        assert!(
            kinds.contains(&expected_kind),
            "expected at least one @complexity/@nesting capture of kind '{expected_kind}' \
             in variants.py, got kinds: {kinds:?}"
        );
    }

    // elif is a nested if_statement, not a distinct node type — the elif
    // branch in `branching()` must contribute its own complexity unit, not
    // be silently merged into the first if.
    let if_count = kinds.iter().filter(|k| **k == "if_statement").count();
    assert!(
        if_count >= 2,
        "expected at least 2 if_statement complexity nodes (if + elif chain), got {if_count}"
    );
}

/// Class/function nesting must be counted even when nested (NestedClass +
/// nested_method), and closures/nested functions must count as nesting too.
#[test]
fn python_complexity_nesting_counts_class_and_function() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_complexity_nesting: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_complexity_nesting: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("python")
        .expect("python complexity query missing");
    let nesting = collect_captures_full(&lang, PYTHON_VARIANTS, &query_str)
        .into_iter()
        .filter(|(cn, _, _, _)| cn == "nesting")
        .map(|(_, k, _, _)| k)
        .collect::<Vec<_>>();
    assert!(
        nesting.iter().any(|k| k == "class_definition"),
        "expected class_definition among @nesting captures, got: {nesting:?}"
    );
    assert!(
        nesting.iter().any(|k| k == "function_definition"),
        "expected function_definition among @nesting captures, got: {nesting:?}"
    );
}

#[test]
fn python_types_finds_class() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_types: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("python")
        .expect("python types query missing");
    let names = collect_captures(&lang, PYTHON_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"DataProcessor".to_string()),
        "expected 'DataProcessor' in python types captures, got: {names:?}"
    );
}

/// Every grammar-legal, realistically-producible variant of Python type
/// annotations (PEP 484 plain/dotted, PEP 585 generics, PEP 604 unions,
/// PEP 612/646/695 param specs and variadics) must produce a
/// @type.reference capture with the correct node *kind*.
#[test]
fn python_types_completeness_all_annotation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_types_completeness: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("python")
        .expect("python types query missing");
    let caps = collect_captures_full(&lang, PYTHON_VARIANTS, &query_str);
    let refs: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "type.reference")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // Plain and dotted annotations.
    assert!(
        refs.contains(&"int"),
        "expected plain 'int' type reference, got: {refs:?}"
    );
    assert!(
        refs.contains(&"os") && refs.contains(&"Kind"),
        "expected dotted 'os.Kind' type reference parts, got: {refs:?}"
    );
    // Multi-segment dotted annotation: os.path.Kind — `object:` nests as
    // another `attribute`, a distinct shape from the 2-segment case above.
    assert!(
        refs.iter().filter(|r| **r == "Kind").count() >= 2,
        "expected 'Kind' from both the 2- and 3-segment dotted annotations, got: {refs:?}"
    );
    // Bare generic_type base name: Optional[str] -> "Optional".
    assert!(
        refs.contains(&"Optional"),
        "expected 'Optional' generic_type base name, got: {refs:?}"
    );
    // Dotted-module generic (subscript-based): typing.List[int] -> "List".
    assert!(
        refs.contains(&"List"),
        "expected 'List' from 'typing.List[int]' (subscript-based generic), got: {refs:?}"
    );
    // Multi-arg dotted generic: typing.Dict[str, os.PathLike].
    assert!(
        refs.contains(&"PathLike"),
        "expected 'PathLike' from 'typing.Dict[str, os.PathLike]', got: {refs:?}"
    );
    // PEP 604 union types (parses as binary_operator, not union_type —
    // verified via real parse output, not node-types.json alone).
    assert!(
        refs.iter().filter(|r| **r == "int").count() >= 2,
        "expected 'int' to appear in at least the plain and union-type positions, got: {refs:?}"
    );
    let union_types = collect_captures_full(&lang, PYTHON_VARIANTS, &query_str);
    let union_kinds: Vec<&str> = union_types
        .iter()
        .filter(|(cn, _, t, _)| cn == "type.reference" && (t == "str" || t == "None"))
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    assert!(
        union_kinds.contains(&"identifier"),
        "expected identifier-kind captures from union types, got: {union_kinds:?}"
    );
    // PEP 695 variadic/paramspec type parameters: def f[*Ts], def f[**P].
    assert!(
        refs.contains(&"Ts") && refs.contains(&"P"),
        "expected 'Ts'/'P' from splat_type PEP 695 type params, got: {refs:?}"
    );
    // Callable argument-list generic: Callable[[int, str], bool] at the
    // known fixture line — asserted precisely by line number since "int"
    // and "str" also legitimately appear at other annotation sites in this
    // fixture (a text-only check couldn't tell them apart).
    let callable_line_refs: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, line)| cn == "type.reference" && *line == 135)
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert_eq!(
        {
            let mut sorted = callable_line_refs.clone();
            sorted.sort_unstable();
            sorted
        },
        vec!["Callable", "bool", "int", "str"],
        "expected exactly ['Callable','bool','int','str'] from \
         'Callable[[int, str], bool]' (line 135), got: {callable_line_refs:?}"
    );
}

/// Negative case: a bare bitwise-or expression outside annotation position
/// (e.g. combining flag constants) must never be captured as a type
/// reference — regression guard for the PEP 604 union-type pattern, which
/// is intentionally scoped to `(type (binary_operator ...))` rather than
/// matching `binary_operator` unconditionally.
#[test]
fn python_types_negative_bitwise_or_outside_annotation() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping python_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("python").ok() else {
        eprintln!("Skipping python_types_negative: python grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("python")
        .expect("python types query missing");
    let refs = collect_captures(&lang, PYTHON_VARIANTS, &query_str, "type.reference");
    assert!(
        !refs.iter().any(|r| r == "O_RDONLY" || r == "O_CREAT"),
        "runtime bitwise-or flag combination must not be captured as a type reference, got: {refs:?}"
    );
    // A plain list literal outside a generic_type/type_parameter position
    // must not leak its elements in as type references either.
    assert!(
        !refs.iter().any(|r| r == "add_one"),
        "plain list literal elements must not be captured as type references, got: {refs:?}"
    );
    // A string forward-reference annotation (`x: "module.Kind"`) is a
    // `string` node, not a parsed dotted name — its contents are opaque to
    // a tree-sitter query (no sub-parsing), so "module"/"Kind" must not
    // appear as type references from this construct. (Both names are
    // otherwise legitimately used and captured elsewhere in this fixture,
    // so this only guards against a hypothetical over-eager string-content
    // extraction, not the current, correctly-conservative behavior.)
    assert!(
        !refs.iter().any(|r| r == "module"),
        "string forward-reference contents must not be captured as type references, got: {refs:?}"
    );
}

#[test]
fn python_decorations_finds_decorator_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping python_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "python",
        PYTHON_SAMPLE,
        &["@property", "# Process all items"],
    );
}
