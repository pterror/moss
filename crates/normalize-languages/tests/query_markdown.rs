//! Query fixture tests for markdown.
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
// Markdown
// ---------------------------------------------------------------------------

const MARKDOWN_SAMPLE: &str = include_str!("fixtures/markdown/sample.md");

const MARKDOWN_VARIANTS: &str = include_str!("fixtures/markdown/variants.md");

// --- Dimension 4: real-world fixture coverage (sample.md) -------------------

#[test]
fn markdown_tags_finds_headings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping markdown_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("markdown").ok() else {
        eprintln!("Skipping markdown_tags: markdown grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("markdown")
        .expect("markdown tags query missing");
    let names = collect_captures(&lang, MARKDOWN_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n.contains("Getting Started") || n.contains("Installation")),
        "expected heading names in markdown tags, got: {names:?}"
    );
    // ATX headings under a task-list-bearing section.
    assert!(
        names.contains(&"Roadmap".to_string()),
        "expected 'Roadmap' heading in markdown tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Support".to_string()),
        "expected 'Support' heading in markdown tags, got: {names:?}"
    );
    // Setext-style ("License\n=======") heading in a real-world position:
    // after prose and a link_reference_definition, not as the first block
    // in its section — this is the shape that previously (pre-fix) matched
    // nothing at all.
    assert!(
        names.contains(&"License".to_string()),
        "expected setext-style 'License' heading in markdown tags, got: {names:?}"
    );
}

// --- Dimensions 2+3: query completeness + extraction depth (variants.md) ---

/// Every heading construct in variants.md, in source order, as
/// `(expected_name, expected_definition_container_kind)`. The container kind
/// differs by heading style: ATX headings anchor `@definition.heading` to
/// the enclosing `section` (this grammar always gives an ATX heading its own
/// section); setext headings anchor directly to `setext_heading` (they don't
/// reliably get their own `section` — see markdown.tags.scm).
const MARKDOWN_VARIANT_HEADINGS: &[(&str, &str)] = &[
    ("ATX level 1", "section"),
    ("ATX level 2", "section"),
    ("ATX level 3", "section"),
    ("ATX level 4", "section"),
    ("ATX level 5", "section"),
    ("ATX level 6", "section"),
    ("ATX level 2 with closing sequence ##", "section"),
    ("Setext level 1", "setext_heading"),
    ("Setext level 2", "setext_heading"),
    ("Back to back A", "setext_heading"),
    ("Back to back B", "setext_heading"),
    ("Preceding content", "section"),
    ("Trailing setext divider", "setext_heading"),
    ("Heading inside a block quote", "section"),
    ("Heading inside a list item", "section"),
];

#[test]
fn markdown_tags_completeness_heading_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping markdown_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("markdown").ok() else {
        eprintln!("Skipping markdown_tags_completeness: markdown grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("markdown")
        .expect("markdown tags query missing");
    let captures = collect_captures_full(&lang, MARKDOWN_VARIANTS, &query_str);

    // Every @name capture must be an `inline` node (dimension 3: kind, not
    // just text) and must appear in the expected list exactly once.
    let names: Vec<&(String, String, String, usize)> =
        captures.iter().filter(|(cn, ..)| cn == "name").collect();
    assert_eq!(
        names.len(),
        MARKDOWN_VARIANT_HEADINGS.len(),
        "expected {} heading names in variants.md, got {}: {:?}",
        MARKDOWN_VARIANT_HEADINGS.len(),
        names.len(),
        names
    );
    for (name, kind, text, _line) in &names {
        assert_eq!(name, "name");
        assert_eq!(
            kind, "inline",
            "expected @name capture for {text:?} to be an 'inline' node, got {kind:?}"
        );
    }
    let text_set: Vec<&str> = names.iter().map(|(_, _, t, _)| t.as_str()).collect();
    for (expected_name, _) in MARKDOWN_VARIANT_HEADINGS {
        assert!(
            text_set.contains(expected_name),
            "expected heading {expected_name:?} in variants.md completeness matrix, got: {text_set:?}"
        );
    }

    // Every @definition.heading capture's node kind must match the expected
    // anchor for its heading style (dimension 3: correctness of the anchor
    // decision documented in markdown.tags.scm, not just presence).
    let defs: Vec<&(String, String, String, usize)> = captures
        .iter()
        .filter(|(cn, ..)| cn == "definition.heading")
        .collect();
    assert_eq!(
        defs.len(),
        MARKDOWN_VARIANT_HEADINGS.len(),
        "expected {} @definition.heading captures in variants.md, got {}",
        MARKDOWN_VARIANT_HEADINGS.len(),
        defs.len()
    );
    let section_anchored = defs.iter().filter(|(_, k, ..)| k == "section").count();
    let setext_anchored = defs
        .iter()
        .filter(|(_, k, ..)| k == "setext_heading")
        .count();
    let expected_section = MARKDOWN_VARIANT_HEADINGS
        .iter()
        .filter(|(_, k)| *k == "section")
        .count();
    let expected_setext = MARKDOWN_VARIANT_HEADINGS
        .iter()
        .filter(|(_, k)| *k == "setext_heading")
        .count();
    assert_eq!(
        section_anchored, expected_section,
        "expected {expected_section} section-anchored (ATX) @definition.heading captures, got {section_anchored}"
    );
    assert_eq!(
        setext_anchored, expected_setext,
        "expected {expected_setext} setext_heading-anchored @definition.heading captures, got {setext_anchored}"
    );
}

#[test]
fn markdown_tags_negative_non_headings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping markdown_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("markdown").ok() else {
        eprintln!("Skipping markdown_tags_negative: markdown grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("markdown")
        .expect("markdown tags query missing");
    let names = collect_captures(&lang, MARKDOWN_VARIANTS, &query_str, "name");

    // Exact count: the negative section must contribute zero additional
    // matches beyond the documented heading constructs above it.
    assert_eq!(
        names.len(),
        MARKDOWN_VARIANT_HEADINGS.len(),
        "negative-section constructs produced unexpected @name matches, got: {names:?}"
    );

    // Specific near-miss constructs that must never match:
    // - seven-hash line exceeds the grammar's max heading level (h1-h6 only)
    assert!(
        !names.iter().any(|n| n.contains("seven hashes")),
        "seven-# line must not match as a heading, got: {names:?}"
    );
    // - a pipe-table header row must not match as a heading (also implicitly
    //   confirms the preceding thematic breaks ---, ***, ___ were not
    //   mistaken for setext underlines, since a false match there would
    //   have shifted or duplicated surrounding captures)
    assert!(
        !names.iter().any(|n| n.contains("Not a heading")),
        "pipe-table header row must not match as a heading, got: {names:?}"
    );
    // - fenced/indented code block contents that look like ATX headings
    assert!(
        !names
            .iter()
            .any(|n| n.contains("inside a fenced code block")),
        "fenced code block content must not match as a heading, got: {names:?}"
    );
    assert!(
        !names
            .iter()
            .any(|n| n.contains("inside an indented code block")),
        "indented code block content must not match as a heading, got: {names:?}"
    );
}
