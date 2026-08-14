//! Query fixture tests for nginx.
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
// Nginx
// ---------------------------------------------------------------------------

const NGINX_SAMPLE: &str = include_str!("fixtures/nginx/nginx.conf");

const NGINX_VARIANTS: &str = include_str!("fixtures/nginx/variants.conf");

// --- tags --------------------------------------------------------------

#[test]
fn nginx_tags_finds_block_directives_and_lua_blocks() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_tags: nginx grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("nginx").expect("nginx tags query missing");
    let names = collect_captures(&lang, NGINX_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "server" || n == "http" || n == "upstream"),
        "expected block directive names in nginx tags, got: {names:?}"
    );
    // Lua block directives (OpenResty) are a distinct node kind with no
    // `name` field; the keyword token itself must still surface as @name.
    assert!(
        names.iter().any(|n| n == "access_by_lua_block"),
        "expected access_by_lua_block among nginx tags names, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "content_by_lua_block"),
        "expected content_by_lua_block among nginx tags names, got: {names:?}"
    );
}

#[test]
fn nginx_tags_completeness_all_lua_block_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_tags_completeness: nginx grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("nginx").expect("nginx tags query missing");
    let full = collect_captures_full(&lang, NGINX_VARIANTS, &query_str);

    // Every one of the 7 lua_block_directive keyword variants from
    // node-types.json must appear as a @name capture co-occurring with
    // @definition.module, with the anonymous token itself as the kind.
    let lua_variants = [
        "access_by_lua_block",
        "balancer_by_lua_block",
        "body_filter_by_lua_block",
        "content_by_lua_block",
        "header_filter_by_lua_block",
        "log_by_lua_block",
        "rewrite_by_lua_block",
    ];
    for variant in lua_variants {
        assert!(
            full.iter()
                .any(|(cap, kind, text, _)| cap == "name" && kind == variant && text == variant),
            "expected @name capture of kind {variant:?} in nginx tags completeness, got: {full:?}"
        );
    }

    // Plain block_directive name is still a `directive` node kind, not
    // conflated with the lua anonymous-token kind.
    assert!(
        full.iter()
            .any(|(cap, kind, text, _)| cap == "name" && kind == "directive" && text == "http"),
        "expected @name capture of kind 'directive' for block_directive, got: {full:?}"
    );
}

// --- calls ---------------------------------------------------------------

#[test]
fn nginx_calls_finds_simple_and_block_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_calls: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("nginx")
        .expect("nginx calls query missing");
    let calls = collect_captures(&lang, NGINX_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "proxy_pass"),
        "expected simple_directive 'proxy_pass' in nginx calls, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "server"),
        "expected block_directive 'server' in nginx calls, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "access_by_lua_block"),
        "expected lua_block_directive 'access_by_lua_block' in nginx calls, got: {calls:?}"
    );
}

#[test]
fn nginx_calls_completeness_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_calls_completeness: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("nginx")
        .expect("nginx calls query missing");
    let full = collect_captures_full(&lang, NGINX_VARIANTS, &query_str);

    // simple_directive.name: kind is always `directive`.
    assert!(
        full.iter().any(|(cap, kind, text, _)| cap == "call"
            && kind == "directive"
            && text == "worker_processes"),
        "expected simple_directive call capture, got: {full:?}"
    );
    // block_directive.name: kind is always `directive`.
    assert!(
        full.iter()
            .any(|(cap, kind, text, _)| cap == "call" && kind == "directive" && text == "http"),
        "expected block_directive call capture, got: {full:?}"
    );
    // All 7 lua_block_directive keyword variants: kind is the anonymous
    // literal token itself (matches its own text).
    let lua_variants = [
        "access_by_lua_block",
        "balancer_by_lua_block",
        "body_filter_by_lua_block",
        "content_by_lua_block",
        "header_filter_by_lua_block",
        "log_by_lua_block",
        "rewrite_by_lua_block",
    ];
    for variant in lua_variants {
        assert!(
            full.iter()
                .any(|(cap, kind, text, _)| cap == "call" && kind == variant && text == variant),
            "expected lua_block_directive call capture {variant:?}, got: {full:?}"
        );
    }
}

#[test]
fn nginx_calls_negative_directive_params_not_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_calls_negative: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("nginx")
        .expect("nginx calls query missing");
    let calls = collect_captures(&lang, NGINX_VARIANTS, &query_str, "call");
    // `auto` is a directive *argument* (worker_processes auto;), never a
    // directive name, and must never surface as a @call capture.
    assert!(
        !calls.iter().any(|c| c == "auto"),
        "directive param 'auto' must not be captured as a call, got: {calls:?}"
    );
    // `1024` (worker_connections 1024;) is a param, not a directive name.
    assert!(
        !calls.iter().any(|c| c == "1024"),
        "directive param '1024' must not be captured as a call, got: {calls:?}"
    );
}

// --- complexity ------------------------------------------------------------

#[test]
fn nginx_complexity_finds_block_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_complexity: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("nginx")
        .expect("nginx complexity query missing");
    let complexity = collect_captures(&lang, NGINX_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 block directive complexity nodes in nginx sample, got: {complexity:?}"
    );
}

