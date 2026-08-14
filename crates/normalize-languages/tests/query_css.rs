//! Query fixture tests for css.
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
use std::sync::Arc;

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

const CSS_SAMPLE: &str = include_str!("fixtures/css/sample.css");

const CSS_VARIANTS: &str = include_str!("fixtures/css/variants.css");

/// `(loader, language, tags, imports)` — see `sql_lang_and_queries`'s doc
/// comment for why the `GrammarLoader` must be kept alongside the
/// `tree_sitter::Language` it produced (it owns the backing `.so`'s
/// `libloading::Library`; dropping it early dangles the language's function
/// pointers).
type CssLangAndQueries = (
    GrammarLoader,
    tree_sitter::Language,
    Arc<String>,
    Arc<String>,
);

fn css_lang_and_queries() -> Option<CssLangAndQueries> {
    let gdir = grammar_dir()?;
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let lang = loader.get("css").ok()?;
    let tags = loader.get_tags("css")?;
    let imports = loader.get_imports("css")?;
    Some((loader, lang, tags, imports))
}

// --- Dimension 4: real-world fixture coverage (sample.css) ------------------

#[test]
fn css_imports_finds_at_import_paths() {
    let Some((_loader, lang, _tags, imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let paths = collect_captures(&lang, CSS_SAMPLE, &imports, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("reset") || p.contains("variables")),
        "expected string @import paths in css sample, got: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .any(|p| p.contains("theme") || p.contains("fonts")),
        "expected url() @import paths in css sample, got: {paths:?}"
    );
}

/// Real-world idioms dimension: attribute/pseudo selectors, combinators,
/// CSS Nesting (`&`), and the generic `at_rule` fallback (@font-face,
/// @layer, @container) all appear in `sample.css` and must all surface as
/// tags — not just the toy `.container`/`.button` rule sets the original
/// fixture had before this pass.
#[test]
fn css_tags_finds_realworld_idioms() {
    let Some((_loader, lang, tags, _imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_tags_realworld: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, CSS_SAMPLE, &tags, "name");
    assert!(
        names.iter().any(|n| n.contains("a[href^=\"https://\"]")),
        "expected attribute selector in css sample tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("input:not([disabled]):focus")),
        "expected chained pseudo-class selector in css sample tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("li:nth-child(2n+1)")),
        "expected pseudo-class-with-arguments selector in css sample tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "& .title" || n == "&:hover"),
        "expected CSS Nesting (`&`) selectors in css sample tags, got: {names:?}"
    );

    let full = collect_captures_full(&lang, CSS_SAMPLE, &tags);
    assert!(
        full.iter()
            .any(|(cap, kind, text, _)| cap == "definition.module"
                && kind == "at_rule"
                && text.starts_with("@font-face")),
        "expected @font-face captured via the generic at_rule fallback, got: {full:?}"
    );
    assert!(
        full.iter()
            .any(|(cap, kind, text, _)| cap == "definition.module"
                && kind == "at_rule"
                && text.starts_with("@layer reset, base")),
        "expected blockless @layer statement captured, got: {full:?}"
    );
    assert!(
        full.iter()
            .any(|(cap, kind, text, _)| cap == "definition.module"
                && kind == "at_rule"
                && text.starts_with("@container")),
        "expected @container captured via the generic at_rule fallback, got: {full:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix (variants.css) --------------------

/// Every at-rule node type this grammar can produce a `@definition.module`
/// for: the three dedicated node types (`media_statement`,
/// `supports_statement`, `scope_statement`) plus every construct that falls
/// through to the generic `at_rule` node (@font-face, blockless `@layer`,
/// block `@layer`, @property, unnamed @container, bare @page, `@page
/// :first` (which internally contains an ERROR node but the outer `at_rule`
/// still matches), and named `@container name (...)` (same — ERROR inside,
/// outer node still matches)). Asserts capture *kind*, not just count, so a
/// future accidental match against some unrelated node type would fail
/// loudly instead of just moving the count.
#[test]
fn css_tags_completeness_definition_module_variants() {
    let Some((_loader, lang, tags, _imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let full = collect_captures_full(&lang, CSS_VARIANTS, &tags);
    let modules: Vec<_> = full
        .iter()
        .filter(|(cap, ..)| cap == "definition.module")
        .collect();

    for kind in ["media_statement", "supports_statement", "scope_statement"] {
        assert!(
            modules.iter().any(|(_, k, ..)| k == kind),
            "expected a definition.module capture of kind '{kind}' in variants.css, got: {modules:?}"
        );
    }

    let at_rule_count = modules.iter().filter(|(_, k, ..)| k == "at_rule").count();
    assert_eq!(
        at_rule_count, 8,
        "expected 8 generic at_rule definition.module captures \
         (@font-face, blockless @layer, block @layer, @property, unnamed \
         @container, bare @page, @page :first, named @container) in \
         variants.css, got {at_rule_count}: {modules:?}"
    );

    // Blockless `@layer` must be captured with its raw text intact (no
    // spurious ` { … }` synthesized anywhere in the query layer — that
    // rendering choice happens in css.rs's build_signature, not here, but
    // the raw captured node text must still be exactly the statement).
    assert!(
        modules
            .iter()
            .any(|(_, k, text, _)| k == "at_rule" && text == "@layer utilities, base;"),
        "expected the blockless @layer statement captured verbatim, got: {modules:?}"
    );
}

#[test]
fn css_tags_completeness_keyframes_and_rule_set_selector_variants() {
    let Some((_loader, lang, tags, _imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_tags_keyframes: run `cargo xtask build-grammars` first");
        return;
    };
    let pairs = collect_tag_pairs(&lang, CSS_VARIANTS, &tags);
    assert!(
        pairs.contains(&(
            "definition.function".to_string(),
            "fade-variant".to_string()
        )),
        "expected @keyframes fade-variant as definition.function, got: {pairs:?}"
    );

    let names = collect_captures(&lang, CSS_VARIANTS, &tags, "name");
    // One representative rule_set per selector node type the grammar's
    // `selectors` list allows (verified via real parse, not node-types.json
    // alone — see variants.css's header comment).
    for expected in [
        ".class-selector",
        "#id-selector",
        "tag-selector",
        "a[data-x]",
        "a[data-x=\"y\"]",
        "a[data-x^=\"y\"]",
        "a[data-x$=\"y\"]",
        "a[data-x*=\"y\"]",
        "a[data-x~=\"y\"]",
        "a[data-x|=\"y\"]",
        ".parent > .child",
        ".a + .b",
        ".a ~ .b",
        ".a .b",
        "svg|circle",
        ":hover",
        ":nth-child(2n+1)",
        "::before",
        "& .nested",
        "&:hover",
        "& > .direct-child",
    ] {
        assert!(
            names.iter().any(|n| n == expected),
            "expected selector variant '{expected}' as a rule_set name in \
             variants.css, got: {names:?}"
        );
    }
}

/// Custom properties (`--foo`) and standard properties must both be
/// captured as `definition.var`, with the leading `--` preserved verbatim
/// in the name (a truncation or trim bug here would silently corrupt every
/// custom-property symbol name).
#[test]
fn css_tags_finds_custom_and_standard_properties() {
    let Some((_loader, lang, tags, _imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_tags_properties: run `cargo xtask build-grammars` first");
        return;
    };
    let pairs = collect_tag_pairs(&lang, CSS_VARIANTS, &tags);
    assert!(
        pairs.contains(&("definition.var".to_string(), "--custom-prop".to_string())),
        "expected --custom-prop as definition.var, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "color".to_string())),
        "expected standard 'color' property as definition.var, got: {pairs:?}"
    );
}

// --- Negative cases -----------------------------------------------------

/// @charset and @namespace each have their own dedicated node type
/// (`charset_statement`/`namespace_statement`) that neither query captures
/// — confirm the generic `(at_rule) @definition.module` pattern does NOT
/// accidentally sweep them in (it shouldn't, since they're structurally
/// distinct node types, but this guards against a future grammar upgrade
/// collapsing them into `at_rule`).
#[test]
fn css_tags_negative_charset_and_namespace_not_captured() {
    let Some((_loader, lang, tags, _imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let full = collect_captures_full(&lang, CSS_VARIANTS, &tags);
    assert!(
        !full
            .iter()
            .any(|(_, kind, ..)| kind == "charset_statement" || kind == "namespace_statement"),
        "expected @charset/@namespace to never be captured (no query handles \
         their dedicated node types), got: {full:?}"
    );
}

/// A `url(...)` inside an ordinary declaration value (e.g.
/// `background: url(...)`) is not an `@import` and must never leak into
/// `@import.path` — nor must the text of a comment that merely mentions
/// `@import`.
#[test]
fn css_imports_negative_declaration_url_and_comment_not_import() {
    let Some((_loader, lang, _tags, imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let paths = collect_captures(&lang, CSS_VARIANTS, &imports, "import.path");
    assert!(
        !paths.iter().any(|p| p.contains("not-an-import")),
        "expected background: url(...) to never match @import.path, got: {paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p.contains("inside-a-comment")),
        "expected a commented-out @import to never match @import.path, got: {paths:?}"
    );
}

#[test]
fn css_imports_completeness_all_clean_variants() {
    let Some((_loader, lang, _tags, imports)) = css_lang_and_queries() else {
        eprintln!("Skipping css_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let paths = collect_captures(&lang, CSS_VARIANTS, &imports, "import.path");
    for expected in [
        "bare-string.css",
        "url-string.css",
        "bare-url.css",
        "with-media.css",
    ] {
        assert!(
            paths.iter().any(|p| p.contains(expected)),
            "expected import path variant '{expected}' in variants.css, got: {paths:?}"
        );
    }
}
