//! Query fixture tests for html.
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
// HTML
// ---------------------------------------------------------------------------

const HTML_SAMPLE: &str = include_str!("fixtures/html/sample.html");

const HTML_VARIANTS: &str = include_str!("fixtures/html/variants.html");

// --- Dimension 4: real-world fixture coverage (sample.html) ----------------

#[test]
fn html_tags_finds_elements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping html_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("html").ok() else {
        eprintln!("Skipping html_tags: html grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("html").expect("html tags query missing");
    let pairs = collect_tag_pairs(&lang, HTML_SAMPLE, &query_str);
    let names: Vec<&str> = pairs.iter().map(|(_, n)| n.as_str()).collect();
    // Void elements without a self-closing slash (link, img, input, br,
    // meta) must still be found via the plain element/start_tag pattern.
    for tag in ["link", "img", "input", "br", "meta"] {
        assert!(
            names.contains(&tag),
            "expected void element '{tag}' in html sample tags, got: {names:?}"
        );
    }
    // script_element and style_element are distinct node kinds from a plain
    // `element`; both must still surface a @definition.var named after the
    // tag.
    assert!(
        names.iter().filter(|n| **n == "script").count() >= 3,
        "expected at least 3 <script> definitions (plain, src, type=module) in html sample, \
         got: {names:?}"
    );
    assert!(
        names.contains(&"style"),
        "expected 'style' element definition in html sample, got: {names:?}"
    );
    // Nested containers (header > nav > ul > li > a) must all surface —
    // refine_kind (Rust-side) is what promotes container elements to
    // Module, but the query itself must find every level.
    for tag in ["header", "nav", "ul", "li", "a", "section", "footer"] {
        assert!(
            names.contains(&tag),
            "expected nested element '{tag}' in html sample tags, got: {names:?}"
        );
    }
    // Every match must be tagged @definition.var (html.tags.scm's only
    // definition kind; refine_kind reclassifies Module vs Variable later).
    assert!(
        pairs.iter().all(|(k, _)| k == "definition.var"),
        "expected every html tag capture to be @definition.var, got: {pairs:?}"
    );
}

#[test]
fn html_imports_finds_real_world_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping html_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("html").ok() else {
        eprintln!("Skipping html_imports: html grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("html")
        .expect("html imports query missing");
    let paths = collect_captures(&lang, HTML_SAMPLE, &query_str, "import.path");
    // <script src="app.js"> (quoted) and <script src=vendor.js> (unquoted)
    for expect in ["app.js", "vendor.js"] {
        assert!(
            paths.iter().any(|p| p.contains(expect)),
            "expected script src '{expect}' among html import paths, got: {paths:?}"
        );
    }
    // <link href="styles.css"> (quoted) and <link href=print.css> (unquoted)
    for expect in ["styles.css", "print.css"] {
        assert!(
            paths.iter().any(|p| p.contains(expect)),
            "expected link href '{expect}' among html import paths, got: {paths:?}"
        );
    }
    // <img src="hero.png"> (quoted), <img src=thumb.png> (unquoted), and
    // <img src="icon.svg" /> (self-closing) — img was entirely unhandled
    // before this fix.
    for expect in ["hero.png", "thumb.png", "icon.svg"] {
        assert!(
            paths.iter().any(|p| p.contains(expect)),
            "expected img src '{expect}' among html import paths, got: {paths:?}"
        );
    }
    // Inline <script>/<style> blocks (no src) and the embedded ES module
    // import (`import { init } from "./init.js"`) live in a different
    // grammar (JavaScript, extracted separately via LanguageEmbedded) — the
    // HTML-level query must not see into raw_text content at all.
    assert!(
        !paths.iter().any(|p| p.contains("init.js")),
        "expected the embedded JS module import 'init.js' to NOT be captured by \
         html.imports.scm (that's JavaScript's grammar, reached via LanguageEmbedded, \
         not HTML's), got: {paths:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.html)

/// Every grammar-legal shape html.tags.scm claims to support — plain leaf
/// element, nested element, self-closing void element, start-tag-only void
/// element, script_element, and style_element — must produce a
/// @definition.var with the correct tag_name text, and the *container* node
/// kind (the thing actually promoted to Module/Variable downstream) must be
/// exactly one of element/script_element/style_element.
#[test]
fn html_tags_completeness_all_element_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping html_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("html").ok() else {
        eprintln!("Skipping html_tags_completeness: html grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("html").expect("html tags query missing");
    let caps = collect_captures_full(&lang, HTML_VARIANTS, &query_str);

    let container_kinds: std::collections::HashSet<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "definition.var")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    for kind in ["element", "script_element", "style_element"] {
        assert!(
            container_kinds.contains(kind),
            "expected a @definition.var container of kind '{kind}' in html variants, \
             got container kinds: {container_kinds:?}"
        );
    }

    let names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // @name is always a `tag_name` node regardless of container variant.
    assert!(
        caps.iter()
            .filter(|(cn, _, _, _)| cn == "name")
            .all(|(_, k, _, _)| k == "tag_name"),
        "expected every @name capture to be a tag_name node, got: {caps:?}"
    );
    // plain leaf (`<span>leaf</span>`) + nested (`<div><span>nested</span></div>`)
    assert!(
        names.iter().filter(|n| **n == "span").count() >= 2,
        "expected 'span' from both the leaf and nested constructs, got: {names:?}"
    );
    assert!(names.contains(&"div"), "expected 'div', got: {names:?}");
    // self-closing void element
    assert!(names.contains(&"br"), "expected 'br', got: {names:?}");
    // start-tag-only void element (no self_closing_tag, no end_tag)
    assert!(names.contains(&"input"), "expected 'input', got: {names:?}");
    // script_element: plain, type="module", src (quoted), src (unquoted)
    assert!(
        names.iter().filter(|n| **n == "script").count() >= 4,
        "expected 4 <script> variants in html variants, got: {names:?}"
    );
    // style_element
    assert!(names.contains(&"style"), "expected 'style', got: {names:?}");
    // link/img appear in every href/src quoting + self-closing combination;
    // tags.scm doesn't filter by tag name at all, so case variants (LINK,
    // Link, IMG, SCRIPT) must surface with their source casing preserved.
    assert!(
        names.iter().filter(|n| **n == "link").count() >= 4,
        "expected 4 lowercase 'link' variants, got: {names:?}"
    );
    assert!(
        names.iter().filter(|n| **n == "img").count() >= 4,
        "expected 4 lowercase 'img' variants, got: {names:?}"
    );
    assert!(
        names.contains(&"LINK"),
        "expected uppercase 'LINK', got: {names:?}"
    );
    assert!(
        names.contains(&"Link"),
        "expected mixed-case 'Link', got: {names:?}"
    );
    assert!(
        names.contains(&"IMG"),
        "expected uppercase 'IMG', got: {names:?}"
    );
    assert!(
        names.contains(&"SCRIPT"),
        "expected uppercase 'SCRIPT', got: {names:?}"
    );
}

/// Comments, entities, erroneous end tags, and doctype must never produce a
/// tags.scm match — none of these node kinds are structurally reachable
/// from the query's element/start_tag/self_closing_tag/script_element/
/// style_element patterns, but this asserts it rather than assuming it.
#[test]
fn html_tags_negative_non_element_constructs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping html_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("html").ok() else {
        eprintln!("Skipping html_tags_negative: html grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("html").expect("html tags query missing");
    let caps = collect_captures_full(&lang, HTML_VARIANTS, &query_str);

    // The stray `</bogus-close>` in the negative section must never surface
    // as a @name — its name lives in an `erroneous_end_tag_name` node, which
    // is not `tag_name` and is not reachable from any query pattern.
    // (Container-node text legitimately contains "bogus-close" as a text
    // substring, since `<section>...</bogus-close>...</section>` is one
    // @definition.var match — the real assertion is scoped to @name.)
    assert!(
        !caps
            .iter()
            .any(|(cn, _, t, _)| cn == "name" && t.contains("bogus-close")),
        "erroneous_end_tag's name must never surface as an @name capture, got: {caps:?}"
    );
    // No definition.var container may be of kind 'comment', 'doctype',
    // 'entity', or 'erroneous_end_tag'.
    for bad_kind in ["comment", "doctype", "entity", "erroneous_end_tag"] {
        assert!(
            !caps
                .iter()
                .any(|(cn, k, _, _)| cn == "definition.var" && k == bad_kind),
            "html.tags.scm must never produce a @definition.var of kind '{bad_kind}'"
        );
    }
}

/// Every grammar-legal reference-attribute shape html.imports.scm claims to
/// support (script[src], link[href], img[src]; quoted vs unquoted value;
/// void vs self-closing tag syntax; case-insensitive tag/attribute names)
/// must produce a correctly-shaped @import.path, with the capture *kind*
/// distinguishing quoted_attribute_value from attribute_value.
#[test]
fn html_imports_completeness_all_reference_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping html_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("html").ok() else {
        eprintln!("Skipping html_imports_completeness: html grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("html")
        .expect("html imports query missing");
    let caps = collect_captures_full(&lang, HTML_VARIANTS, &query_str);
    let path_caps: Vec<&(String, String, String, usize)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.path")
        .collect();

    // (text, expected_kind) — expected_kind distinguishes quoted vs unquoted
    // per the fixture's own naming convention.
    let required: &[(&str, &str)] = &[
        ("quoted.js", "quoted_attribute_value"),
        ("unquoted.js", "attribute_value"),
        ("quoted.css", "quoted_attribute_value"),
        ("unquoted.css", "attribute_value"),
        ("quoted-sc.css", "quoted_attribute_value"),
        ("unquoted-sc.css", "attribute_value"),
        ("quoted.png", "quoted_attribute_value"),
        ("unquoted.png", "attribute_value"),
        ("quoted-sc.png", "quoted_attribute_value"),
        ("unquoted-sc.png", "attribute_value"),
        ("upper-link.css", "quoted_attribute_value"),
        ("upper-img.png", "quoted_attribute_value"),
        ("upper-script.js", "quoted_attribute_value"),
        ("mixed-case.css", "quoted_attribute_value"),
    ];
    for (text, kind) in required {
        assert!(
            path_caps
                .iter()
                .any(|(_, k, t, _)| k == kind && t.contains(text)),
            "expected @import.path capture (kind={kind}, text contains '{text}') in html \
             variants, got: {path_caps:?}"
        );
    }
}

/// Constructs that must NOT produce an @import: elements missing the
/// reference attribute entirely (inline script, href-less link, src-less
/// img), and reference-bearing tags outside html.imports.scm's documented
/// scope (`<a href>` is navigation not a resource load; `<iframe src>` is a
/// real resource load but not yet in scope — see the .scm file's comments).
#[test]
fn html_imports_negative_out_of_scope_and_missing_attrs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping html_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("html").ok() else {
        eprintln!("Skipping html_imports_negative: html grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("html")
        .expect("html imports query missing");
    let paths = collect_captures(&lang, HTML_VARIANTS, &query_str, "import.path");
    for forbidden in ["example.com", "frame.html"] {
        assert!(
            !paths.iter().any(|p| p.contains(forbidden)),
            "'{forbidden}' must not appear as an html import path (a[href] and iframe[src] \
             are outside html.imports.scm's documented scope), got: {paths:?}"
        );
    }
}