#[test]
fn nginx_complexity_counts_block_and_lua_block_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_complexity_counts: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_complexity_counts: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("nginx")
        .expect("nginx complexity query missing");
    let full = collect_captures_full(&lang, NGINX_VARIANTS, &query_str);

    // variants.conf has exactly 11 block_directive nodes and 7
    // lua_block_directive nodes (verified via `normalize syntax query`),
    // and both contribute to @complexity and @nesting.
    let complexity_block = full
        .iter()
        .filter(|(cap, kind, ..)| cap == "complexity" && kind == "block_directive")
        .count();
    let complexity_lua = full
        .iter()
        .filter(|(cap, kind, ..)| cap == "complexity" && kind == "lua_block_directive")
        .count();
    assert_eq!(
        complexity_block, 11,
        "expected 11 block_directive @complexity captures, got: {full:?}"
    );
    assert_eq!(
        complexity_lua, 7,
        "expected 7 lua_block_directive @complexity captures, got: {full:?}"
    );

    let nesting_block = full
        .iter()
        .filter(|(cap, kind, ..)| cap == "nesting" && kind == "block_directive")
        .count();
    let nesting_lua = full
        .iter()
        .filter(|(cap, kind, ..)| cap == "nesting" && kind == "lua_block_directive")
        .count();
    assert_eq!(
        nesting_block, 11,
        "expected 11 block_directive @nesting captures, got: {full:?}"
    );
    assert_eq!(
        nesting_lua, 7,
        "expected 7 lua_block_directive @nesting captures, got: {full:?}"
    );
}

// --- imports ---------------------------------------------------------------

#[test]
fn nginx_imports_finds_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_imports: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("nginx")
        .expect("nginx imports query missing");
    let paths = collect_captures(&lang, NGINX_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("mime.types") || p.contains("fastcgi_params")),
        "expected include paths in nginx imports, got: {paths:?}"
    );
    // Quoted include path: quotes are preserved at the query-capture layer
    // (stripping happens downstream in normalize-facts::strip_import_quotes).
    assert!(
        paths.iter().any(|p| p == "\"sites-enabled/site.conf\""),
        "expected quoted include path in nginx imports, got: {paths:?}"
    );
    // Glob include: the anchored query captures only the literal prefix, not
    // 3 fragmented bogus paths (see nginx.imports.scm for why the full glob
    // text is unrecoverable at the query layer).
    assert!(
        paths.iter().any(|p| p == "/etc/nginx/conf.d/"),
        "expected glob include prefix in nginx imports, got: {paths:?}"
    );
    // Exactly one import.path per include line, even for the glob include —
    // this is the regression check for the fragmentation bug.
    assert_eq!(
        paths.len(),
        4,
        "expected exactly 4 import.path captures (one per include line) in nginx sample, got: {paths:?}"
    );
}

#[test]
fn nginx_imports_completeness_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_imports_completeness: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("nginx")
        .expect("nginx imports query missing");
    let full = collect_captures_full(&lang, NGINX_VARIANTS, &query_str);
    let paths: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.path")
        .map(|(_, _, text, _)| text.as_str())
        .collect();

    // One @import.path per include variant in variants.conf, in source order:
    assert_eq!(
        paths,
        vec![
            "/etc/nginx/mime.types",     // plain unquoted path
            "\"quoted/path.conf\"",      // double-quoted path
            "'single/quoted.conf'",      // single-quoted path
            "relative/conf.d/site.conf", // relative path, no leading slash
            "/etc/nginx/conf.d/",        // glob path -- literal prefix only
            "sites-enabled/",            // bare wildcard with dir prefix
        ],
        "unexpected import.path set/order for nginx variants fixture, got full: {full:?}"
    );

    // Every import.path capture is a `param` node (the first child of the
    // simple_directive), not a `generic`/`string` grandchild -- confirms the
    // query captures the param wrapper, matching what strip_import_quotes
    // downstream expects to receive (quotes intact).
    for (cap, kind, text, _) in &full {
        if cap == "import.path" {
            assert_eq!(
                kind, "param",
                "expected import.path capture kind 'param' for {text:?}, got {kind:?}"
            );
        }
    }
}

#[test]
fn nginx_imports_negative_non_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nginx_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nginx").ok() else {
        eprintln!("Skipping nginx_imports_negative: nginx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("nginx")
        .expect("nginx imports query missing");
    let full = collect_captures_full(&lang, NGINX_VARIANTS, &query_str);

    // `includes_something_else 1;` must NOT match -- #eq? requires the exact
    // directive name "include", not merely a name containing "include" as a
    // substring/prefix.
    assert!(
        !full
            .iter()
            .any(|(_, _, text, _)| text.contains("includes_something_else")),
        "directive 'includes_something_else' must not match the imports query, got: {full:?}"
    );
    // block_directive-form directives (events, http, server, location) never
    // produce @import captures -- the query only matches simple_directive.
    let import_count = full.iter().filter(|(cap, ..)| cap == "import").count();
    assert_eq!(
        import_count, 6,
        "expected exactly 6 @import matches (one per include line) in nginx variants, got: {full:?}"
    );
}
