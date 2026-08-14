//! Query fixture tests for elisp.
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
// Emacs Lisp
// ---------------------------------------------------------------------------

const ELISP_SAMPLE: &str = include_str!("fixtures/elisp/sample.el");

#[test]
fn elisp_tags_finds_functions_and_vars() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elisp_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_tags: elisp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("elisp").expect("elisp tags query missing");
    let names = collect_captures(&lang, ELISP_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "sample-greet"),
        "expected 'sample-greet' function in elisp tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "sample-counter"),
        "expected 'sample-counter' var in elisp tags, got: {names:?}"
    );
}

#[test]
fn elisp_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elisp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_calls: elisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("elisp")
        .expect("elisp calls query missing");
    let calls = collect_captures(&lang, ELISP_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "message" || c == "setq" || c == "dolist"),
        "expected a standard form in elisp calls, got: {calls:?}"
    );
}

#[test]
fn elisp_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elisp_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_complexity: elisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elisp")
        .expect("elisp complexity query missing");
    let complexity = collect_captures(&lang, ELISP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in elisp sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn elisp_imports_finds_require() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elisp_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_imports: elisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elisp")
        .expect("elisp imports query missing");
    let paths = collect_captures(&lang, ELISP_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("cl-lib") || p.contains("subr-x")),
        "expected 'cl-lib' or 'subr-x' in elisp import paths, got: {paths:?}"
    );
}

const ELISP_VARIANTS: &str = include_str!("fixtures/elisp/variants.el");

#[test]
fn elisp_complexity_negative_ordinary_lists_are_not_complexity() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elisp_complexity_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_complexity_negative: elisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elisp")
        .expect("elisp complexity query missing");
    // The original bug: a bare `(list) @complexity` counted every
    // parenthesized expression, so a zero-branch function like
    // `(defun variants-zero-branch-body (a b) (+ a b))` produced false
    // @complexity hits for its parameter list `(a b)` and its call
    // `(+ a b)`. Isolate that one function's source and assert zero
    // @complexity captures survive.
    let zero_branch_src = "(defun add-two (a b)\n  (+ a b))\n";
    let complexity = collect_captures(&lang, zero_branch_src, &query_str, "complexity");
    assert_eq!(
        complexity,
        Vec::<String>::new(),
        "a zero-branch function must produce zero @complexity captures, got: {complexity:?}"
    );
}

#[test]
fn elisp_complexity_completeness_special_form_and_list_headed_branches() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elisp_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_complexity_completeness: elisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elisp")
        .expect("elisp complexity query missing");
    let captures = collect_captures_full(&lang, ELISP_VARIANTS, &query_str);

    // special_form-headed forms (if/cond/while/and/or/condition-case): the
    // previous query only matched `(list . (symbol) @_fn ...)`, so these
    // five — all special_form-headed, not list-headed — were entirely
    // invisible to complexity counting.
    for kind_text in [
        ("special_form", "(if (> n 0)"),
        ("special_form", "(cond ((= n 0)"),
        ("special_form", "(while (> n 0)"),
        ("special_form", "(and a (or b nil))"),
        ("special_form", "(condition-case e (error \"x\")"),
    ] {
        assert!(
            captures
                .iter()
                .any(|(cap, kind, text, _)| cap == "complexity"
                    && kind == kind_text.0
                    && text.starts_with(kind_text.1)),
            "expected @complexity capture starting with {:?} of kind {}, got: {captures:?}",
            kind_text.1,
            kind_text.0
        );
    }

    // list-headed forms (when/unless/dolist/dotimes/until/case/pcase/
    // cl-loop/ignore-errors): these already worked before the fix and must
    // keep working.
    for text_prefix in [
        "(when a",
        "(unless b",
        "(dolist (x lst)",
        "(dotimes (i n)",
        "(until (<= n 0)",
        "(case n",
        "(pcase n",
        "(cl-loop for x in lst",
        "(ignore-errors",
    ] {
        assert!(
            captures
                .iter()
                .any(|(cap, kind, text, _)| cap == "complexity"
                    && kind == "list"
                    && text.starts_with(text_prefix)),
            "expected list-headed @complexity capture starting with {text_prefix:?}, got: {captures:?}"
        );
    }
}

