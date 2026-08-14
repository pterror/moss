//! Query fixture tests for commonlisp.
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
// Common Lisp
// ---------------------------------------------------------------------------

const COMMONLISP_SAMPLE: &str = include_str!("fixtures/commonlisp/sample.lisp");

#[test]
fn commonlisp_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_tags: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("commonlisp")
        .expect("commonlisp tags query missing");
    let names = collect_captures(&lang, COMMONLISP_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "factorial"),
        "expected 'factorial' function in commonlisp tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "point" || n == "shape"),
        "expected 'point' or 'shape' struct/class in commonlisp tags, got: {names:?}"
    );
}

#[test]
fn commonlisp_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_calls: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("commonlisp")
        .expect("commonlisp calls query missing");
    let calls = collect_captures(&lang, COMMONLISP_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "format" || c == "setf" || c == "dolist"),
        "expected a standard form call in commonlisp sample, got: {calls:?}"
    );
}

#[test]
fn commonlisp_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_complexity: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("commonlisp")
        .expect("commonlisp complexity query missing");
    let complexity = collect_captures(&lang, COMMONLISP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in commonlisp sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn commonlisp_imports_finds_require() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_imports: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("commonlisp")
        .expect("commonlisp imports query missing");
    let paths = collect_captures(&lang, COMMONLISP_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("alexandria") || p.contains("iterate")),
        "expected 'alexandria' or 'iterate' in commonlisp import paths, got: {paths:?}"
    );
}

const COMMONLISP_VARIANTS: &str = include_str!("fixtures/commonlisp/variants.lisp");

#[test]
fn commonlisp_tags_completeness_unified_defun_node_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_tags_completeness: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("commonlisp")
        .expect("commonlisp tags query missing");
    let pairs = collect_tag_pairs(&lang, COMMONLISP_VARIANTS, &query_str);

    // defun/defgeneric/defmethod/defmacro all parse as the SAME unified
    // `defun` grammar node, discriminated only by defun_header.keyword's
    // text — the actual bug this test guards: defmethod/defmacro were
    // previously misclassified as @definition.function.
    let expected: &[(&str, &str)] = &[
        ("definition.function", "plain-defun"),
        ("definition.function", "generic-op"), // defgeneric
        ("definition.method", "generic-op"),   // defmethod
        ("definition.macro", "plain-defmacro"),
        ("definition.class", "plain-class"),
        ("definition.class", "plain-struct"),
        ("definition.module", ":plain-package"),
        ("definition.type", "plain-type"),
        ("definition.constant", "+plain-constant+"),
        ("definition.constant", "*plain-parameter*"),
    ];
    for (kind, name) in expected {
        assert!(
            pairs.iter().any(|(k, n)| k == kind && n == name),
            "expected ({kind}, {name}) in commonlisp tags variants, got: {pairs:?}"
        );
    }
    // defmethod with a qualifier (:before) must still resolve to the same
    // definition.method/generic-op pair, not get confused by the qualifier.
    let method_count = pairs
        .iter()
        .filter(|(k, n)| k == "definition.method" && n == "generic-op")
        .count();
    assert_eq!(
        method_count, 2,
        "expected 2 definition.method 'generic-op' entries (plain + :before-qualified), got: {pairs:?}"
    );
    // setf-expander names — function_name: (list_lit), previously
    // uncaptured entirely.
    let setf_count = pairs
        .iter()
        .filter(|(_, n)| *n == "(setf setter-target)")
        .count();
    assert_eq!(
        setf_count, 2,
        "expected 2 setf-expander definitions '(setf setter-target)' (defun + defmethod \
         forms), got: {pairs:?}"
    );
}

#[test]
fn commonlisp_tags_negative_calls_and_locals_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_tags_negative: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("commonlisp")
        .expect("commonlisp tags query missing");
    let pairs = collect_tag_pairs(&lang, COMMONLISP_VARIANTS, &query_str);
    let bad_name = "local-not-a-def";
    assert!(
        !pairs.iter().any(|(_, n)| n == bad_name),
        "'{bad_name}' must not appear as a tags definition, got: {pairs:?}"
    );
}

