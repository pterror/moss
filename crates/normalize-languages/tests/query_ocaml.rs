//! Query fixture tests for ocaml.
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

const OCAML_SAMPLE: &str = include_str!("fixtures/ocaml/sample.ml");

#[test]
fn ocaml_decorations_finds_attribute_and_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ocaml_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "ocaml",
        OCAML_SAMPLE,
        &["[@inline]", "(** Classify"],
    );
}
