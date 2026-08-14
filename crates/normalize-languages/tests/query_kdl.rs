//! Query fixture tests for kdl.
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
// KDL
// ---------------------------------------------------------------------------

const KDL_SAMPLE: &str = include_str!("fixtures/kdl/sample.kdl");

const KDL_VARIANTS: &str = include_str!("fixtures/kdl/variants.kdl");

// --- Dimension 4: real-world fixture coverage (sample.kdl) ------------------

#[test]
fn kdl_tags_finds_containers_and_leaves() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kdl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kdl").ok() else {
        eprintln!("Skipping kdl_tags: kdl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("kdl").expect("kdl tags query missing");
    let pairs = collect_tag_pairs(&lang, KDL_SAMPLE, &query_str);

    // Container nodes (have a `{ ... }` children block) -> @definition.class.
    for container in ["package", "dependencies", "dev-dependencies", "config"] {
        assert!(
            pairs.contains(&("definition.class".to_string(), container.to_string())),
            "expected container node '{container}' as @definition.class in kdl tags, \
             got: {pairs:?}"
        );
    }
    // Leaf nodes (no children block) -> @definition.var.
    for leaf in [
        "name",
        "version",
        "serde",
        "tokio",
        "criterion",
        "empty-node",
    ] {
        assert!(
            pairs.contains(&("definition.var".to_string(), leaf.to_string())),
            "expected leaf node '{leaf}' as @definition.var in kdl tags, got: {pairs:?}"
        );
    }
    // Quoted node name inside dependencies.
    assert!(
        pairs.contains(&("definition.var".to_string(), "\"kdl-rs\"".to_string())),
        "expected quoted leaf node '\"kdl-rs\"' as @definition.var in kdl tags, \
         got: {pairs:?}"
    );
    // Typed leaf value inside config: `port (u16)8080` — the node itself
    // (`port`) is untyped; the (u16) annotation is on the *value*, not the
    // node, so this is still a plain @definition.var on `port`.
    assert!(
        pairs.contains(&("definition.var".to_string(), "port".to_string())),
        "expected 'port' (with a typed value) as @definition.var in kdl tags, \
         got: {pairs:?}"
    );
}

// --- Dimension 2/3: completeness matrix (variants.kdl) ----------------------

#[test]
fn kdl_tags_completeness_quoted_and_typed_node_names() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kdl_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kdl").ok() else {
        eprintln!("Skipping kdl_tags_completeness: kdl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("kdl").expect("kdl tags query missing");
    let pairs = collect_tag_pairs(&lang, KDL_VARIANTS, &query_str);

    assert!(
        pairs.contains(&("definition.class".to_string(), "container".to_string())),
        "expected bare-word container 'container' as @definition.class, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "plain-leaf".to_string())),
        "expected bare-word leaf 'plain-leaf' as @definition.var, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&(
            "definition.var".to_string(),
            "\"quoted node name\"".to_string()
        )),
        "expected quoted leaf node name as @definition.var, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&(
            "definition.class".to_string(),
            "container-quoted".to_string()
        )),
        "expected quoted-child container as @definition.class, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "\"quoted-child\"".to_string())),
        "expected quoted leaf child node as @definition.var, got: {pairs:?}"
    );
    // Typed node-name variants: `(type)identifier`, both container and leaf.
    assert!(
        pairs.contains(&(
            "definition.class".to_string(),
            "typed-container".to_string()
        )),
        "expected typed container '(u8)typed-container' name as @definition.class, \
         got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "typed-leaf".to_string())),
        "expected typed leaf '(i64)typed-leaf' name as @definition.var, got: {pairs:?}"
    );
    // Trailing same-line comments must not suppress the name capture.
    assert!(
        pairs.contains(&(
            "definition.var".to_string(),
            "trailing-comment-leaf".to_string()
        )),
        "expected 'trailing-comment-leaf' captured despite trailing same-line comment, \
         got: {pairs:?}"
    );
    assert!(
        pairs.contains(&(
            "definition.var".to_string(),
            "trailing-comment-typed".to_string()
        )),
        "expected typed 'trailing-comment-typed' captured despite trailing same-line \
         comment, got: {pairs:?}"
    );
}

/// Negative case: KDL's slash-dash (`/-`) whole-node comment syntax still
/// parses as a live `node` (not ERROR, not omitted from the tree — verified
/// via `normalize syntax ast`), so an unanchored `(node (identifier) @name
/// ...)` pattern silently extracted commented-out config as real symbols.
/// The anchored fix must exclude a node whose own header is slash-dashed,
/// for both the untyped and typed name forms.
#[test]
fn kdl_tags_negative_slash_dash_disabled_nodes_not_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kdl_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kdl").ok() else {
        eprintln!("Skipping kdl_tags_negative: kdl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("kdl").expect("kdl tags query missing");
    let pairs = collect_tag_pairs(&lang, KDL_VARIANTS, &query_str);
    let names: Vec<&str> = pairs.iter().map(|(_, n)| n.as_str()).collect();

    for disabled in [
        "disabled-leaf",
        "disabled-typed-leaf",
        "disabled-top-level-container",
        "disabled-typed-container",
    ] {
        assert!(
            !names.contains(&disabled),
            "slash-dash-disabled node '{disabled}' must not be captured as a symbol, \
             got: {names:?}"
        );
    }
    // Its live sibling inside the same container must still be captured —
    // guards against the fix over-excluding the whole container.
    assert!(
        names.contains(&"live-sibling"),
        "expected live sibling 'live-sibling' still captured despite a disabled \
         sibling in the same container, got: {names:?}"
    );
}