#[test]
fn elisp_cfg_completeness_special_form_headed_constructs() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elisp_cfg_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_cfg_completeness: elisp grammar .so not found");
        return;
    };
    let query_str = loader.get_cfg("elisp").expect("elisp cfg query missing");
    let captures = collect_captures_full(&lang, ELISP_VARIANTS, &query_str);

    // Before the fix, `while` (the single most fundamental Lisp loop
    // construct) produced zero @cfg.loop matches because it is
    // special_form-headed, not list-headed like the pre-existing pattern
    // required.
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "cfg.loop"
            && kind == "special_form"
            && text.starts_with("(while")),
        "expected special_form-headed `while` to produce @cfg.loop, got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "cfg.branch"
                && kind == "special_form"
                && text.starts_with("(if")),
        "expected special_form-headed `if` to produce @cfg.branch, got: {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(cap, kind, text, _)| cap == "cfg.match"
                && kind == "special_form"
                && text.starts_with("(cond")),
        "expected special_form-headed `cond` to produce @cfg.match, got: {captures:?}"
    );
    assert!(
        captures.iter().any(|(cap, kind, text, _)| cap == "cfg.try"
            && kind == "special_form"
            && text.starts_with("(condition-case")),
        "expected special_form-headed `condition-case` to produce @cfg.try, got: {captures:?}"
    );
}

#[test]
fn elisp_tags_negative_special_form_internals_are_not_definitions() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elisp_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_tags_negative: elisp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("elisp").expect("elisp tags query missing");
    let pairs = collect_tag_pairs(&lang, ELISP_VARIANTS, &query_str);

    // The original bug: `(special_form . (symbol) @name)` anchored to the
    // first named child of ANY special_form, not just defvar/defconst —
    // fabricating @definition.constant tags for setq's target, a
    // condition-case exception variable, and and/or's leading operand.
    for bogus_name in ["total", "caught-var", "'not-a-definition"] {
        assert!(
            !pairs
                .iter()
                .any(|(kind, name)| kind == "definition.constant" && name == bogus_name),
            "'{bogus_name}' must not be tagged as @definition.constant, got: {pairs:?}"
        );
    }
}

#[test]
fn elisp_tags_completeness_defcustom_and_defclass() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elisp_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_tags_completeness: elisp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("elisp").expect("elisp tags query missing");
    let pairs = collect_tag_pairs(&lang, ELISP_VARIANTS, &query_str);

    // defvar/defconst (special_form-headed) still work after re-anchoring.
    assert!(pairs.contains(&(
        "definition.constant".to_string(),
        "variants-var".to_string()
    )));
    assert!(pairs.contains(&(
        "definition.constant".to_string(),
        "variants-const".to_string()
    )));
    // defcustom (list-headed) was entirely absent before this fix — the
    // single most common "user-facing config variable" idiom in real Emacs
    // Lisp packages.
    assert!(pairs.contains(&(
        "definition.constant".to_string(),
        "variants-custom".to_string()
    )));
    // defclass (EIEIO) alongside cl-defstruct.
    assert!(pairs.contains(&("definition.class".to_string(), "variants-class".to_string())));
    assert!(pairs.contains(&("definition.class".to_string(), "variants-point".to_string())));
}

#[test]
fn elisp_imports_completeness_require_theme() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elisp_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elisp").ok() else {
        eprintln!("Skipping elisp_imports_completeness: elisp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elisp")
        .expect("elisp imports query missing");
    let paths = collect_captures(&lang, ELISP_VARIANTS, &query_str, "import.path");
    // The comment in elisp.imports.scm documented `require-theme` but the
    // code matched "load-theme" instead — require-theme was entirely
    // unmatched. Both forms must now work.
    assert!(
        paths.iter().any(|p| p.contains("modus-themes")),
        "expected require-theme's path in elisp imports, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("modus-vivendi")),
        "expected load-theme's path to still work, got: {paths:?}"
    );
}