#[test]
fn commonlisp_imports_completeness_all_forms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping commonlisp_imports_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_imports_completeness: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("commonlisp")
        .expect("commonlisp imports query missing");
    let paths = collect_captures(&lang, COMMONLISP_VARIANTS, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("variants-quoted-dep")),
        "expected quoted require path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("variants-string-dep")),
        "expected string require path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("variants-used-pkg")),
        "expected use-package path, got: {paths:?}"
    );
    // ql:quickload parses as a `package_lit` node, not `sym_lit` — the
    // previous query required node kind sym_lit and so never matched any
    // ql:quickload form at all.
    assert!(
        paths.iter().any(|p| p.contains("variants-quickloaded-pkg")),
        "expected ql:quickload path (package_lit node, not sym_lit), got: {paths:?}"
    );
}

#[test]
fn commonlisp_calls_negative_special_forms_are_not_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_calls_negative: commonlisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("commonlisp")
        .expect("commonlisp calls query missing");
    let calls = collect_captures(&lang, COMMONLISP_VARIANTS, &query_str, "call");
    for special_form in [
        "defun",
        "defgeneric",
        "defmethod",
        "defmacro",
        "defclass",
        "defstruct",
        "defpackage",
        "deftype",
        "defconstant",
        "defparameter",
        "let",
        "dolist",
        "when",
        "if",
        "use-package",
        "require",
    ] {
        assert!(
            !calls.contains(&special_form.to_string()),
            "'{special_form}' is a special form/definition macro, must not appear as \
             @call, got: {calls:?}"
        );
    }
    for real_call in ["format", "print", "slot-value"] {
        assert!(
            calls.contains(&real_call.to_string()),
            "expected real call '{real_call}' still present, got: {calls:?}"
        );
    }
}

#[test]
fn commonlisp_node_name_finds_all_definition_shapes() {
    // node_name() in commonlisp.rs previously ALWAYS returned None — this is
    // the function the actual symbol-extraction pipeline uses for a
    // definition's name (not the tags query's own @name capture text; see
    // normalize_facts::extract::build_symbol_from_def), so every Common
    // Lisp definition was silently dropped from `normalize view`/symbol
    // extraction regardless of query correctness. Verified live via
    // `normalize view <file>.lisp` before and after the fix.
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping commonlisp_node_name: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("commonlisp").ok() else {
        eprintln!("Skipping commonlisp_node_name: commonlisp grammar .so not found");
        return;
    };
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser
        .parse(COMMONLISP_VARIANTS, None)
        .expect("parse failed");
    let support = normalize_languages::CommonLisp;
    let query_str = loader
        .get_tags("commonlisp")
        .expect("commonlisp tags query missing");
    let query = Query::new(&lang, &query_str).expect("query compilation failed");
    let mut qcursor = QueryCursor::new();
    let source_bytes = COMMONLISP_VARIANTS.as_bytes();

    // Mirror real production usage (normalize_facts::extract::
    // collect_symbols_from_tags): node_name() is called on the node bound to
    // the query's @definition.* capture, NOT on a node found by scanning
    // `source`'s direct children — `defun`/`defgeneric`/`defmethod`/
    // `defmacro` forms are nested one level inside an outer `list_lit`
    // wrapper that shares the exact same byte span (confirmed via
    // `normalize syntax query -p ... "(list_lit (defun) @x)"`), so
    // `root.children()` reports their kind as "list_lit", not "defun" — only
    // the tags query's own descendant-matching correctly reaches the inner
    // `defun` node the way the real pipeline does.
    let mut found_defun = false;
    let mut found_setf_defun = false;
    let mut found_defclass = false;
    let mut found_defpackage = false;
    let mut matches = qcursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            if !cap_name.starts_with("definition.") {
                continue;
            }
            let node = cap.node;
            let name =
                normalize_languages::Language::node_name(&support, &node, COMMONLISP_VARIANTS);
            match (node.kind(), name) {
                ("defun", Some("plain-defun")) => found_defun = true,
                ("defun", Some("(setf setter-target)")) => found_setf_defun = true,
                ("list_lit", Some("plain-class")) => found_defclass = true,
                ("list_lit", Some(":plain-package")) => found_defpackage = true,
                _ => {}
            }
        }
    }
    assert!(
        found_defun,
        "node_name did not find 'plain-defun' via defun_header.function_name"
    );
    assert!(
        found_setf_defun,
        "node_name did not find the setf-expander name '(setf setter-target)'"
    );
    assert!(
        found_defclass,
        "node_name did not find 'plain-class' via the plain list_lit path"
    );
    assert!(
        found_defpackage,
        "node_name did not find ':plain-package' (a kwd_lit name, not sym_lit)"
    );
}
