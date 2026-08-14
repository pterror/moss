//! Global check that every registered `.scm` query compiles.
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
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Guardrail: every registered `.scm` query file must compile.
//
// This is the regression test for the class of bug fixed by af9dbc0b: a
// `.scm` file with a structurally-invalid pattern (references a node type
// that doesn't exist in the grammar, or targets syntax that's actually a
// parse error) makes `tree_sitter::Query::new` fail. Every call site that
// consumes these queries (`GrammarLoader::get_compiled_query`, and the many
// production call sites that do `Query::new(..).ok()?`) treats that failure
// as "no query" and silently produces zero results — no panic, no log, no
// error surfaced to the user. `lua.tags.scm` shipped broken for an unknown
// period of time before anything caught it, because no test compiled every
// bundled query against its real grammar; only a hand-picked subset was
// covered by `test_tags_queries_compile` and similar spot checks.
//
// This test is the general form: it walks every `.scm` file physically
// present under `src/queries/` (which — per a one-time audit — is exactly
// the set reachable via `include_str!` from `grammar_loader.rs`, i.e. the
// full "registered" set), asks `GrammarLoader` for that query the same way
// production code does (`get_tags`, `get_calls`, etc.), and compiles it
// against the real grammar with the exact same `tree_sitter::Query::new`
// call production uses. Adding a new `.scm` file automatically gets covered
// — nothing to remember to wire up.
// ---------------------------------------------------------------------------

/// Look up the query source for `(lang, purpose)` via the same public
/// `GrammarLoader` getter that production code calls for that query
/// purpose. Keep this in sync with the query "purposes" the loader
/// supports (tags/calls/complexity/types/imports/decorations/refactor/
/// test_regions/cfg) — a purpose present as a filename suffix under
/// `src/queries/` but missing here fails loudly below rather than being
/// silently skipped.
fn query_source_for(loader: &GrammarLoader, lang: &str, purpose: &str) -> Option<Arc<String>> {
    match purpose {
        "tags" => loader.get_tags(lang),
        "calls" => loader.get_calls(lang),
        "complexity" => loader.get_complexity(lang),
        "types" => loader.get_types(lang),
        "imports" => loader.get_imports(lang),
        "decorations" => loader.get_decorations(lang),
        "refactor" => loader.get_refactor(lang),
        "test_regions" => loader.get_test_regions(lang),
        "cfg" => loader.get_cfg(lang),
        _ => None,
    }
}

#[test]
fn all_registered_queries_compile() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!(
            "Skipping all_registered_queries_compile: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);

    let queries_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/queries");
    let mut entries: Vec<(String, String)> = std::fs::read_dir(&queries_dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", queries_dir.display()))
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let stem = name.strip_suffix(".scm")?;
            let (lang, purpose) = stem.rsplit_once('.')?;
            Some((lang.to_string(), purpose.to_string()))
        })
        .collect();
    entries.sort();

    assert!(
        !entries.is_empty(),
        "no .scm query files found under {} — grammar_dir/CARGO_MANIFEST_DIR probably wrong",
        queries_dir.display()
    );

    let mut failures = Vec::new();
    let mut skipped_no_grammar = Vec::new();
    let mut compiled = 0usize;

    for (lang, purpose) in &entries {
        let query_str = match query_source_for(&loader, lang, purpose) {
            Some(q) => q,
            None => {
                failures.push(format!(
                    "{lang}.{purpose}.scm: file exists on disk but query_source_for() in \
                     this test has no getter mapped for purpose '{purpose}' — add it there"
                ));
                continue;
            }
        };
        let grammar = match loader.get(lang) {
            Ok(g) => g,
            Err(e) => {
                skipped_no_grammar.push(format!("{lang} ({e})"));
                continue;
            }
        };
        match tree_sitter::Query::new(&grammar, &query_str) {
            Ok(_) => compiled += 1,
            Err(e) => {
                failures.push(format!("{lang}.{purpose}.scm: Query::new failed: {e}"));
            }
        }
    }

    if !skipped_no_grammar.is_empty() {
        eprintln!(
            "all_registered_queries_compile: skipped {} language(s) with no compiled grammar \
             .so in the grammar dir (not a query bug, just missing grammar): {}",
            skipped_no_grammar.len(),
            skipped_no_grammar.join(", ")
        );
    }

    assert!(
        failures.is_empty(),
        "{} of {} registered query file(s) failed to compile against their grammar:\n{}",
        failures.len(),
        entries.len(),
        failures.join("\n")
    );

    eprintln!(
        "all_registered_queries_compile: {compiled} of {} registered query files compiled \
         successfully ({} skipped for missing grammar)",
        entries.len(),
        skipped_no_grammar.len()
    );
}
