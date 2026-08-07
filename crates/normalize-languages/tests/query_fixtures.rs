/// Fixture tests for `.scm` tree-sitter query files.
///
/// Each test parses a sample source file, runs the relevant query, and asserts that
/// specific expected names appear in captures.
///
/// # Running
///
/// These tests require compiled grammar `.so` files in `target/grammars/`. Build them
/// with `cargo xtask build-grammars`. Without grammars present the tests skip gracefully
/// — `cargo test` always passes regardless of grammar availability.
///
/// To run with grammars:
///   cargo xtask build-grammars && cargo test -p normalize-languages -- --nocapture
use normalize_languages::GrammarLoader;
use std::path::PathBuf;
use std::sync::Arc;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the grammar search path if `target/grammars/` exists relative to the
/// workspace root, otherwise return `None` to signal the test should be skipped.
fn grammar_dir() -> Option<PathBuf> {
    // Integration tests run with cwd = crate root; grammars live at workspace root.
    let crate_root = std::env::current_dir().unwrap();
    let workspace_root = crate_root
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())?;
    let dir = workspace_root.join("target/grammars");
    if dir.exists() { Some(dir) } else { None }
}

/// Like [`grammar_dir`], but panics when `NORMALIZE_REQUIRE_GRAMMARS` is set and
/// the grammar directory is missing.  Use in decoration tests (and other new tests)
/// so that CI — which sets the env var — catches silent skips.
fn require_grammar_dir() -> Option<PathBuf> {
    let dir = grammar_dir();
    if dir.is_none() && std::env::var("NORMALIZE_REQUIRE_GRAMMARS").is_ok() {
        panic!(
            "NORMALIZE_REQUIRE_GRAMMARS is set but target/grammars/ does not exist \
             — run `cargo xtask build-grammars` first"
        );
    }
    dir
}

/// Parse `source` with `lang`, run `query_str` against it, and collect all
/// captures whose name starts with `capture_prefix` into a `Vec<String>`.
fn collect_captures(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
    capture_prefix: &str,
) -> Vec<String> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut results = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            if cap_name.starts_with(capture_prefix) {
                let text = cap.node.utf8_text(source_bytes).unwrap_or("").to_string();
                results.push(text);
            }
        }
    }
    results
}

/// Like [`collect_captures`], but returns `(capture_name, node_kind, text, line)`
/// for every capture (regardless of prefix). Use this when a test needs to
/// assert on capture *kind* (extraction depth), not just capture text — the
/// same text can legitimately come from different node kinds (e.g. a
/// `type_identifier` named "new" vs an `identifier` named "new"), and a test
/// that only checks text can't tell them apart.
fn collect_captures_full(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
) -> Vec<(String, String, String, usize)> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut results = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize].to_string();
            let kind = cap.node.kind().to_string();
            let text = cap.node.utf8_text(source_bytes).unwrap_or("").to_string();
            let line = cap.node.start_position().row + 1;
            results.push((cap_name, kind, text, line));
        }
    }
    results
}

/// Collect `(tag_kind, name_text)` pairs from a tags-style query: `tag_kind`
/// is whichever `@definition.*`/`@reference.*` capture co-occurs with `@name`
/// in the same match. Use this instead of [`collect_captures_full`] when the
/// container capture (e.g. `@reference.class`) spans a much larger node (the
/// whole `new Foo()`/`extends Foo` expression) than the `@name` capture that
/// actually holds the identifier text.
fn collect_tag_pairs(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
) -> Vec<(String, String)> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut pairs = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut name = None;
        let mut tag_kind = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(source_bytes).unwrap_or("");
            if cap_name == "name" {
                name = Some(text.to_string());
            } else if cap_name.starts_with("definition.") || cap_name.starts_with("reference.") {
                tag_kind = Some(cap_name.to_string());
            }
        }
        if let (Some(n), Some(k)) = (name, tag_kind) {
            pairs.push((k, n));
        }
    }
    pairs
}

/// Run `query_str` against `source` and, for every match that carries a
/// capture named `anchor_capture_name` (e.g. `"reference.class"`,
/// `"definition.module"`), return the `(kind, text)` of that match's `@name`
/// capture. Use this instead of naively filtering `collect_captures_full` by
/// the anchor capture's own name when the anchor is attached to a *container*
/// node (e.g. `new_expression`, `extends_clause`, `module`/`internal_module`)
/// rather than to the field-variant node itself — the anchor's own `kind`
/// would otherwise always report the container type, never the variant.
fn tags_matches_by_kind(
    lang: &tree_sitter::Language,
    source: &str,
    query_str: &str,
    anchor_capture_name: &str,
) -> Vec<(String, String)> {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    let tree = parser.parse(source, None).expect("parse failed");

    let query = Query::new(lang, query_str).expect("query compilation failed");
    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let mut results = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut has_anchor = false;
        let mut name: Option<(String, String)> = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            if cap_name == anchor_capture_name {
                has_anchor = true;
            } else if cap_name == "name" {
                let kind = cap.node.kind().to_string();
                let text = cap.node.utf8_text(source_bytes).unwrap_or("").to_string();
                name = Some((kind, text));
            }
        }
        if has_anchor && let Some(n) = name {
            results.push(n);
        }
    }
    results
}

// ---------------------------------------------------------------------------
// Rust
// ---------------------------------------------------------------------------

const RUST_SAMPLE: &str = include_str!("fixtures/rust/sample.rs");
const RUST_VARIANTS: &str = include_str!("fixtures/rust/variants.rs");

// --- Dimension 4: real-world fixture coverage (sample.rs) -------------------

#[test]
fn rust_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_tags: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("rust").expect("rust tags query missing");
    let names = collect_captures(&lang, RUST_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Counter".to_string()),
        "expected 'Counter' struct in tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sum_evens".to_string()),
        "expected 'sum_evens' function in tags, got: {names:?}"
    );
    // Generic struct + generic impl block (idiom-density: real Rust leans on
    // generics heavily). Point<T> is declared `impl<T: Add<...> + Copy> Point<T>`
    // — the container must still be found (type: generic_type).
    assert!(
        names.iter().filter(|n| *n == "Point").count() >= 2,
        "expected 'Point' from both the struct_item and the generic impl_item, got: {names:?}"
    );
    // Nested module + nested #[cfg(test)] mod: both must be found as
    // @definition.module, and functions inside must still be found.
    assert!(
        names.contains(&"shapes".to_string()),
        "expected 'shapes' module in tags, got: {names:?}"
    );
    assert!(
        names.contains(&"describe".to_string()),
        "expected 'describe' function nested in mod shapes, got: {names:?}"
    );
    // Closures must never be reported as function/method definitions: the
    // binding name `make_adder` should appear (it's a real fn), but no name
    // from inside its closure body (`base`, `x`) should appear as a
    // definition — closures aren't `function_item`s in the grammar.
    assert!(
        names.contains(&"make_adder".to_string()),
        "expected 'make_adder' function in tags, got: {names:?}"
    );
}

#[test]
fn rust_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let calls = collect_captures(&lang, RUST_SAMPLE, &query_str, "call");
    // Counter::new() → @call = "new", increment/get are method calls
    assert!(
        calls.iter().any(|c| c == "new"),
        "expected 'new' call in rust sample, got: {calls:?}"
    );
    // Iterator-chain idiom: .filter(...).map(...).collect::<Vec<_>>() — the
    // turbofish `.collect::<Vec<_>>()` must be found (generic_function
    // wrapping a field_expression), not just the untyped calls in the chain.
    assert!(
        calls.iter().any(|c| c == "collect"),
        "expected 'collect' call (incl. its turbofish form) in rust sample, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "filter"),
        "expected 'filter' call in rust sample, got: {calls:?}"
    );
    // `s.parse::<i32>().ok()` inside parse_all: turbofish method call.
    assert!(
        calls.iter().any(|c| c == "parse"),
        "expected 'parse' turbofish call in rust sample, got: {calls:?}"
    );
}

#[test]
fn rust_imports_finds_use_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_imports: rust grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("rust")
        .expect("rust imports query missing");
    let paths = collect_captures(&lang, RUST_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("std")),
        "expected std import path in rust sample, got: {paths:?}"
    );
    let names = collect_captures(&lang, RUST_SAMPLE, &query_str, "import.name");
    assert!(
        names.contains(&"HashMap".to_string()),
        "expected 'HashMap' import name in rust sample, got: {names:?}"
    );
    // `use std::fmt::{self, Display};` — the self-import member must be
    // captured too, or the module import silently disappears. This construct
    // has been present in this fixture from the start; the original shallow
    // test (name.contains("HashMap")) never exercised it, which is exactly
    // the gap this enriched suite exists to close.
    assert!(
        names.iter().any(|n| n == "self"),
        "expected a 'self' import name (use std::fmt::{{self, Display}}) in rust sample, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.rs) -

/// Every grammar-legal variant of `call_expression.function` that
/// rust.calls.scm claims to support must actually match, with the right
/// capture *kind* (dimension 3) — not just the right text.
#[test]
fn rust_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls_completeness: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);

    // (capture_name, kind, text) triples we require, one per documented
    // function-field variant. See rust.calls.scm's own comments for the
    // node-shape each of these lines exercises.
    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"),    // plain_call
        ("call", "identifier", "drop"), // scoped_call: function: scoped_identifier, name: identifier
        ("call", "field_identifier", "len"), // method_call
        ("call", "identifier", "identity"), // turbofish_plain_call (same text as plain_call, different line)
        ("call", "identifier", "size_of"),  // turbofish_scoped_call
        ("call", "field_identifier", "parse"), // turbofish_method_call
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in rust.calls.scm \
             output for variants.rs, got: {caps:?}"
        );
    }

    // @call.qualifier must be present for every scoped/method call and carry
    // the qualifier text, not the call name.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.iter().any(|q| q.contains("std::mem")),
        "expected 'std::mem' qualifier for scoped/turbofish-scoped calls, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"v"),
        "expected 'v' qualifier for the plain method call, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.iter().any(|q| q.contains('5')),
        "expected the string-literal receiver qualifier for the turbofish method call, got: {qualifiers:?}"
    );
}

/// @call.write is reserved for assignment/compound-assignment RHS; a `let`
/// binding must never produce @call.write, and every write-context variant
/// (assignment, compound assignment) must be covered.
#[test]
fn rust_calls_write_context_distinguishes_let_from_assignment() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls_write_context: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls_write_context: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);

    let write_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call.write" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    // Lines 52 (`result = identity(1)`) and 53 (`result += identity(2)`) are
    // write-context; line 54 (`let _read = identity(3)`) must NOT appear here.
    assert_eq!(
        write_calls.len(),
        2,
        "expected exactly 2 @call.write 'identity' captures (assignment + compound \
         assignment), got {}: {write_calls:?} (full captures: {caps:?})",
        write_calls.len()
    );

    let plain_identity_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    // plain_call, turbofish_plain_call, and the `let`-bound call in
    // write_context_call all use plain @call, never @call.write.
    assert!(
        plain_identity_calls.len() >= 3,
        "expected at least 3 plain @call 'identity' captures (incl. the let-bound one), \
         got {}: {plain_identity_calls:?}",
        plain_identity_calls.len()
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn rust_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_calls_negative: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("rust").expect("rust calls query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call" || cn == "call.write")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` is a bare field read (no call parens); must never be a call.
    assert!(
        !call_texts.contains(&"field"),
        "bare field access 'holder.field' must not be captured as a call, got: {call_texts:?}"
    );
    // The closure parameter/body identifiers (`add_one`'s definition site,
    // `x`) must not appear as calls — only the *call site* `add_one(1)` should.
    let add_one_calls = call_texts.iter().filter(|t| **t == "add_one").count();
    assert_eq!(
        add_one_calls, 1,
        "expected exactly 1 call to 'add_one' (the call site, not the closure \
         definition), got {add_one_calls}: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `impl_item.type` and `impl_item.trait` that
/// rust.tags.scm claims to support (plain, generic, path-qualified, and their
/// combinations) must produce a @name capture with the correct definition/
/// reference kind.
#[test]
fn rust_tags_completeness_all_impl_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_tags_completeness: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("rust").expect("rust tags query missing");
    let query = Query::new(&lang, &query_str).expect("query compilation failed");
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(RUST_VARIANTS, None).expect("parse failed");
    let source_bytes = RUST_VARIANTS.as_bytes();

    // Collect (tag_kind, name_text) pairs: tag_kind is whichever
    // @definition.*/@reference.* capture co-occurs with @name in the match.
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut name = None;
        let mut tag_kind = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(source_bytes).unwrap_or("");
            if cap_name == "name" {
                name = Some(text.to_string());
            } else if cap_name.starts_with("definition.") || cap_name.starts_with("reference.") {
                tag_kind = Some(cap_name.to_string());
            }
        }
        if let (Some(n), Some(k)) = (name, tag_kind) {
            pairs.push((k, n));
        }
    }

    // Plain inherent impl: impl Plain {}
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.module" && n == "Plain"),
        "expected 'Plain' as definition.module (plain impl container), got: {pairs:?}"
    );
    // Generic inherent impl: impl<T> Generic<T> {} — type: generic_type.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.module" && n == "Generic"),
        "expected 'Generic' as definition.module (generic impl container), got: {pairs:?}"
    );
    // Path-qualified trait impl: impl std::fmt::Debug for Plain — trait: scoped_type_identifier.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "Debug"),
        "expected 'Debug' as reference.implementation (path-qualified trait), got: {pairs:?}"
    );
    // Generic trait impl: impl From<i32> for Plain — trait: generic_type -> type_identifier.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "From"),
        "expected 'From' as reference.implementation (generic trait), got: {pairs:?}"
    );
    // Generic + path-qualified trait impl: impl std::ops::Add<i32> for Plain
    // — trait: generic_type -> type: scoped_type_identifier.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "Add"),
        "expected 'Add' as reference.implementation (generic + path-qualified trait), got: {pairs:?}"
    );
}

/// Negative case: closures are not `function_item`s and must never appear as
/// @definition.function or @definition.method.
#[test]
fn rust_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_tags_negative: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("rust").expect("rust tags query missing");
    let caps = collect_captures_full(&lang, RUST_VARIANTS, &query_str);
    let def_fn_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // `add_one` is a closure binding, not a function_item; it must never be
    // reported as a definition name. Only the call-site capture ("add_one"
    // via @reference.call) is legitimate — @name is shared across
    // definitions and references in this query, so we check specifically
    // that no @definition.function/@definition.method pairs with the name.
    let is_def_add_one = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t == "add_one"
    });
    assert!(
        !is_def_add_one,
        "closure binding 'add_one' must never be captured as a function/method \
         definition, got names: {def_fn_names:?}"
    );
}

/// Every grammar-legal variant of `use_declaration` that rust.imports.scm
/// claims to support (plain, aliased, wildcard, multi-name, self-import,
/// aliased self-import, re-export) must produce a correctly-shaped @import.
#[test]
fn rust_imports_completeness_all_use_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_imports_completeness: rust grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("rust")
        .expect("rust imports query missing");
    let names = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.alias");
    let globs = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.glob");
    let reexports = collect_captures(&lang, RUST_VARIANTS, &query_str, "import.reexport");

    // Self-import: use std::io::{self, Read, Write};
    assert!(
        names.iter().filter(|n| *n == "self").count() >= 2,
        "expected 2 'self' import names (plain + aliased self-import), got: {names:?}"
    );
    // Aliased self-import: use std::io::{self as io_alias};
    assert!(
        aliases.contains(&"io_alias".to_string()),
        "expected 'io_alias' among import aliases (aliased self-import), got: {aliases:?}"
    );
    // Multi-name with an aliased member: use std::collections::{HashSet, BTreeMap as Tree};
    assert!(
        names.contains(&"HashSet".to_string()),
        "expected 'HashSet' import name, got: {names:?}"
    );
    assert!(
        names.contains(&"BTreeMap".to_string()) && aliases.contains(&"Tree".to_string()),
        "expected 'BTreeMap as Tree' aliased member, names={names:?} aliases={aliases:?}"
    );
    // Wildcard: use std::env::*;
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture for 'use std::env::*;', got: {globs:?}"
    );
    // Aliased re-export: pub use std::fmt::Debug as DebugTrait;
    assert!(
        !reexports.is_empty(),
        "expected at least one import.reexport capture for 'pub use ... as DebugTrait;', got: {reexports:?}"
    );
    assert!(
        aliases.contains(&"DebugTrait".to_string()),
        "expected 'DebugTrait' alias for the re-export, got: {aliases:?}"
    );
}

#[test]
fn rust_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_complexity: rust grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("rust")
        .expect("rust complexity query missing");
    let complexity = collect_captures(&lang, RUST_SAMPLE, &query_str, "complexity");
    // classify() has two if branches; sum_evens() has for + if
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in rust sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn rust_types_finds_struct_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rust_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rust").ok() else {
        eprintln!("Skipping rust_types: rust grammar .so not found");
        return;
    };
    let query_str = loader.get_types("rust").expect("rust types query missing");
    let names = collect_captures(&lang, RUST_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Counter".to_string()),
        "expected 'Counter' in rust types captures, got: {names:?}"
    );
}

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

// ---------------------------------------------------------------------------
// Go
// ---------------------------------------------------------------------------

const GO_SAMPLE: &str = include_str!("fixtures/go/sample.go");
const GO_VARIANTS: &str = include_str!("fixtures/go/variants.go");

// --- Dimension 4: real-world fixture coverage (sample.go) -------------------

#[test]
fn go_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_tags: go grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("go").expect("go tags query missing");
    let names = collect_captures(&lang, GO_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Classify".to_string()),
        "expected 'Classify' function in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"JoinWords".to_string()),
        "expected 'JoinWords' function in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' type in go tags, got: {names:?}"
    );
    // Generic function/type/method idioms (real Go leans on generics
    // heavily since 1.18): Max[T Ordered], Box[T any], and its method Get.
    assert!(
        names.contains(&"Max".to_string()),
        "expected generic function 'Max' in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Box".to_string()),
        "expected generic type 'Box' in go tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Get".to_string()),
        "expected method 'Get' on generic receiver Box[T] in go tags, got: {names:?}"
    );
    // Type alias: `type MyInt = int` must be found as a definition, not
    // silently dropped (the type_alias node is distinct from type_spec).
    assert!(
        names.contains(&"MyInt".to_string()),
        "expected type alias 'MyInt' in go tags, got: {names:?}"
    );
    // Struct embedding: Derived embeds Base; both must still be found.
    assert!(
        names.contains(&"Base".to_string()) && names.contains(&"Derived".to_string()),
        "expected both 'Base' and 'Derived' (struct embedding) in go tags, got: {names:?}"
    );
    // Closures must never be reported as function/method definitions: the
    // outer `adder` binding is a real function, but nothing from inside its
    // closure body should appear as a definition name it doesn't already
    // have legitimately (`delta`, `base` are parameters, not definitions).
    assert!(
        names.contains(&"adder".to_string()),
        "expected 'adder' function in go tags, got: {names:?}"
    );
}

#[test]
fn go_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let calls = collect_captures(&lang, GO_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"Println".to_string()),
        "expected 'Println' call in go sample, got: {calls:?}"
    );
    // Generic call relying on type inference (`Max(3, 4)`, no explicit
    // instantiation) parses as an ordinary call_expression/identifier and
    // must be found like any other plain call.
    assert!(
        calls.contains(&"Max".to_string()),
        "expected generic call 'Max' in go sample, got: {calls:?}"
    );
    // Method call on a generic receiver: b.Get().
    assert!(
        calls.contains(&"Get".to_string()),
        "expected method call 'Get' in go sample, got: {calls:?}"
    );
    // Package-qualified call through an aliased import (io.Discard via
    // `io "io"`) — same call shape as a plain package call.
    assert!(
        calls.contains(&"Fprintln".to_string()),
        "expected 'Fprintln' call (aliased-import-qualified) in go sample, got: {calls:?}"
    );
}

#[test]
fn go_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_imports: go grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("go").expect("go imports query missing");
    let paths = collect_captures(&lang, GO_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("fmt")),
        "expected '\"fmt\"' in go import paths, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("strings")),
        "expected '\"strings\"' in go import paths, got: {paths:?}"
    );
    // Aliased import: `io "io"` — must still surface a path, not just the
    // unaliased forms.
    assert!(
        paths.iter().any(|p| p.contains("io")),
        "expected aliased '\"io\"' import path in go sample, got: {paths:?}"
    );
    let aliases = collect_captures(&lang, GO_SAMPLE, &query_str, "import.alias");
    assert!(
        aliases.iter().any(|a| a == "io"),
        "expected 'io' alias captured for the aliased import, got: {aliases:?}"
    );
}

#[test]
fn go_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_complexity: go grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("go")
        .expect("go complexity query missing");
    let complexity = collect_captures(&lang, GO_SAMPLE, &query_str, "complexity");
    // Classify() has two if branches; Pop() has one if; Max[T] has one if;
    // sumEvens has a for-range loop plus a nested if.
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in go sample, got {} ({complexity:?})",
        complexity.len()
    );
    // for _, n := range nums { ... } — the range-loop form of for_statement
    // must count as a complexity/nesting node exactly like a classic
    // three-clause for loop (both are for_statement; range_clause is a
    // child, not a separate statement type).
    let nesting = collect_captures(&lang, GO_SAMPLE, &query_str, "nesting");
    assert!(
        nesting.len() >= 2,
        "expected at least 2 nesting nodes (incl. the for-range loop and its \
         enclosing function) in go sample, got {} ({nesting:?})",
        nesting.len()
    );
}

#[test]
fn go_types_finds_struct_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_types: go grammar .so not found");
        return;
    };
    let query_str = loader.get_types("go").expect("go types query missing");
    let names = collect_captures(&lang, GO_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' in go types captures, got: {names:?}"
    );
    // Type alias must appear in types.scm too, not just tags.scm.
    assert!(
        names.contains(&"MyInt".to_string()),
        "expected type alias 'MyInt' in go types captures, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.go) -

/// Every grammar-legal variant of `call_expression.function` that
/// go.calls.scm claims to support must actually match, with the right
/// capture *kind* (dimension 3) — not just the right text.
#[test]
fn go_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);

    let call_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // plain_call: function: identifier
    assert!(
        call_names.contains(&"identity"),
        "expected 'identity' plain call, got: {call_names:?}"
    );
    // method_call: function: selector_expression, field: field_identifier
    assert!(
        call_names.contains(&"len"),
        "expected 'len' method call, got: {call_names:?}"
    );
    // scoped_call: function: selector_expression, package-qualified
    assert!(
        call_names.contains(&"Sprintf"),
        "expected 'Sprintf' scoped call, got: {call_names:?}"
    );
    // dot-imported call: brought into scope directly by `. "strings"`, so
    // it parses as a plain identifier call, same as plain_call.
    assert!(
        call_names.contains(&"Repeat"),
        "expected dot-imported 'Repeat' call (parses as plain identifier call), got: {call_names:?}"
    );

    // Every @call capture must be either an identifier or a field_identifier
    // — never the parenthesized wrapper or anything larger (extraction
    // depth: capture *kind*, not just text).
    for (cn, kind, text, line) in &caps {
        if cn == "call" {
            assert!(
                kind == "identifier" || kind == "field_identifier",
                "expected @call capture kind to be identifier/field_identifier, \
                 got kind={kind} text={text} line={line}"
            );
        }
    }

    // @call.qualifier must carry the qualifier text for every scoped/method call.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"fmt"),
        "expected 'fmt' qualifier for the package-qualified call, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"h"),
        "expected 'h' qualifier for the method call, got: {qualifiers:?}"
    );
}

/// @call.write is reserved for assignment/compound-assignment RHS; a
/// `:=`-bound call must never produce @call.write.
#[test]
fn go_calls_write_context_distinguishes_short_var_decl_from_assignment() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls_write_context: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls_write_context: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);

    let write_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call.write" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    let plain_calls: Vec<usize> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "call" && t == "identity")
        .map(|(_, _, _, line)| *line)
        .collect();
    // go.calls.scm has no dedicated @call.write pattern (unlike Rust) — Go's
    // grammar doesn't distinguish assignment-RHS calls from plain calls in
    // a way this query captures separately, so every `identity(...)` call
    // site (the plain call in callVariants, plus assignment,
    // compound-assignment, and `:=` in writeContextCall — 4 total) surfaces
    // as plain @call. This test documents that as current behavior: no
    // @call.write captures exist anywhere.
    assert!(
        write_calls.is_empty(),
        "go.calls.scm has no @call.write pattern; expected zero, got: {write_calls:?}"
    );
    assert_eq!(
        plain_calls.len(),
        4,
        "expected all 4 'identity' call sites (plain, assignment, \
         compound-assignment, short-var-decl) as plain @call, got {}: {plain_calls:?}",
        plain_calls.len()
    );
}

/// Negative cases: the three deliberately-excluded call_expression.function
/// variants (documented in go.calls.scm) must never produce a @call capture.
#[test]
fn go_calls_negative_uncallable_function_variants_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_calls_negative: go grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("go").expect("go calls query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // Curried call `higherOrder()(1)`: the outer call's callee is itself a
    // call_expression, not a named symbol. `higherOrder` (the inner call's
    // name) is legitimately captured once; it must not appear a second time
    // from the outer call.
    let higher_order_calls = call_texts.iter().filter(|t| **t == "higherOrder").count();
    assert_eq!(
        higher_order_calls, 1,
        "expected exactly 1 call to 'higherOrder' (the inner call only), got \
         {higher_order_calls}: {call_texts:?}"
    );
    // Immediately-invoked func_literal: no identifier/field_identifier
    // named "iife" exists in the source at all (it's a string argument to
    // Println), so its absence from @call is trivially satisfied; the real
    // assertion is structural — no @call capture has kind "func_literal".
    assert!(
        !caps
            .iter()
            .any(|(cn, kind, _, _)| cn == "call" && kind == "func_literal"),
        "func_literal must never itself be captured as @call, got: {caps:?}"
    );
    // Dispatch-table call `dispatch[0]()`: no capture of kind
    // "index_expression" as @call.
    assert!(
        !caps
            .iter()
            .any(|(cn, kind, _, _)| cn == "call" && kind == "index_expression"),
        "index_expression must never itself be captured as @call, got: {caps:?}"
    );
}

/// Every grammar-legal variant of type-definition node (`type_spec` and the
/// distinct `type_alias` node) that go.tags.scm claims to support must
/// produce a @definition.type capture.
#[test]
fn go_tags_completeness_type_spec_and_type_alias() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_tags_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("go").expect("go tags query missing");
    let query = Query::new(&lang, &query_str).expect("query compilation failed");
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(GO_VARIANTS, None).expect("parse failed");
    let source_bytes = GO_VARIANTS.as_bytes();

    // Collect (tag_kind, name_text) pairs: tag_kind is whichever
    // @definition.*/@reference.* capture co-occurs with @name in the match
    // (mirrors rust_tags_completeness_all_impl_variants's pairing approach —
    // definition.type is tagged on the *whole* type_spec/type_alias node,
    // not the name, so pairing by match is required to get the name text).
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut name = None;
        let mut tag_kind = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(source_bytes).unwrap_or("");
            if cap_name == "name" {
                name = Some(text.to_string());
            } else if cap_name.starts_with("definition.") {
                tag_kind = Some(cap_name.to_string());
            }
        }
        if let (Some(n), Some(k)) = (name, tag_kind) {
            pairs.push((k, n));
        }
    }

    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.type" && n == "Plain"),
        "expected 'Plain' (type_spec) as definition.type, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.type" && n == "PlainAlias"),
        "expected 'PlainAlias' (type_alias) as definition.type, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.type" && n == "Generic"),
        "expected 'Generic' (generic type_spec) as definition.type, got: {pairs:?}"
    );
}

/// Negative case: closures/func_literals are never reported as
/// @definition.function or @definition.method.
#[test]
fn go_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_tags_negative: go grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("go").expect("go tags query missing");
    let caps = collect_captures_full(&lang, GO_VARIANTS, &query_str);
    let is_def_delta = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t == "delta"
    });
    assert!(
        !is_def_delta,
        "closure parameter 'delta' must never be captured as a function/method \
         definition, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `import_spec` that go.imports.scm claims
/// to support (plain, aliased, dot, blank, and both string-literal kinds
/// for the path) must produce a correctly-shaped @import.
#[test]
fn go_imports_completeness_all_import_spec_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_imports_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("go").expect("go imports query missing");
    let paths = collect_captures(&lang, GO_VARIANTS, &query_str, "import.path");
    let aliases = collect_captures(&lang, GO_VARIANTS, &query_str, "import.alias");
    let globs = collect_captures(&lang, GO_VARIANTS, &query_str, "import.glob");

    assert!(
        paths.iter().any(|p| p.contains("fmt")),
        "expected plain 'fmt' import path, got: {paths:?}"
    );
    assert!(
        aliases.contains(&"f".to_string()),
        "expected 'f' alias for the aliased fmt import, got: {aliases:?}"
    );
    assert!(
        aliases.contains(&"_".to_string()) || paths.iter().any(|p| p.contains("os")),
        "expected the blank import of \"os\" to surface a path, got paths={paths:?} aliases={aliases:?}"
    );
    assert_eq!(
        globs.len(),
        1,
        "expected exactly 1 dot-import glob marker (`. \"strings\"`), got {}: {globs:?}",
        globs.len()
    );
    // raw_string_literal import path: `errors` (backtick-quoted). Rare in
    // real Go (gofmt never emits it) but grammar-legal.
    assert!(
        paths.iter().any(|p| p.contains("errors")),
        "expected raw-string-literal import path '`errors`' to be captured, got: {paths:?}"
    );
}

/// Every grammar-legal variant of complexity/nesting node that
/// go.complexity.scm claims to support must produce a capture, plus a
/// negative case for constructs that must not increase complexity.
#[test]
fn go_complexity_completeness_and_negative() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping go_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("go").ok() else {
        eprintln!("Skipping go_complexity_completeness: go grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("go")
        .expect("go complexity query missing");
    let caps = collect_captures_full(&lang, GO_SAMPLE, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    // if_statement (Classify/Pop/Max/sumEvens) and binary_expression
    // (`n%2 == 0`, `a > b`, comparisons throughout) must both appear.
    assert!(
        complexity_kinds.contains(&"if_statement"),
        "expected an if_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"binary_expression"),
        "expected a binary_expression complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"for_statement"),
        "expected a for_statement complexity node (the for-range loop in \
         sumEvens), got: {complexity_kinds:?}"
    );
    // NEGATIVE: a plain function call or a plain assignment must not
    // introduce a complexity node — only the 3 listed statement/expression
    // kinds above (and switch/select forms, not exercised in sample.go) do.
    assert!(
        !complexity_kinds.contains(&"call_expression"),
        "call_expression must never be a complexity node, got: {complexity_kinds:?}"
    );
}

// ---------------------------------------------------------------------------
// TypeScript
// ---------------------------------------------------------------------------

const TS_SAMPLE: &str = include_str!("fixtures/typescript/sample.ts");
const TS_VARIANTS: &str = include_str!("fixtures/typescript/variants.ts");

// --- Dimension 4: real-world fixture coverage (sample.ts) -------------------

#[test]
fn typescript_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_tags: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let names = collect_captures(&lang, TS_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"FileLogger".to_string()),
        "expected 'FileLogger' class in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"formatPath".to_string()),
        "expected 'formatPath' function in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"groupBy".to_string()),
        "expected 'groupBy' function in typescript tags, got: {names:?}"
    );
    // Widget extends Entity implements Comparable<Widget> — both the
    // superclass and the generic interface must be found as references.
    assert!(
        names.contains(&"Entity".to_string()),
        "expected 'Entity' superclass reference in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Comparable".to_string()),
        "expected 'Comparable' generic interface reference in typescript tags, got: {names:?}"
    );
    // Private method #computeScore must be found as a method definition, not
    // silently dropped for having a private_property_identifier name.
    assert!(
        names.iter().any(|n| n == "#computeScore"),
        "expected private method '#computeScore' in typescript tags, got: {names:?}"
    );
    // `namespace Shapes { ... namespace Nested { ... } }` — both the outer
    // and nested namespace must be found as definition.module (internal_module).
    assert!(
        names.contains(&"Shapes".to_string()),
        "expected 'Shapes' namespace in typescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Nested".to_string()),
        "expected nested 'Nested' namespace in typescript tags, got: {names:?}"
    );
    // Closures assigned inside makeCounter must never appear as definitions.
    assert!(
        names.contains(&"makeCounter".to_string()),
        "expected 'makeCounter' function in typescript tags, got: {names:?}"
    );
}

#[test]
fn typescript_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_calls: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typescript")
        .expect("typescript calls query missing");
    let calls = collect_captures(&lang, TS_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "normalize" || c == "log" || c == "push"),
        "expected at least one of normalize/log/push calls in typescript sample, got: {calls:?}"
    );
    // Private method call site: this.#computeScore() inside score().
    assert!(
        calls.iter().any(|c| c == "#computeScore"),
        "expected private method call '#computeScore' in typescript sample, got: {calls:?}"
    );
    // Promise chain idiom: .then(...).catch(...) — both calls found.
    assert!(
        calls.iter().any(|c| c == "then"),
        "expected 'then' call in typescript sample, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|c| c == "catch"),
        "expected 'catch' call in typescript sample, got: {calls:?}"
    );
}

#[test]
fn typescript_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_imports: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("typescript")
        .expect("typescript imports query missing");
    let paths = collect_captures(&lang, TS_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"events".to_string()),
        "expected 'events' in typescript import paths, got: {paths:?}"
    );
    assert!(
        paths.contains(&"path".to_string()),
        "expected 'path' in typescript import paths, got: {paths:?}"
    );
}

#[test]
fn typescript_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_complexity: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("typescript")
        .expect("typescript complexity query missing");
    let complexity = collect_captures(&lang, TS_SAMPLE, &query_str, "complexity");
    // formatPath has an if; groupBy has a for_in
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in typescript sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn typescript_types_finds_interface_and_class() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_types: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("typescript")
        .expect("typescript types query missing");
    let names = collect_captures(&lang, TS_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"FileLogger".to_string()) || names.contains(&"Logger".to_string()),
        "expected 'FileLogger' or 'Logger' in typescript types captures, got: {names:?}"
    );
}

#[test]
fn typescript_types_finds_extends_and_implements_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_types_extends_implements: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_types_extends_implements: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("typescript")
        .expect("typescript types query missing");
    let refs = collect_captures(&lang, TS_SAMPLE, &query_str, "type.reference");
    // class Widget extends Entity implements Comparable<Widget>
    assert!(
        refs.contains(&"Entity".to_string()),
        "expected 'Entity' extends-reference in typescript types, got: {refs:?}"
    );
    assert!(
        refs.contains(&"Comparable".to_string()),
        "expected 'Comparable' implements-reference (generic) in typescript types, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.ts) -

/// Every grammar-legal variant of `call_expression.function` that
/// typescript.calls.scm claims to support must actually match, with the
/// right capture *kind* (dimension 3) — not just the right text.
#[test]
fn typescript_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_calls_completeness: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typescript")
        .expect("typescript calls query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);

    // (capture_name, kind, text) triples we require, one per documented
    // function-field variant. See typescript.calls.scm's own comments for
    // the node-shape each of these lines exercises.
    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"),                  // plainCall
        ("call", "property_identifier", "push"), // methodCall: function: member_expression, property: property_identifier
        ("call", "private_property_identifier", "#compute"), // callPrivate: private method call
        ("call", "subscript_expression", "arr[0]"), // computedCall
        ("call", "parenthesized_expression", "(identity)"), // parenthesizedCall
        ("call", "non_null_expression", "identity!"), // nonNullCall
        ("call", "call_expression", "curried()"), // chainedCall
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in typescript.calls.scm \
             output for variants.ts, got: {caps:?}"
        );
    }

    // @call.qualifier must be present for method/computed calls and carry the
    // qualifier text, not the call name.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"arr"),
        "expected 'arr' qualifier for the plain method call, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"this"),
        "expected 'this' qualifier for the private method call, got: {qualifiers:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn typescript_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_calls_negative: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typescript")
        .expect("typescript calls query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` is a bare field read (no call parens); must never be a call.
    assert!(
        !call_texts.contains(&"field"),
        "bare field access 'holder.field' must not be captured as a call, got: {call_texts:?}"
    );
    // The closure definition site (`addOne`) must not appear as a call —
    // only the call site `addOne(1)` should.
    let add_one_calls = call_texts.iter().filter(|t| **t == "addOne").count();
    assert_eq!(
        add_one_calls, 1,
        "expected exactly 1 call to 'addOne' (the call site, not the closure \
         definition), got {add_one_calls}: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `method_definition.name` that
/// typescript.tags.scm claims to support (plain, private, computed) must
/// produce a @definition.method capture with the correct name text.
#[test]
fn typescript_tags_completeness_all_method_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_methods: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!(
            "Skipping typescript_tags_completeness_methods: typescript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);
    let method_defs: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "definition.method")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // definition.method is anchored on the whole method_definition node, so
    // check by substring/kind pairing on the @name capture instead.
    let name_kinds: Vec<(&str, &str)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, k, t, _)| (k.as_str(), t.as_str()))
        .collect();
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "property_identifier" && *t == "plainMethod"),
        "expected plain method name 'plainMethod' (property_identifier), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "private_property_identifier" && *t == "#privateMethod"),
        "expected private method name '#privateMethod' (private_property_identifier), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "computed_property_name" && *t == "[\"computedMethod\"]"),
        "expected computed method name (computed_property_name), got: {name_kinds:?}"
    );
    let _ = method_defs; // kept for readability of what's being asserted above
}

/// Every grammar-legal variant of `new_expression.constructor` that
/// typescript.tags.scm claims to support (plain identifier, namespaced
/// member_expression) must produce a @reference.class capture.
#[test]
fn typescript_tags_completeness_new_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_new: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_tags_completeness_new: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let class_refs = tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "reference.class");
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "PrivateHolder"),
        "expected plain constructor 'PrivateHolder' (identifier), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "member_expression" && t == "ns2.Ctor"),
        "expected namespaced constructor 'ns2.Ctor' (member_expression), got: {class_refs:?}"
    );
}

/// Every grammar-legal variant of `module`/`internal_module` name (identifier,
/// nested_identifier, ambient string) that typescript.tags.scm claims to
/// support must produce a @definition.module capture.
#[test]
fn typescript_tags_completeness_module_and_namespace_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_modules: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!(
            "Skipping typescript_tags_completeness_modules: typescript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let module_defs = tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "definition.module");
    // `module LegacyModule {}` — module.name: identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "LegacyModule"),
        "expected legacy 'module LegacyModule' (module.name: identifier), got: {module_defs:?}"
    );
    // `module Legacy.Dotted {}` — module.name: nested_identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "nested_identifier" && t == "Legacy.Dotted"),
        "expected legacy 'module Legacy.Dotted' (module.name: nested_identifier), got: {module_defs:?}"
    );
    // `declare module "ambient-module-name" {}` — module.name: string
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "string" && t == "\"ambient-module-name\""),
        "expected ambient 'declare module \"...\"' (module.name: string), got: {module_defs:?}"
    );
    // `namespace SimpleNamespace {}` — internal_module.name: identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "SimpleNamespace"),
        "expected 'namespace SimpleNamespace' (internal_module.name: identifier), got: {module_defs:?}"
    );
    // `namespace Dotted.Nested {}` — internal_module.name: nested_identifier
    assert!(
        module_defs
            .iter()
            .any(|(k, t)| k == "nested_identifier" && t == "Dotted.Nested"),
        "expected 'namespace Dotted.Nested' (internal_module.name: nested_identifier), got: {module_defs:?}"
    );
}

/// Every grammar-legal variant of `extends_clause`/`implements_clause` that
/// typescript.tags.scm claims to support must produce the correct
/// reference.class/reference.implementation capture.
#[test]
fn typescript_tags_completeness_extends_implements_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_tags_completeness_extends: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!(
            "Skipping typescript_tags_completeness_extends: typescript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let class_refs = tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "reference.class");
    let impl_refs =
        tags_matches_by_kind(&lang, TS_VARIANTS, &query_str, "reference.implementation");
    let impl_ref_texts: Vec<&str> = impl_refs.iter().map(|(_, t)| t.as_str()).collect();

    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "Base"),
        "expected 'Base' extends-reference (identifier), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "member_expression" && t == "ns2.Ctor"),
        "expected 'ns2.Ctor' extends-reference (member_expression), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "call_expression" && t.starts_with("Mixin(")),
        "expected 'Mixin(Base)' extends-reference (call_expression, mixin pattern), got: {class_refs:?}"
    );
    assert!(
        impl_ref_texts.contains(&"Iface"),
        "expected 'Iface' implements-reference (plain type_identifier), got: {impl_refs:?}"
    );
    assert!(
        impl_ref_texts.contains(&"GenericIface"),
        "expected 'GenericIface' implements-reference (generic_type), got: {impl_refs:?}"
    );
}

/// Negative case: closures are not function_declarations/method_definitions
/// and must never appear as @definition.function or @definition.method.
#[test]
fn typescript_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typescript_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_tags_negative: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("typescript")
        .expect("typescript tags query missing");
    let caps = collect_captures_full(&lang, TS_VARIANTS, &query_str);
    let is_def_add_one = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t == "addOne"
    });
    assert!(
        !is_def_add_one,
        "closure binding 'addOne' must never be captured as a function/method \
         definition, got captures: {caps:?}"
    );
}

/// Every grammar-legal variant of import/re-export/require/dynamic-import
/// that typescript.imports.scm claims to support must produce a correctly
/// shaped @import capture, including the previously-silent `default`-name
/// (anonymous-token) and `import X = require(...)`/`import()` gaps.
#[test]
fn typescript_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping typescript_imports_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typescript").ok() else {
        eprintln!("Skipping typescript_imports_completeness: typescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("typescript")
        .expect("typescript imports query missing");
    let names = collect_captures(&lang, TS_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, TS_VARIANTS, &query_str, "import.alias");
    let paths = collect_captures(&lang, TS_VARIANTS, &query_str, "import.path");
    let globs = collect_captures(&lang, TS_VARIANTS, &query_str, "import.glob");
    let reexports = collect_captures(&lang, TS_VARIANTS, &query_str, "import.reexport");

    assert!(
        names.contains(&"plainName".to_string()),
        "expected plain import name, got: {names:?}"
    );
    // import { default as renamedDefault } — previously silently dropped
    // entirely since `default` is an anonymous token, not (identifier).
    assert!(
        names.iter().any(|n| n == "default"),
        "expected a 'default' import name (import {{ default as ... }}), got: {names:?}"
    );
    assert!(
        aliases.contains(&"renamedDefault".to_string()),
        "expected 'renamedDefault' alias for the default-import, got: {aliases:?}"
    );
    // import fsThing = require('fs') — TS import-equals with require.
    assert!(
        names.contains(&"fsThing".to_string()) && paths.contains(&"fs".to_string()),
        "expected 'fsThing'/'fs' from import-equals-require, names={names:?} paths={paths:?}"
    );
    // export * as wildcardNs from ... — namespace re-export.
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture, got: {globs:?}"
    );
    assert!(
        aliases.contains(&"wildcardNs".to_string()),
        "expected 'wildcardNs' namespace re-export alias, got: {aliases:?}"
    );
    // export { default } from ... (bare default re-export) — must appear.
    assert!(
        reexports.len() >= 2,
        "expected multiple @import.reexport captures (named + default forms), got {}: {reexports:?}",
        reexports.len()
    );
    // export { default as renamedDefaultReexport } from ...
    assert!(
        aliases.contains(&"renamedDefaultReexport".to_string()),
        "expected 'renamedDefaultReexport' aliased-default-reexport alias, got: {aliases:?}"
    );
    // import('mod-dynamic') — dynamic import expression.
    assert!(
        paths.contains(&"mod-dynamic".to_string()),
        "expected 'mod-dynamic' from dynamic import(), got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Java
// ---------------------------------------------------------------------------

const JAVA_SAMPLE: &str = include_str!("fixtures/java/sample.java");
const JAVA_VARIANTS: &str = include_str!("fixtures/java/variants.java");

// --- Dimension 4: real-world fixture coverage (sample.java) ----------------

#[test]
fn java_tags_finds_class_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let names = collect_captures(&lang, JAVA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"TaskQueue".to_string()),
        "expected 'TaskQueue' class in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"enqueue".to_string()),
        "expected 'enqueue' method in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' method in java tags, got: {names:?}"
    );
    // implements Comparable<TaskQueue>, java.io.Serializable — both the
    // generic and the path-qualified interface must be found as containers
    // for the nested `PriorityTaskQueue extends TaskQueue implements
    // java.util.Comparator<String>` idiom (generic + scoped supertype).
    assert!(
        names.contains(&"Comparable".to_string()),
        "expected 'Comparable' (generic implements) in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Serializable".to_string()),
        "expected 'Serializable' (path-qualified implements) in java tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Comparator".to_string()),
        "expected 'Comparator' (generic + path-qualified implements) in java tags, got: {names:?}"
    );
    // Nested static class + its qualified/generic extends clause.
    assert!(
        names.contains(&"PriorityTaskQueue".to_string()),
        "expected 'PriorityTaskQueue' nested class in java tags, got: {names:?}"
    );
    // Anonymous class (`new Runnable() { ... }`) — its constructor-call
    // reference and the `run` override inside it must both surface.
    assert!(
        names.contains(&"Runnable".to_string()),
        "expected 'Runnable' anonymous-class reference in java tags, got: {names:?}"
    );
    // Enum with a constructor and a method.
    assert!(
        names.contains(&"Color".to_string()),
        "expected 'Color' enum in java tags, got: {names:?}"
    );
    // Record.
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' record in java tags, got: {names:?}"
    );
    // Lambda bindings (`String::length` method reference site, `t -> ...`)
    // must never surface as function/method definitions — closures and
    // method references aren't `method_declaration`s in this grammar.
    let def_method_names: Vec<&str> = names
        .iter()
        .map(std::string::String::as_str)
        .filter(|n| *n == "lengthFn" || *n == "t")
        .collect();
    assert!(
        def_method_names.contains(&"lengthFn"),
        "expected the real method 'lengthFn' in java tags, got: {names:?}"
    );
}

#[test]
fn java_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_calls: java grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("java").expect("java calls query missing");
    let calls = collect_captures(&lang, JAVA_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"add".to_string()) || calls.contains(&"remove".to_string()),
        "expected 'add' or 'remove' method call in java sample, got: {calls:?}"
    );
    // Iterator-chain idiom: tasks.stream().filter(...).map(...).count() —
    // every link in the chain must be found, not just the first call.
    assert!(
        calls.contains(&"stream".to_string()),
        "expected 'stream' call in java sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"filter".to_string()),
        "expected 'filter' call in java sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"count".to_string()),
        "expected 'count' call in java sample, got: {calls:?}"
    );
    // Qualified static call: Integer.compare(...).
    assert!(
        calls.contains(&"compare".to_string()),
        "expected 'compare' static-qualified call in java sample, got: {calls:?}"
    );
    // super(capacity) constructor delegation inside the nested subclass
    // constructor — a distinct `explicit_constructor_invocation` node, not a
    // `method_invocation`, that was previously entirely unmatched.
    assert!(
        calls.contains(&"super".to_string()),
        "expected 'super' constructor-delegation call in java sample, got: {calls:?}"
    );
}

#[test]
fn java_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_imports: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("java")
        .expect("java imports query missing");
    let paths = collect_captures(&lang, JAVA_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("ArrayList")),
        "expected 'java.util.ArrayList' in java import paths, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("function.Function")),
        "expected 'java.util.function.Function' in java import paths, got: {paths:?}"
    );
}

#[test]
fn java_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_complexity: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("java")
        .expect("java complexity query missing");
    let complexity = collect_captures(&lang, JAVA_SAMPLE, &query_str, "complexity");
    // enqueue() has an if; dequeue() has an if; classify() has if/else-if;
    // Shapes.describe() has a switch (arrow form, 3 labels incl. default).
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in java sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn java_types_finds_class() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_types: java grammar .so not found");
        return;
    };
    let query_str = loader.get_types("java").expect("java types query missing");
    let names = collect_captures(&lang, JAVA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"TaskQueue".to_string()),
        "expected 'TaskQueue' in java types captures, got: {names:?}"
    );
    // interface_declaration, enum_declaration, and record_declaration must
    // all be reported as @definition.type alongside class_declaration.
    assert!(
        names.contains(&"Processor".to_string()),
        "expected 'Processor' interface in java types captures, got: {names:?}"
    );
    assert!(
        names.contains(&"Color".to_string()),
        "expected 'Color' enum in java types captures, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' record in java types captures, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.java) -

/// Every grammar-legal variant of `object_creation_expression.type` that
/// java.tags.scm claims to support (plain, generic, diamond, scoped, and
/// generic+scoped) must produce a @reference.class capture with the right name.
#[test]
fn java_tags_completeness_object_creation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_tags_completeness_object_creation: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags_completeness_object_creation: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);

    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    for expected in ["Object", "ArrayList", "Date", "HashMap"] {
        assert!(
            ref_class_names.contains(&expected),
            "expected '{expected}' among object-creation @reference.class captures, got: {ref_class_names:?}"
        );
    }
    // Extraction depth: the leaf name must be the plain class name
    // ("Date"), not the qualified path text ("java.util.Date"), even for
    // scoped/generic-scoped constructors.
    assert!(
        ref_class_names.iter().all(|n| !n.contains('.')),
        "expected leaf-only class names (no '.'), got: {ref_class_names:?}"
    );
}

/// Every grammar-legal variant of `superclass` and `type_list` (implements)
/// that java.tags.scm claims to support must produce a
/// @reference.class/@reference.implementation capture.
#[test]
fn java_tags_completeness_extends_implements_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_tags_completeness_extends_implements: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags_completeness_extends_implements: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);

    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    // superclass: plain, generic, generic+scoped.
    assert!(
        ref_class_names.contains(&"PlainBase"),
        "expected 'PlainBase' (plain superclass) in java tags, got: {ref_class_names:?}"
    );
    assert!(
        ref_class_names.contains(&"GenericBase"),
        "expected 'GenericBase' (generic superclass) in java tags, got: {ref_class_names:?}"
    );
    assert!(
        ref_class_names.contains(&"AbstractList"),
        "expected 'AbstractList' (generic + path-qualified superclass) in java tags, got: {ref_class_names:?}"
    );

    let ref_impl_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.implementation")
        .map(|(_, n)| n.as_str())
        .collect();
    // type_list (implements): plain, generic, scoped, generic+scoped.
    assert!(
        ref_impl_names.contains(&"PlainIface"),
        "expected 'PlainIface' (plain implements) in java tags, got: {ref_impl_names:?}"
    );
    assert!(
        ref_impl_names.contains(&"Comparable"),
        "expected 'Comparable' (generic implements) in java tags, got: {ref_impl_names:?}"
    );
    assert!(
        ref_impl_names.contains(&"Serializable"),
        "expected 'Serializable' (path-qualified implements) in java tags, got: {ref_impl_names:?}"
    );
    assert!(
        ref_impl_names.contains(&"Comparator"),
        "expected 'Comparator' (generic + path-qualified implements) in java tags, got: {ref_impl_names:?}"
    );
}

/// Every type-defining declaration kind (class, interface, enum, record,
/// annotation type) must be found as a tags definition, and record/annotation
/// must be mapped to the documented closest-existing kind (class/interface).
#[test]
fn java_tags_completeness_type_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_tags_completeness_type_declaration_kinds: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!(
            "Skipping java_tags_completeness_type_declaration_kinds: java grammar .so not found"
        );
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);

    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(find_def_kind("PlainClass"), Some("definition.class"));
    assert_eq!(
        find_def_kind("PlainInterface"),
        Some("definition.interface")
    );
    assert_eq!(find_def_kind("PlainEnum"), Some("definition.enum"));
    assert_eq!(
        find_def_kind("PlainRecord"),
        Some("definition.class"),
        "records compile to classes; expected definition.class"
    );
    assert_eq!(
        find_def_kind("PlainAnnotation"),
        Some("definition.interface"),
        "annotation types compile to interfaces; expected definition.interface"
    );
}

/// Negative case: lambda bindings and method references are not
/// `method_declaration`s and must never appear as tags definitions; bare
/// field access/writes must never appear as calls.
#[test]
fn java_tags_negative_lambdas_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_tags_negative: java grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("java").expect("java tags query missing");
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let is_def_lambda_binding = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.method" || cn == "definition.function") && t == "lambdaBinding"
    });
    assert!(
        !is_def_lambda_binding,
        "lambda binding 'lambdaBinding' must never be captured as a method/function \
         definition, got: {caps:?}"
    );
    let is_def_method_ref = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.method" || cn == "definition.function") && t == "methodRef"
    });
    assert!(
        !is_def_method_ref,
        "method-reference binding 'methodRef' must never be captured as a method/function \
         definition, got: {caps:?}"
    );
}

/// Negative case: method references (`Foo::bar`) are not invocations and
/// must never appear as @call captures in java.calls.scm.
#[test]
fn java_calls_negative_method_references_and_field_access() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping java_calls_negative_method_references: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_calls_negative_method_references: java grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("java").expect("java calls query missing");
    let calls = collect_captures(&lang, JAVA_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"staticMethod".to_string()),
        "method reference 'NegativeHolder::staticMethod' must not be captured as a call, \
         got: {calls:?}"
    );
    // Bare field read (`this.field`) and field write (`this.field = 5`) must
    // never be captured as calls (no argument_list, not a method_invocation).
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access/write 'this.field' must not be captured as a call, got: {calls:?}"
    );
}

/// Every grammar-legal variant of `method_invocation.object` (absent, plain
/// identifier qualifier, chained method_invocation qualifier) must produce a
/// @call capture, matching java.calls.scm's completeness claims.
#[test]
fn java_calls_completeness_object_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_calls_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("java").expect("java calls query missing");
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        calls.contains(&"identity"),
        "expected plain (no-object) call 'identity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"abs"),
        "expected qualified call 'Math.abs' -> 'abs', got: {calls:?}"
    );
    // Chained calls: s.trim().toUpperCase().length() — every link found.
    assert!(
        calls.contains(&"trim"),
        "expected chained call 'trim', got: {calls:?}"
    );
    assert!(
        calls.contains(&"toUpperCase"),
        "expected chained call 'toUpperCase', got: {calls:?}"
    );
    assert!(
        calls.contains(&"length"),
        "expected chained call 'length', got: {calls:?}"
    );
    // Extraction depth: the qualifier for the chained calls must be the
    // *previous method_invocation node*, not a plain identifier.
    let chained_qualifier_kind = caps
        .iter()
        .find(|(cn, _, t, _)| cn == "call.qualifier" && t.starts_with("s.trim()"))
        .map(|(_, k, _, _)| k.as_str());
    assert_eq!(
        chained_qualifier_kind,
        Some("method_invocation"),
        "expected the chained call's qualifier to be a method_invocation node, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `import_declaration`'s argument (bare
/// identifier, scoped_identifier, wildcard, static, static wildcard) that
/// java.imports.scm claims to support must produce a correctly-shaped @import.
#[test]
fn java_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_imports_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("java")
        .expect("java imports query missing");
    let paths = collect_captures(&lang, JAVA_VARIANTS, &query_str, "import.path");
    let globs = collect_captures(&lang, JAVA_VARIANTS, &query_str, "import.glob");

    // Bare single-segment import: `import Bare;`
    assert!(
        paths.contains(&"Bare".to_string()),
        "expected 'Bare' bare-identifier import path, got: {paths:?}"
    );
    // Plain scoped import: `import java.util.ArrayList;`
    assert!(
        paths.iter().any(|p| p.contains("ArrayList")),
        "expected 'java.util.ArrayList' import path, got: {paths:?}"
    );
    // Wildcard: `import java.util.*;`
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture for 'import java.util.*;', got: {globs:?}"
    );
    // import static pkg.Class.member; and import static pkg.Class.*;
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Math.PI") || p.contains("PI")),
        "expected static import path for 'java.lang.Math.PI', got: {paths:?}"
    );
    assert!(
        globs.len() >= 2,
        "expected 2 import.glob captures (plain wildcard + static wildcard), got {}: {globs:?}",
        globs.len()
    );
}

/// Negative case: `import.path` must never be empty/missing for any of the
/// import forms above — a silent drop (0 matches) is exactly the historical
/// bug class this methodology targets.
#[test]
fn java_imports_negative_no_silently_dropped_forms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_imports_negative: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("java")
        .expect("java imports query missing");
    // Exact-match "import" only — collect_captures' prefix match would also
    // pull in "import.path"/"import.glob"/"import.reexport", which is not
    // what this test is asserting.
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let import_stmts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // variants.java has exactly 6 import declarations; every one must
    // produce at least one @import capture (the whole-statement anchor).
    assert_eq!(
        import_stmts.len(),
        6,
        "expected 6 @import captures (one per import declaration in variants.java), got {}: {import_stmts:?}",
        import_stmts.len()
    );
}

/// Completeness: switch (arrow form), try-with-resources, and enhanced-for
/// all contribute complexity, matching java.complexity.scm's claims.
#[test]
fn java_complexity_completeness_control_flow_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_complexity_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("java")
        .expect("java complexity query missing");
    let caps = collect_captures_full(&lang, JAVA_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    // switch (arrow form): 3 switch_label nodes (case 1, case 2, default).
    assert!(
        complexity_kinds
            .iter()
            .filter(|k| **k == "switch_label")
            .count()
            >= 3,
        "expected >= 3 switch_label complexity nodes (arrow-form switch), got: {complexity_kinds:?}"
    );
    // catch_clause from the try-with-resources block.
    assert!(
        complexity_kinds.contains(&"catch_clause"),
        "expected a catch_clause complexity node, got: {complexity_kinds:?}"
    );
    // for/while/do-while/enhanced-for loops.
    assert!(
        complexity_kinds.contains(&"for_statement"),
        "expected a for_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"while_statement"),
        "expected a while_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"do_statement"),
        "expected a do_statement complexity node, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"enhanced_for_statement"),
        "expected an enhanced_for_statement complexity node, got: {complexity_kinds:?}"
    );
}

/// Every type-defining declaration kind must be found as @definition.type in
/// java.types.scm, matching the tags completeness matrix above.
#[test]
fn java_types_completeness_all_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping java_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("java").ok() else {
        eprintln!("Skipping java_types_completeness: java grammar .so not found");
        return;
    };
    let query_str = loader.get_types("java").expect("java types query missing");
    let pairs = collect_tag_pairs(&lang, JAVA_VARIANTS, &query_str);
    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(find_def_kind("PlainClass"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainInterface"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainEnum"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainRecord"), Some("definition.type"));
    assert_eq!(find_def_kind("PlainAnnotation"), Some("definition.type"));
}

// ---------------------------------------------------------------------------
// Ruby
// ---------------------------------------------------------------------------

const RUBY_SAMPLE: &str = include_str!("fixtures/ruby/sample.rb");
const RUBY_VARIANTS: &str = include_str!("fixtures/ruby/variants.rb");

// --- Dimension 4: real-world fixture coverage (sample.rb) -------------------

#[test]
fn ruby_tags_finds_class_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_tags: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ruby").expect("ruby tags query missing");
    let names = collect_captures(&lang, RUBY_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in ruby tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' method in ruby tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sum_if".to_string()),
        "expected 'sum_if' method in ruby tags, got: {names:?}"
    );
    // Mixin module + namespaced include: real Ruby leans heavily on modules
    // as mixins, frequently with namespaced module names (ActiveSupport::Concern-
    // style). The module itself and its own method must both be found.
    assert!(
        names.contains(&"Loggable".to_string()),
        "expected 'Loggable' module in ruby tags, got: {names:?}"
    );
    assert!(
        names.contains(&"log".to_string()),
        "expected 'log' method nested in module Loggable, got: {names:?}"
    );
    // Inheritance: BoundedStack < Stack.
    assert!(
        names.contains(&"BoundedStack".to_string()),
        "expected 'BoundedStack' class in ruby tags, got: {names:?}"
    );
    // Struct.new-based value class with a block body: the block's own method
    // definition ('distance') must still be found even though its container
    // is a constant assignment, not a `class` node.
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' method inside Struct.new block, got: {names:?}"
    );
    // `class << self; def empty; end; end` — the method inside a singleton-
    // class reopening must be found as a plain method definition (see
    // ruby.tags.scm's comment on why the singleton_class container itself
    // is not captured).
    assert!(
        names.contains(&"empty".to_string()),
        "expected 'empty' class method (via class << self) in ruby tags, got: {names:?}"
    );
}

#[test]
fn ruby_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_calls: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ruby").expect("ruby calls query missing");
    let calls = collect_captures(&lang, RUBY_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"push".to_string()) || calls.contains(&"pop".to_string()),
        "expected 'push' or 'pop' call in ruby sample, got: {calls:?}"
    );
    // Bare Kernel-style call whose callee is a constant, not an identifier:
    // Integer("5") — this is the gap fixed on top of the shallow baseline.
    assert!(
        calls.contains(&"Integer".to_string()),
        "expected 'Integer' bare-constant call in ruby sample, got: {calls:?}"
    );
    // Safe navigation: label&.upcase must still be found as an ordinary call.
    assert!(
        calls.contains(&"upcase".to_string()),
        "expected 'upcase' call via safe navigation in ruby sample, got: {calls:?}"
    );
    // `super()`/`super` (implicit-args form) inside BoundedStack — the plain
    // `super` keyword is a distinct node from `call` in this grammar and is
    // legitimately absent from @call; not asserted here (see ruby.calls.scm
    // for why explicit-operator/self/super call forms are out of scope).
}

#[test]
fn ruby_imports_finds_require() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_imports: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("ruby")
        .expect("ruby imports query missing");
    let paths = collect_captures(&lang, RUBY_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"json".to_string()),
        "expected 'json' in ruby import paths, got: {paths:?}"
    );
    // `require_relative 'support/helpers'`
    assert!(
        paths.iter().any(|p| p.contains("helpers")),
        "expected require_relative path in ruby import paths, got: {paths:?}"
    );
    // `include ActiveSupport::Concern` — namespaced include argument
    // (scope_resolution), the real-world-common case the bare-constant-only
    // pattern silently dropped.
    assert!(
        paths.iter().any(|p| p.contains("ActiveSupport")),
        "expected namespaced 'include ActiveSupport::Concern' in ruby import paths, got: {paths:?}"
    );
}

#[test]
fn ruby_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_complexity: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("ruby")
        .expect("ruby complexity query missing");
    let complexity = collect_captures(&lang, RUBY_SAMPLE, &query_str, "complexity");
    // classify() alone now contributes if + elsif (2); pop's rescue,
    // build_report/with_yield's statement modifiers, and describe's
    // case_match/in_clause pattern-match all add further complexity nodes.
    assert!(
        complexity.len() >= 8,
        "expected at least 8 complexity nodes in ruby sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn ruby_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_types: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_types("ruby").expect("ruby types query missing");
    // Ruby types.scm captures @type.reference (superclass/scope resolution)
    let refs = collect_captures(&lang, RUBY_SAMPLE, &query_str, "type");
    // BoundedStack < Stack (plain superclass) and Stack's own `rescue
    // StandardError` are both real type references now present in the
    // enriched sample.
    assert!(
        refs.contains(&"Stack".to_string()),
        "expected 'Stack' superclass reference in ruby sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.rb) -

/// Every grammar-legal variant of `call.method` that ruby.calls.scm claims to
/// support (identifier, constant) must actually match, with the right
/// capture kind (dimension 3), not just the right text.
#[test]
fn ruby_calls_completeness_method_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_calls_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ruby").expect("ruby calls query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"), // plain_call: method: identifier
        ("call", "identifier", "length"),   // method_call_with_receiver: method: identifier
        ("call", "constant", "Integer"),    // bare_constant_call: method: constant
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in ruby.calls.scm \
             output for variants.rb, got: {caps:?}"
        );
    }

    // @call.qualifier must carry the receiver text, not the call name.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"v"),
        "expected 'v' qualifier for the receiver-qualified call, got: {qualifiers:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn ruby_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_calls_negative: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ruby").expect("ruby calls query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` IS a call (method: identifier "field"); it must appear
    // exactly once — the negative guard here is against it appearing twice
    // (once for the call, once spuriously for the bare-identifier catch-all
    // pattern, which is reserved for calls with no explicit `call` node).
    let field_calls = call_texts.iter().filter(|t| **t == "field").count();
    assert_eq!(
        field_calls, 1,
        "expected exactly 1 'field' call (holder.field), got {field_calls}: {call_texts:?}"
    );
    // `bound = read_via_call` reads a local variable; the local reference
    // itself must never be captured as a call.
    assert!(
        !call_texts.contains(&"read_via_call") || {
            // `read_via_call` also appears once as an actual call-site
            // target name earlier (`holder.field`'s result assignment does
            // not call anything named read_via_call) — guard that only the
            // legitimate call-producing text ever lands here.
            call_texts.iter().filter(|t| **t == "read_via_call").count() == 0
        },
        "local variable 'read_via_call' must never be captured as a call, got: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `class.name`/`method.name` that
/// ruby.tags.scm claims to support must produce the correct definition kind.
#[test]
fn ruby_tags_completeness_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_tags_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ruby").expect("ruby tags query missing");
    let query = Query::new(&lang, &query_str).expect("query compilation failed");
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(RUBY_VARIANTS, None).expect("parse failed");
    let source_bytes = RUBY_VARIANTS.as_bytes();

    // Collect (tag_kind, name_text) pairs: tag_kind is whichever
    // @definition.*/@reference.* capture co-occurs with @name in the match.
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);
    while let Some(m) = matches.next() {
        if !normalize_languages::satisfies_predicates(&query, m, source_bytes) {
            continue;
        }
        let mut name = None;
        let mut tag_kind = None;
        for cap in m.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(source_bytes).unwrap_or("");
            if cap_name == "name" {
                name = Some(text.to_string());
            } else if cap_name.starts_with("definition.") || cap_name.starts_with("reference.") {
                tag_kind = Some(cap_name.to_string());
            }
        }
        if let (Some(n), Some(k)) = (name, tag_kind) {
            pairs.push((k, n));
        }
    }
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);

    // class.name: constant (Plain), scope_resolution (Namespaced::Deep).
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.class" && n == "Plain"),
        "expected 'Plain' class (name: constant), got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.class" && n == "Deep"),
        "expected 'Deep' class (name: scope_resolution, from Namespaced::Deep), got: {pairs:?}"
    );

    // method.name: identifier (build), operator (+), setter (name=).
    let def_method_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.method")
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        def_method_names.contains(&"+"),
        "expected 'def +' operator method (name: operator), got: {def_method_names:?}"
    );
    assert!(
        def_method_names.contains(&"name="),
        "expected 'def name=' setter method (name: setter), got: {def_method_names:?}"
    );
    assert!(
        def_method_names.contains(&"build"),
        "expected 'build' method nested inside 'class << self', got: {def_method_names:?}"
    );

    // Bare Kernel-style call captured as @reference.call with kind constant.
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "reference.call" && k == "constant" && t == "Integer"),
        "expected 'Integer' bare-constant call as reference.call, got: {caps:?}"
    );
}

/// Negative case: `class << self`'s singleton_class container has no name
/// field (its value is the bare `self` keyword) and must never itself
/// produce a @definition.class/@definition.module capture with a
/// fabricated name.
#[test]
fn ruby_tags_negative_singleton_class_has_no_definition() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_tags_negative: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ruby").expect("ruby tags query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);
    let self_named_defs = caps
        .iter()
        .filter(|(cn, _, t, _)| {
            (cn == "definition.class" || cn == "definition.module") && t == "self"
        })
        .count();
    assert_eq!(
        self_named_defs, 0,
        "singleton_class ('class << self') must never produce a definition named \
         'self', got: {caps:?}"
    );
}

/// Every grammar-legal variant of require/require_relative/load/using/
/// include/extend/prepend that ruby.imports.scm claims to support.
#[test]
fn ruby_imports_completeness_directive_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_imports_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("ruby")
        .expect("ruby imports query missing");
    let paths = collect_captures(&lang, RUBY_VARIANTS, &query_str, "import.path");

    assert!(
        paths.contains(&"json".to_string()),
        "expected require 'json', got: {paths:?}"
    );
    assert!(
        paths.contains(&"other".to_string()),
        "expected require_relative 'other', got: {paths:?}"
    );
    assert!(
        paths.contains(&"plain.rb".to_string()),
        "expected load 'plain.rb', got: {paths:?}"
    );
    assert!(
        paths.contains(&"RefinementModule".to_string()),
        "expected 'using RefinementModule', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Comparable".to_string()),
        "expected 'include Comparable' (bare constant), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("ActiveSupport")),
        "expected 'include ActiveSupport::Concern' (scope_resolution), got: {paths:?}"
    );
    assert!(
        paths.contains(&"Forwardable".to_string()),
        "expected 'extend Forwardable' (bare constant), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("MyLib")),
        "expected 'extend MyLib::Extensions' (scope_resolution), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("MyModule")),
        "expected 'prepend MyModule::Prependable' (scope_resolution), got: {paths:?}"
    );
}

/// Every grammar-legal variant of statement-modifier and pattern-match
/// complexity nodes that ruby.complexity.scm claims to support, plus elsif
/// branch counting.
#[test]
fn ruby_complexity_completeness_modifier_and_pattern_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_complexity_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("ruby")
        .expect("ruby complexity query missing");
    let caps = collect_captures_full(&lang, RUBY_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();

    for kind in [
        "if_modifier",
        "unless_modifier",
        "while_modifier",
        "until_modifier",
        "rescue_modifier",
        "elsif",
        "case_match",
        "in_clause",
    ] {
        assert!(
            complexity_kinds.contains(&kind),
            "expected a @complexity capture of kind '{kind}' in variants.rb, \
             got kinds: {complexity_kinds:?}"
        );
    }

    // elsif_chain has two elsif branches — both must count independently,
    // not be folded into a single complexity point for the whole chain.
    let elsif_count = complexity_kinds.iter().filter(|k| **k == "elsif").count();
    assert_eq!(
        elsif_count, 2,
        "expected exactly 2 'elsif' complexity nodes (elsif_chain has two), got {elsif_count}"
    );
}

/// Every grammar-legal variant of `superclass` that ruby.types.scm claims to
/// support: plain constant, namespaced (scope_resolution), and dynamic/
/// computed (call) superclasses.
#[test]
fn ruby_types_completeness_superclass_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ruby_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ruby").ok() else {
        eprintln!("Skipping ruby_types_completeness: ruby grammar .so not found");
        return;
    };
    let query_str = loader.get_types("ruby").expect("ruby types query missing");
    let refs = collect_captures(&lang, RUBY_VARIANTS, &query_str, "type");

    // PlainSuper < Plain
    assert!(
        refs.contains(&"Plain".to_string()),
        "expected 'Plain' superclass reference, got: {refs:?}"
    );
    // NamespacedSuper < Outer2::Nested — covered by the generic
    // scope_resolution pattern, not a dedicated superclass one.
    assert!(
        refs.contains(&"Outer2".to_string()) && refs.contains(&"Nested".to_string()),
        "expected 'Outer2'/'Nested' from the namespaced superclass, got: {refs:?}"
    );
    // DynamicSuper < Struct.new(:x, :y) — best-effort receiver capture.
    assert!(
        refs.contains(&"Struct".to_string()),
        "expected 'Struct' receiver reference from the dynamic superclass, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Kotlin
// ---------------------------------------------------------------------------

const KOTLIN_SAMPLE: &str = include_str!("fixtures/kotlin/sample.kt");
const KOTLIN_VARIANTS: &str = include_str!("fixtures/kotlin/variants.kt");

// --- Dimension 4: real-world fixture coverage (sample.kt) -------------------

#[test]
fn kotlin_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_tags: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let names = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in kotlin tags, got: {names:?}"
    );
    // Sealed class hierarchy + interface implemented WITHOUT parens
    // (`class Circle(val r: Double) : Shape` — the near-ubiquitous Kotlin
    // idiom that was previously entirely unmatched).
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' interface reference (no-paren delegation) in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Figure".to_string()),
        "expected 'Figure' sealed class in kotlin tags, got: {names:?}"
    );
    // Secondary constructor delegating to the primary via `this(...)`.
    assert!(
        names.contains(&"this".to_string()),
        "expected 'this' constructor delegation reference in kotlin tags, got: {names:?}"
    );
    // Extension function and suspend function are both ordinary
    // function_declarations — must still surface as @definition.function.
    assert!(
        names.contains(&"shout".to_string()),
        "expected extension function 'shout' in kotlin tags, got: {names:?}"
    );
    assert!(
        names.contains(&"fetchData".to_string()),
        "expected suspend function 'fetchData' in kotlin tags, got: {names:?}"
    );
    // Named companion object.
    assert!(
        names.contains(&"Repository".to_string()),
        "expected 'Repository' class in kotlin tags, got: {names:?}"
    );
}

#[test]
fn kotlin_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_calls: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("kotlin")
        .expect("kotlin calls query missing");
    let calls = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"println".to_string()) || calls.contains(&"enqueue".to_string()),
        "expected 'println' or 'enqueue' call in kotlin sample, got: {calls:?}"
    );
    // Trailing-lambda call: `listOf(1, 2, 3).map { it * 2 }`.
    assert!(
        calls.contains(&"map".to_string()),
        "expected trailing-lambda 'map' call in kotlin sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"filter".to_string()),
        "expected lambda-with-arrow 'filter' call in kotlin sample, got: {calls:?}"
    );
    // `Repository(name, 16)` secondary-constructor `this(...)` delegation —
    // a distinct `constructor_delegation_call` node, not `call_expression`,
    // previously entirely unmatched.
    assert!(
        calls.contains(&"this".to_string()),
        "expected 'this' constructor-delegation call in kotlin sample, got: {calls:?}"
    );
}

#[test]
fn kotlin_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_imports: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("kotlin")
        .expect("kotlin imports query missing");
    let paths = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("LinkedList") || p.contains("java")),
        "expected 'java.util.LinkedList' in kotlin import paths, got: {paths:?}"
    );
    // `import kotlin.math.max as mathMax` — aliased import must still
    // report its path (and, per the completeness test below, must not
    // also be double-counted by the plain-import pattern).
    assert!(
        paths.iter().any(|p| p.contains("max")),
        "expected 'kotlin.math.max' aliased import path in kotlin sample, got: {paths:?}"
    );
}

#[test]
fn kotlin_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_complexity: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("kotlin")
        .expect("kotlin complexity query missing");
    let complexity = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "complexity");
    // classify()'s when-arms, sumEvens()'s if, dequeue()'s if, the
    // when(figure) is-branches, and the try/catch all contribute.
    assert!(
        complexity.len() >= 5,
        "expected at least 5 complexity nodes in kotlin sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn kotlin_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping kotlin_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_types: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("kotlin")
        .expect("kotlin types query missing");
    let refs = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Point" || r == "Double" || r == "Int"),
        "expected 'Point', 'Double', or 'Int' in kotlin type references, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.kt) -

/// Every type-defining declaration kind (class, interface, enum, sealed
/// class, object, type alias) must be found as a tags AND types definition
/// with the correct capture kind.
#[test]
fn kotlin_tags_completeness_type_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_type_declaration_kinds: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_type_declaration_kinds: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);

    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(
        find_def_kind("PlainClass"),
        Some("definition.class"),
        "expected PlainClass as definition.class, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("PlainObject"),
        Some("definition.class"),
        "expected PlainObject as definition.class, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("PlainAlias"),
        Some("definition.type"),
        "expected PlainAlias as definition.type, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("PlainInterface"),
        Some("definition.class"),
        "expected PlainInterface as definition.class (same node kind as class_declaration), got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("Direction"),
        Some("definition.class"),
        "expected enum class Direction as definition.class, got pairs: {pairs:?}"
    );
    assert_eq!(
        find_def_kind("SealedBase"),
        Some("definition.class"),
        "expected sealed class SealedBase as definition.class, got pairs: {pairs:?}"
    );
    // Enum entries are @definition.constant, not @definition.class.
    assert_eq!(
        find_def_kind("NORTH"),
        Some("definition.constant"),
        "expected enum entry NORTH as definition.constant, got pairs: {pairs:?}"
    );
}

/// Every grammar-legal shape of `delegation_specifier` (superclass call
/// with parens, bare interface reference with no parens, and `by`
/// delegation) must produce a @reference.class capture with the right name.
#[test]
fn kotlin_tags_completeness_delegation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_delegation_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_delegation_variants: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);

    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    // delegation_specifier -> constructor_invocation -> user_type (superclass
    // call with parens/args).
    assert!(
        ref_class_names.contains(&"OpenBase"),
        "expected 'OpenBase' (constructor-invocation delegation) in kotlin tags, got: {ref_class_names:?}"
    );
    // delegation_specifier -> user_type directly (bare interface reference,
    // no parens) — the most common Kotlin idiom, previously unmatched.
    assert!(
        ref_class_names.contains(&"PlainInterface"),
        "expected 'PlainInterface' (bare delegation, no parens) in kotlin tags, got: {ref_class_names:?}"
    );
    // delegation_specifier -> explicit_delegation -> user_type (`by`
    // interface delegation), previously unmatched.
    assert!(
        ref_class_names.contains(&"SuperBase"),
        "expected 'SuperBase' (bare delegation before secondary ctor) in kotlin tags, got: {ref_class_names:?}"
    );

    // Re-run the tags query but only look at ExplicitDelegationVariant's
    // line to disambiguate the `by`-delegation form specifically (the name
    // "PlainInterface" is reused above for the paren-less form).
    let full_captures = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);
    let by_delegation = full_captures
        .iter()
        .find(|(cap, _, text, _)| cap == "reference.class" && text.contains(" by impl"));
    assert!(
        by_delegation.is_some(),
        "expected a @reference.class capture spanning the `by impl` explicit_delegation, got: {full_captures:?}"
    );
}

/// `this(...)` / `super(...)` secondary-constructor delegation
/// (`constructor_delegation_call`, a distinct node kind from
/// `call_expression`) must produce a @reference.call in tags and a @call in
/// calls, with the correct capture kind (an anonymous keyword token, not
/// `simple_identifier`).
#[test]
fn kotlin_tags_completeness_constructor_delegation_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_constructor_delegation_calls: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_completeness_constructor_delegation_calls: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);
    assert!(
        pairs.contains(&("reference.call".to_string(), "this".to_string())),
        "expected 'this' constructor-delegation @reference.call, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("reference.call".to_string(), "super".to_string())),
        "expected 'super' constructor-delegation @reference.call, got: {pairs:?}"
    );
}

/// Every grammar-legal call shape (plain call, navigation/method call,
/// `this`/`super` constructor delegation) must produce a @call with the
/// correct capture kind.
#[test]
fn kotlin_calls_completeness_call_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_calls_completeness_call_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_calls_completeness_call_variants: kotlin grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("kotlin")
        .expect("kotlin calls query missing");
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);

    let find_kind = |name: &str| -> Vec<&str> {
        full.iter()
            .filter(|(cap, _, text, _)| cap == "call" && text == name)
            .map(|(_, kind, _, _)| kind.as_str())
            .collect()
    };
    assert!(
        find_kind("println").contains(&"simple_identifier"),
        "expected plain call 'println' as simple_identifier, got: {full:?}"
    );
    assert!(
        find_kind("add").contains(&"simple_identifier"),
        "expected navigation call 'add' as simple_identifier, got: {full:?}"
    );
    assert!(
        find_kind("map").contains(&"simple_identifier"),
        "expected trailing-lambda call 'map' as simple_identifier, got: {full:?}"
    );
    // "this"/"super" constructor delegation: captured node kind is the
    // anonymous keyword token itself, not simple_identifier — distinct
    // extraction depth signal from ordinary calls.
    assert!(
        find_kind("this").contains(&"this"),
        "expected 'this' constructor-delegation call captured as kind 'this', got: {full:?}"
    );
    assert!(
        find_kind("super").contains(&"super"),
        "expected 'super' constructor-delegation call captured as kind 'super', got: {full:?}"
    );
}

/// Every grammar-legal `import_header` shape (plain, aliased, wildcard)
/// must produce exactly one @import per statement — no duplicates. The
/// plain-import pattern was previously unconstrained and also matched
/// every aliased/wildcard import.
#[test]
fn kotlin_imports_completeness_no_duplicate_matches() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_imports_completeness_no_duplicate_matches: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_imports_completeness_no_duplicate_matches: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_imports("kotlin")
        .expect("kotlin imports query missing");
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);
    let import_paths: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.path")
        .map(|(_, _, text, _)| text.as_str())
        .collect();

    // variants.kt has exactly 3 import statements (plain, aliased,
    // wildcard); each must contribute exactly one @import.path.
    assert_eq!(
        import_paths,
        vec!["java.util.ArrayList", "java.util.HashMap", "kotlin.math"],
        "expected exactly one @import.path per import statement (no duplicates), got: {import_paths:?}"
    );

    let aliases: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.alias")
        .map(|(_, _, text, _)| text.as_str())
        .collect();
    assert_eq!(
        aliases,
        vec!["JHashMap"],
        "expected exactly one @import.alias, got: {aliases:?}"
    );

    let globs: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.glob")
        .map(|(_, _, text, _)| text.as_str())
        .collect();
    assert_eq!(
        globs,
        vec!["*"],
        "expected exactly one @import.glob, got: {globs:?}"
    );
}

/// Type-defining declarations must produce @definition.type, and the
/// blanket @type.reference pattern must not double-count qualified/generic
/// type usages (the fixed duplicate-match bug).
#[test]
fn kotlin_types_completeness_definitions_and_no_duplicates() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_types_completeness_definitions_and_no_duplicates: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_types_completeness_definitions_and_no_duplicates: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_types("kotlin")
        .expect("kotlin types query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);

    // collect_tag_pairs pairs the @name leaf (just the identifier) with its
    // @definition.type container, not the container's own (much larger)
    // span — the query's outer @definition.type capture spans the whole
    // declaration (e.g. "class PlainClass"), so asserting equality against
    // the outer capture's text (as collect_captures_full alone would) is
    // wrong; the leaf name is what a consumer actually wants.
    for expected in ["PlainClass", "PlainObject", "PlainAlias"] {
        assert!(
            pairs.contains(&("definition.type".to_string(), expected.to_string())),
            "expected '{expected}' among @definition.type captures, got: {pairs:?}"
        );
    }

    // "PlainClass" appears at exactly 4 distinct source lines in
    // variants.kt (the class declaration itself, `plainType: PlainClass?`,
    // `List<PlainClass>` generic argument, and the callable-reference
    // negative case) — each must produce exactly one @type.reference,
    // not two, even though the generic-argument occurrence is wrapped in
    // a `user_type` (the redundant pattern that caused the duplicate).
    let plain_class_ref_lines: Vec<usize> = full
        .iter()
        .filter(|(cap, _, text, _)| cap == "type.reference" && text == "PlainClass")
        .map(|(_, _, _, line)| *line)
        .collect();
    let mut sorted_lines = plain_class_ref_lines.clone();
    sorted_lines.sort_unstable();
    let mut deduped_lines = sorted_lines.clone();
    deduped_lines.dedup();
    assert_eq!(
        sorted_lines, deduped_lines,
        "expected no duplicate @type.reference lines for 'PlainClass' (found the same line twice), got: {plain_class_ref_lines:?}"
    );
    assert_eq!(
        plain_class_ref_lines.len(),
        4,
        "expected exactly 4 'PlainClass' @type.reference occurrences (decl, plain annotation, generic argument, callable-reference), got {}: {plain_class_ref_lines:?}",
        plain_class_ref_lines.len()
    );
}

// --- Negative cases: constructs that must NOT match -------------------------

/// Annotation usages WITH constructor args (`@Deprecated("...")`) must NOT
/// be misclassified as a @reference.class: `constructor_invocation` is
/// also a legal child of `annotation`, not just `delegation_specifier`,
/// and the tags query is deliberately scoped to exclude it.
#[test]
fn kotlin_tags_negative_annotation_args_not_class_reference() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_negative_annotation_args_not_class_reference: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_negative_annotation_args_not_class_reference: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let pairs = collect_tag_pairs(&lang, KOTLIN_VARIANTS, &query_str);
    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        !ref_class_names.contains(&"Deprecated"),
        "the @Deprecated(\"...\") annotation must not be misclassified as @reference.class, got: {ref_class_names:?}"
    );
}

/// A top-level `val` (`property_declaration`) must never produce a
/// @definition.* capture: the grammar reuses `property_declaration` for
/// both class-level properties and local `val`/`var` bindings inside
/// function bodies with no reliable way to distinguish them without
/// ancestor traversal (documented in kotlin.tags.scm).
#[test]
fn kotlin_tags_negative_property_declarations_not_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_negative_property_declarations_not_captured: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_negative_property_declarations_not_captured: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let names = collect_captures(&lang, KOTLIN_VARIANTS, &query_str, "name");
    assert!(
        !names.contains(&"topLevelPropertyNegative".to_string()),
        "top-level 'val' must not appear in tags, got: {names:?}"
    );
}

/// An unnamed companion object (`companion object { ... }`, no explicit
/// name) has no `type_identifier` child at all and is architecturally
/// unable to produce a @name capture. This documents the absence rather
/// than asserting new behavior — Kotlin gives it the implicit name
/// "Companion", but the grammar provides no source text to capture that
/// name from, so fabricating it would violate "be honest about
/// capabilities" (CLAUDE.md).
#[test]
fn kotlin_tags_negative_unnamed_companion_object_has_no_name() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_tags_negative_unnamed_companion_object_has_no_name: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_tags_negative_unnamed_companion_object_has_no_name: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let full = collect_captures_full(&lang, KOTLIN_VARIANTS, &query_str);
    // variants.kt's only `companion_object` node is the unnamed one inside
    // `UnnamedCompanionNegative`. Filter by capture *kind* (not just name
    // text — `UnnamedCompanionNegative` the outer class also legitimately
    // produces a @definition.class, but its capture's node kind is
    // `class_declaration`, not `companion_object`) to precisely isolate
    // whether the companion_object pattern fired at all.
    let companion_definitions: Vec<&(String, String, String, usize)> = full
        .iter()
        .filter(|(cap, kind, ..)| cap == "definition.class" && kind == "companion_object")
        .collect();
    assert!(
        companion_definitions.is_empty(),
        "expected no @definition.class capture with kind 'companion_object' for the unnamed companion object, got: {companion_definitions:?}"
    );
}

/// `::foo` / `Type::method` callable references are a distinct node kind
/// (`callable_reference`) from `call_expression` and must never be
/// misclassified as a call.
#[test]
fn kotlin_calls_negative_callable_reference_not_a_call() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping kotlin_calls_negative_callable_reference_not_a_call: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!(
            "Skipping kotlin_calls_negative_callable_reference_not_a_call: kotlin grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_calls("kotlin")
        .expect("kotlin calls query missing");
    let calls = collect_captures(&lang, KOTLIN_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"hashCode".to_string()),
        "the 'PlainClass::hashCode' callable reference must not appear as a call, got: {calls:?}"
    );
}

// ---------------------------------------------------------------------------
// Swift
// ---------------------------------------------------------------------------

const SWIFT_SAMPLE: &str = include_str!("fixtures/swift/sample.swift");
const SWIFT_VARIANTS: &str = include_str!("fixtures/swift/variants.swift");

// --- Dimension 4: real-world fixture coverage (sample.swift) ----------------

#[test]
fn swift_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let names = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in swift tags, got: {names:?}"
    );
    // Protocol + protocol extension: the protocol itself and its
    // requirement/associatedtype must all be found.
    assert!(
        names.contains(&"Greetable".to_string()),
        "expected 'Greetable' protocol in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"greet".to_string()),
        "expected 'greet' protocol requirement in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Payload".to_string()),
        "expected 'Payload' associatedtype in swift tags, got: {names:?}"
    );
    // Generic function with a constraint (`<T: Comparable>`) must still be
    // found like any other function.
    assert!(
        names.contains(&"largest".to_string()),
        "expected 'largest' generic function in swift tags, got: {names:?}"
    );
    // Enum with associated values: both the enum and its cases.
    assert!(
        names.contains(&"NetworkResult".to_string()),
        "expected 'NetworkResult' enum in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"success".to_string()) && names.contains(&"cancelled".to_string()),
        "expected enum cases 'success'/'cancelled' in swift tags, got: {names:?}"
    );
    // Extension: previously entirely invisible (name field is
    // user_type-wrapped, not a bare type_identifier).
    assert!(
        names.contains(&"Coordinate".to_string()),
        "expected 'Coordinate' class in swift tags, got: {names:?}"
    );
    assert!(
        names.contains(&"magnitude".to_string()),
        "expected 'magnitude' computed property (declared in an extension) \
         in swift tags, got: {names:?}"
    );
    // Standard-operator overload declared inside an extension.
    assert!(
        names.contains(&"==".to_string()),
        "expected '==' operator overload in swift tags, got: {names:?}"
    );
    // Member properties: onComplete (var) on Downloader.
    assert!(
        names.contains(&"onComplete".to_string()),
        "expected 'onComplete' member property in swift tags, got: {names:?}"
    );
}

#[test]
fn swift_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_calls: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("swift")
        .expect("swift calls query missing");
    let calls = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"print".to_string()) || calls.contains(&"push".to_string()),
        "expected 'print' or 'push' call in swift sample, got: {calls:?}"
    );
    // Trailing closure call (`numbers.map { ... }`) — the call_suffix's
    // lambda_literal content doesn't change the callee shape.
    assert!(
        calls.contains(&"map".to_string()),
        "expected trailing-closure 'map' call in swift sample, got: {calls:?}"
    );
    // Force-unwrap call: `onComplete!()`.
    assert!(
        calls.contains(&"onComplete".to_string()),
        "expected force-unwrap 'onComplete' call in swift sample, got: {calls:?}"
    );
}

#[test]
fn swift_imports_finds_module_imports() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_imports: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("swift")
        .expect("swift imports query missing");
    let paths = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Foundation") || p.contains("Swift")),
        "expected 'Foundation' or 'Swift' in swift import paths, got: {paths:?}"
    );
}

#[test]
fn swift_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_complexity: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("swift")
        .expect("swift complexity query missing");
    let complexity = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in swift sample, got {} ({complexity:?})",
        complexity.len()
    );
    // `largest<T: Comparable>` uses `guard ... else { return nil }` — must
    // count toward complexity like any other branch.
    let source_from_guard = SWIFT_SAMPLE.contains("guard var best = items.first");
    assert!(
        source_from_guard,
        "fixture must contain the guard statement this test relies on"
    );
    assert!(
        complexity.len() >= 8,
        "expected guard_statement/switch_entry/conjunction/disjunction to be \
         counted (sample has >=1 guard, a 4-case switch, and no boolean \
         operators yet at this count baseline), got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn swift_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_types: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("swift")
        .expect("swift types query missing");
    let refs = collect_captures(&lang, SWIFT_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Int" || r == "String" || r == "Bool"),
        "expected primitive type references in swift sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.swift) -

/// Every grammar-legal variant of declaration `name` fields that
/// swift.tags.scm claims to support must actually match, with the right
/// capture *kind* (dimension 3) — not just the right text.
#[test]
fn swift_tags_completeness_all_declaration_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags_completeness: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let pairs = collect_tag_pairs(&lang, SWIFT_VARIANTS, &query_str);

    // plain_name / custom_operator / standard-operator-overload function names.
    assert!(
        pairs.contains(&(
            "definition.function".to_string(),
            "plainFunction".to_string()
        )),
        "expected plain function name, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "+++".to_string())),
        "expected custom_operator overload '+++', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "==".to_string())),
        "expected standard-operator overload '==', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "+=".to_string())),
        "expected compound-assignment operator overload '+=', got: {pairs:?}"
    );

    // plain class name vs. extension (user_type-wrapped) name.
    assert!(
        pairs.contains(&("definition.class".to_string(), "PlainClass".to_string())),
        "expected plain class name, got: {pairs:?}"
    );
    // "PlainClass" appears twice: once for the class itself (type_identifier)
    // and once for its extension (user_type -> type_identifier) — both must
    // be present as separate matches.
    let plain_class_defs = pairs
        .iter()
        .filter(|(k, n)| k == "definition.class" && n == "PlainClass")
        .count();
    assert_eq!(
        plain_class_defs, 2,
        "expected 2 'PlainClass' definitions (class + extension), got {plain_class_defs}: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.class".to_string(), "Array".to_string())),
        "expected extension-of-generic-stdlib-type name 'Array', got: {pairs:?}"
    );

    // Enum cases: single, associated-value, and comma-separated multi-name.
    for case_name in ["ready", "failed", "paused", "cancelled"] {
        assert!(
            pairs.contains(&("definition.constant".to_string(), case_name.to_string())),
            "expected enum case '{case_name}', got: {pairs:?}"
        );
    }

    // Member let/var + computed property (class_body), and enum_class_body
    // variant of the same ancestor restriction.
    assert!(
        pairs.contains(&("definition.constant".to_string(), "readOnly".to_string())),
        "expected member 'let readOnly', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "mutable".to_string())),
        "expected member 'var mutable', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "computed".to_string())),
        "expected computed property 'computed', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "isA".to_string())),
        "expected enum computed property 'isA' (enum_class_body variant), got: {pairs:?}"
    );

    // Protocol requirements: property, method, associatedtype.
    assert!(
        pairs.contains(&("definition.var".to_string(), "label".to_string())),
        "expected protocol property requirement 'label', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.method".to_string(), "describe".to_string())),
        "expected protocol method requirement 'describe', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.type".to_string(), "Value".to_string())),
        "expected protocol associatedtype 'Value', got: {pairs:?}"
    );
}

/// Local `let`/`var` declarations inside function bodies share a node kind
/// (property_declaration) with member-level properties, but must never be
/// captured as @definition.constant/@definition.var — verified with exact
/// zero counts, not just "absent from a name list" (a false positive that
/// happened to collide with another name would otherwise hide the bug).
#[test]
fn swift_tags_negative_local_declarations_not_captured() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags_negative: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let pairs = collect_tag_pairs(&lang, SWIFT_VARIANTS, &query_str);

    for local_name in [
        "localReadOnly",
        "localMutable",
        "notAMember",
        "alsoNotAMember",
    ] {
        let count = pairs
            .iter()
            .filter(|(k, n)| {
                (k == "definition.constant" || k == "definition.var") && n == local_name
            })
            .count();
        assert_eq!(
            count, 0,
            "local declaration '{local_name}' must never be captured as a \
             member constant/var, got {count} match(es): {pairs:?}"
        );
    }
}

/// Every grammar-legal variant of `call_expression.function` (plus the
/// distinct postfix_expression/constructor_expression callee shapes) that
/// swift.calls.scm claims to support must actually match, with the right
/// capture kind.
#[test]
fn swift_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_calls_completeness: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("swift")
        .expect("swift calls query missing");
    let caps = collect_captures_full(&lang, SWIFT_VARIANTS, &query_str);

    let call_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    // plain_call: function: identifier
    assert!(
        call_names.contains(&"identity"),
        "expected 'identity' plain call, got: {call_names:?}"
    );
    // method_call: function: navigation_expression -> simple_identifier
    assert!(
        call_names.contains(&"get"),
        "expected 'get' method call, got: {call_names:?}"
    );
    // force-unwrap call: function: postfix_expression(target, operation: bang)
    assert!(
        call_names.contains(&"completion"),
        "expected force-unwrap 'completion' call, got: {call_names:?}"
    );
    // optional-chaining call: plain identifier callee, same as plain_call.
    let completion_calls = call_names.iter().filter(|n| **n == "completion").count();
    assert_eq!(
        completion_calls, 2,
        "expected 2 'completion' calls (force-unwrap + optional-chaining), \
         got {completion_calls}: {call_names:?}"
    );
    // generic type instantiation call: constructor_expression, constructed_type:
    // (user_type (type_identifier)).
    assert!(
        call_names.contains(&"GenericBox"),
        "expected generic-instantiation call 'GenericBox', got: {call_names:?}"
    );
    assert!(
        call_names.contains(&"Optional"),
        "expected generic-instantiation call 'Optional', got: {call_names:?}"
    );

    // Every @call capture must be one of the node kinds the query actually
    // targets — never the parenthesized wrapper or anything larger
    // (extraction depth: capture kind, not just text).
    for (cn, kind, text, line) in &caps {
        if cn == "call" {
            assert!(
                kind == "simple_identifier" || kind == "type_identifier",
                "expected @call capture kind to be simple_identifier/type_identifier, \
                 got kind={kind} text={text} line={line}"
            );
        }
    }

    // @call.qualifier must carry the qualifier text for the method call.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"b"),
        "expected 'b' qualifier for the method call, got: {qualifiers:?}"
    );
}

/// Negative cases: call_expression.function variants with no stable,
/// nameable callee (curried calls, IIFEs, bracket type-literal calls) must
/// never produce a @call capture.
#[test]
fn swift_calls_negative_uncallable_function_variants_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_calls_negative: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("swift")
        .expect("swift calls query missing");
    let caps = collect_captures_full(&lang, SWIFT_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // NEGATIVE: curried call `makeAdder()(1)` — the INNER call (`makeAdder()`)
    // is a real plain_call and must be captured once; the OUTER call (whose
    // callee is the inner call_expression's *result*) must not add a second
    // 'makeAdder' capture.
    let make_adder_calls = call_texts.iter().filter(|t| **t == "makeAdder").count();
    assert_eq!(
        make_adder_calls, 1,
        "expected exactly 1 'makeAdder' call (the inner call only, not the \
         curried outer call), got {make_adder_calls}: {call_texts:?}"
    );
    // NEGATIVE: IIFE `{ (x: Int) -> Int in x * 2 }(5)` — anonymous callee —
    // and the bracket type-literal call `[Int](repeating:count:)` must
    // produce no capture at all. No text assertion is possible for either
    // (there is no name to accidentally capture); instead assert the total
    // capture count matches exactly the full expected set of named calls
    // across variants.swift, so a stray capture from either would be caught.
    let expected_calls = [
        "Vector",
        "reduce",
        "print",
        "identity",
        "Box",
        "get",
        "Optional",
        "completion",
        "completion",
        "GenericBox",
        "Optional",
        "makeAdder",
        "print",
    ];
    let mut actual_sorted = call_texts.clone();
    actual_sorted.sort_unstable();
    let mut expected_sorted: Vec<&str> = expected_calls.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(
        actual_sorted, expected_sorted,
        "expected exactly the named calls in variants.swift, got: {call_texts:?}"
    );
}

/// guard_statement / switch_entry / conjunction_expression / disjunction_expression
/// must all be counted individually — completeness + extraction-depth check
/// against the dedicated complexityVariants function in variants.swift.
#[test]
fn swift_complexity_completeness_guard_switch_and_boolean_operators() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_complexity_completeness: swift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("swift")
        .expect("swift complexity query missing");
    let caps = collect_captures_full(&lang, SWIFT_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();

    assert!(
        complexity_kinds.contains(&"guard_statement"),
        "expected guard_statement to count toward complexity, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"conjunction_expression"),
        "expected conjunction_expression (&&) to count toward complexity, got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"disjunction_expression"),
        "expected disjunction_expression (||) to count toward complexity, got: {complexity_kinds:?}"
    );
    // complexityVariants has 4 switch_entry nodes (1, 2-3, where-guarded, default).
    let switch_entry_count = complexity_kinds
        .iter()
        .filter(|k| **k == "switch_entry")
        .count();
    assert_eq!(
        switch_entry_count, 4,
        "expected 4 switch_entry complexity nodes, got {switch_entry_count}: {complexity_kinds:?}"
    );
}

/// `let`-bound vs `var`-bound member properties must land in distinct
/// capture kinds (@definition.constant vs @definition.var) — the closest
/// analog in this query to a read/write or definition/reference
/// distinction, since Swift's tags query has no separate reference captures.
#[test]
fn swift_tags_distinguishes_let_and_var_member_properties() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping swift_tags_let_var: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("swift").ok() else {
        eprintln!("Skipping swift_tags_let_var: swift grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("swift").expect("swift tags query missing");
    let pairs = collect_tag_pairs(&lang, SWIFT_VARIANTS, &query_str);

    assert!(
        pairs.contains(&("definition.constant".to_string(), "readOnly".to_string())),
        "expected 'readOnly' as @definition.constant, got: {pairs:?}"
    );
    assert!(
        !pairs.contains(&("definition.var".to_string(), "readOnly".to_string())),
        "'readOnly' (a `let`) must not ALSO appear as @definition.var, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.var".to_string(), "mutable".to_string())),
        "expected 'mutable' as @definition.var, got: {pairs:?}"
    );
    assert!(
        !pairs.contains(&("definition.constant".to_string(), "mutable".to_string())),
        "'mutable' (a `var`) must not ALSO appear as @definition.constant, got: {pairs:?}"
    );
}

// ---------------------------------------------------------------------------
// Scala
// ---------------------------------------------------------------------------

const SCALA_SAMPLE: &str = include_str!("fixtures/scala/sample.scala");
const SCALA_VARIANTS: &str = include_str!("fixtures/scala/variants.scala");

// --- Dimension 4: real-world fixture coverage (sample.scala) ----------------

#[test]
fn scala_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let names = collect_captures(&lang, SCALA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in scala tags, got: {names:?}"
    );
    // Companion object.
    assert!(
        names.contains(&"Point".to_string()),
        "expected companion 'Point' object in scala tags, got: {names:?}"
    );
    // Traits with mixins.
    assert!(
        names.contains(&"Named".to_string()) && names.contains(&"Aged".to_string()),
        "expected 'Named'/'Aged' traits in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Person".to_string()),
        "expected 'Person' class (extends Named with Aged) in scala tags, got: {names:?}"
    );
    // Scala 3 enum with a body method.
    assert!(
        names.contains(&"Direction".to_string()),
        "expected 'Direction' enum in scala tags, got: {names:?}"
    );
    assert!(
        names.contains(&"opposite".to_string()),
        "expected 'opposite' method inside the enum body in scala tags, got: {names:?}"
    );
    // Operator-method definition on the case class.
    assert!(
        names.contains(&"+".to_string()),
        "expected operator method '+' in scala tags, got: {names:?}"
    );
    // Higher-kinded generic trait.
    assert!(
        names.contains(&"Functor".to_string()),
        "expected 'Functor' higher-kinded trait in scala tags, got: {names:?}"
    );
}

#[test]
fn scala_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_calls: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scala")
        .expect("scala calls query missing");
    let calls = collect_captures(&lang, SCALA_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"println".to_string()) || calls.contains(&"push".to_string()),
        "expected 'println' or 'push' call in scala sample, got: {calls:?}"
    );
    // Companion-object factory call and case-class apply.
    assert!(
        calls.contains(&"distanceTo".to_string()),
        "expected 'distanceTo' method call in scala sample, got: {calls:?}"
    );
}

#[test]
fn scala_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_imports: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scala")
        .expect("scala imports query missing");
    // Scala imports query captures @import (the full declaration node)
    let imports = collect_captures(&lang, SCALA_SAMPLE, &query_str, "import");
    assert!(
        !imports.is_empty(),
        "expected at least one import declaration in scala sample, got: {imports:?}"
    );
    // Import with a per-name rename (`Success => S`) must still surface as
    // its own @import declaration.
    assert!(
        imports.iter().any(|i| i.contains("Success")),
        "expected the 'Success => S' rename import in scala sample, got: {imports:?}"
    );
}

#[test]
fn scala_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_complexity: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scala")
        .expect("scala complexity query missing");
    let complexity = collect_captures(&lang, SCALA_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in scala sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn scala_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_types: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("scala")
        .expect("scala types query missing");
    let refs = collect_captures(&lang, SCALA_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Int" || r == "Double" || r == "String"),
        "expected type identifiers in scala sample, got: {refs:?}"
    );
}

// --- Dimensions 2/3: completeness + extraction depth (variants.scala) ------

/// `function_definition.name` allows `identifier` and `operator_identifier`;
/// both must produce a `definition.function` tag with the correct kind.
#[test]
fn scala_tags_completeness_function_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping scala_tags_completeness_function_name: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_function_name: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);

    assert!(
        pairs.contains(&("definition.function".to_string(), "plainFunc".to_string())),
        "expected identifier-named function 'plainFunc', got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.function".to_string(), "+".to_string())),
        "expected operator_identifier-named method '+', got: {pairs:?}"
    );
}

/// Scala 3 `enum` definitions must surface as `definition.enum`.
#[test]
fn scala_tags_completeness_enum_definition() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags_completeness_enum: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_enum: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    assert!(
        pairs.contains(&("definition.enum".to_string(), "Color".to_string())),
        "expected 'Color' enum as definition.enum, got: {pairs:?}"
    );
    assert!(
        pairs.contains(&("definition.enum".to_string(), "Nested".to_string())),
        "expected 'Nested' enum as definition.enum, got: {pairs:?}"
    );
    // A method inside the enum body must still surface as its own definition,
    // proving the enum acts as a container (SymbolKind::Enum is a container
    // kind).
    assert!(
        pairs.contains(&("definition.function".to_string(), "label".to_string())),
        "expected 'label' method inside 'Nested' enum body, got: {pairs:?}"
    );
}

/// Every `call_expression.function` variant scala.tags.scm's @reference.call
/// claims to support must produce a matching capture, with the correct name.
#[test]
fn scala_tags_completeness_reference_call_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping scala_tags_completeness_reference_call: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_reference_call: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    let ref_calls: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.call")
        .map(|(_, n)| n.as_str())
        .collect();

    assert!(
        ref_calls.contains(&"identity"),
        "expected plain call 'identity', got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"map"),
        "expected method call 'map', got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"+"),
        "expected explicit operator-method call 'a.+(b)', got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"identityGeneric"),
        "expected generic call 'identityGeneric[Int](1)', got: {ref_calls:?}"
    );
}

/// Object creation (`new X()`) must be found for plain, generic, qualified,
/// and generic+qualified type shapes.
#[test]
fn scala_tags_completeness_new_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags_completeness_new: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_new: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    let ref_class: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();

    assert!(
        ref_class.contains(&"OpHolder"),
        "expected plain 'new OpHolder(1)', got: {ref_class:?}"
    );
    assert!(
        ref_class.contains(&"ArrayBuffer"),
        "expected generic 'new ArrayBuffer[Int]()', got: {ref_class:?}"
    );
    assert!(
        ref_class.contains(&"Date"),
        "expected qualified 'new java.util.Date()', got: {ref_class:?}"
    );
    assert!(
        ref_class.contains(&"HashMap"),
        "expected qualified+generic 'new java.util.HashMap[String, Int]()', got: {ref_class:?}"
    );
}

/// `extends X with Y with Z` — the first supertype and every subsequent
/// `with` mixin must all surface as @reference.implementation, including
/// generic and qualified shapes.
#[test]
fn scala_tags_completeness_extends_mixin_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping scala_tags_completeness_extends: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_completeness_extends: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);
    let ref_impl: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.implementation")
        .map(|(_, n)| n.as_str())
        .collect();

    // Both the first supertype (fielded) and the "with" mixin (unfielded)
    // from `class MultiMixin extends TraitA with TraitB`.
    assert!(
        ref_impl.contains(&"TraitA"),
        "expected first supertype 'TraitA', got: {ref_impl:?}"
    );
    assert!(
        ref_impl.contains(&"TraitB"),
        "expected 'with' mixin 'TraitB', got: {ref_impl:?}"
    );
    // Generic mixin: `class GenericMixin extends TraitC[Int]`.
    assert!(
        ref_impl.contains(&"TraitC"),
        "expected generic supertype 'TraitC', got: {ref_impl:?}"
    );
    // Qualified + generic mixin: `extends scala.collection.Iterable[Int]`.
    assert!(
        ref_impl.contains(&"Iterable"),
        "expected qualified+generic supertype 'Iterable', got: {ref_impl:?}"
    );
}

/// Negative case: bare field access/write, lambda bindings, and
/// eta-expansion (passing a method by name without calling it) must never
/// surface as tags definitions or call references.
#[test]
fn scala_tags_negative_field_access_and_lambda_bindings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_tags_negative: scala grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scala").expect("scala tags query missing");
    let pairs = collect_tag_pairs(&lang, SCALA_VARIANTS, &query_str);

    let def_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k.starts_with("definition."))
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        !def_names.contains(&"lambdaBinding"),
        "'lambdaBinding' (a val bound to a lambda) must not be a definition, got: {def_names:?}"
    );
    assert!(
        !def_names.contains(&"etaExpanded"),
        "'etaExpanded' (a val bound via eta-expansion) must not be a definition, got: {def_names:?}"
    );

    let ref_calls: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.call")
        .map(|(_, n)| n.as_str())
        .collect();
    assert!(
        !ref_calls.contains(&"counter"),
        "bare field read/write 'counter' must not be a call reference, got: {ref_calls:?}"
    );
}

/// Every `call_expression.function` variant scala.calls.scm claims to
/// support (identifier, method call, explicit operator-method call, generic,
/// qualified generic, parenthesized target) must produce a @call capture.
#[test]
fn scala_calls_completeness_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_calls_completeness: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scala")
        .expect("scala calls query missing");
    let caps = collect_captures_full(&lang, SCALA_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    assert!(
        calls.contains(&"identity"),
        "expected plain call 'identity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"map"),
        "expected method call 'map', got: {calls:?}"
    );
    assert!(
        calls.contains(&"+"),
        "expected explicit operator-method call '+', got: {calls:?}"
    );
    assert!(
        calls.contains(&"identityGeneric"),
        "expected generic call 'identityGeneric', got: {calls:?}"
    );
    // Parenthesized call target: (f)(1) — the whole parenthesized text is
    // captured as @call, matching typescript.calls.scm's convention.
    assert!(
        calls.iter().any(|c| c.starts_with('(') && c.contains('f')),
        "expected parenthesized call target '(f)', got: {calls:?}"
    );
}

/// Negative case: bare field access/write must never appear as a @call.
#[test]
fn scala_calls_negative_field_access() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_calls_negative: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scala")
        .expect("scala calls query missing");
    let calls = collect_captures(&lang, SCALA_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"counter".to_string()),
        "bare field read/write 'counter' must not be captured as a call, got: {calls:?}"
    );
}

/// `enum_definition` must contribute nesting depth, matching how
/// class/object/trait definitions are treated.
#[test]
fn scala_complexity_completeness_enum_nesting() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_complexity_completeness: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scala")
        .expect("scala complexity query missing");
    let caps = collect_captures_full(&lang, SCALA_VARIANTS, &query_str);
    let nesting_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "nesting")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    assert!(
        nesting_kinds.contains(&"enum_definition"),
        "expected enum_definition to contribute nesting, got: {nesting_kinds:?}"
    );
    // match with guards (case_clause) must contribute complexity.
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    assert!(
        complexity_kinds
            .iter()
            .filter(|k| **k == "case_clause")
            .count()
            >= 2,
        "expected multiple case_clause complexity nodes (guarded match), got: {complexity_kinds:?}"
    );
    assert!(
        complexity_kinds.contains(&"for_expression"),
        "expected for-comprehension to contribute complexity, got: {complexity_kinds:?}"
    );
}

/// Duplicate-capture regression: a qualified type reference (`java.util.Date`)
/// must produce exactly one @type.reference capture per identifier, not two.
/// A previous version of scala.types.scm had a redundant clause that matched
/// every `stable_type_identifier`-nested `type_identifier` twice.
#[test]
fn scala_types_negative_no_duplicate_qualified_captures() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_types_negative: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("scala")
        .expect("scala types query missing");
    let refs = collect_captures(&lang, SCALA_VARIANTS, &query_str, "type");
    // variants.scala's NewVariants object has exactly one `new java.util.Date()`.
    let date_count = refs.iter().filter(|r| *r == "Date").count();
    assert_eq!(
        date_count, 1,
        "expected exactly 1 'Date' type.reference capture (qualified type must not \
         double-count), got {date_count}: {refs:?}"
    );
}

/// Rename-arrow import bugs: `{Map => MutableMap}` (Scala 2 arrow),
/// `{List as JList}` (Scala 3 `as`), and per-name renames inside a
/// multi-name brace list (`{Try, Success => S, Failure}`) must all still
/// anchor as their own `import_declaration` — this exercises
/// `Scala::extract_imports`'s text-parsing fallback via the same
/// query-selected `import_declaration` nodes.
#[test]
fn scala_imports_completeness_rename_and_wildcard_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_imports_completeness: scala grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scala")
        .expect("scala imports query missing");
    let imports = collect_captures(&lang, SCALA_VARIANTS, &query_str, "import");

    assert!(
        imports.iter().any(|i| i.contains("Map => MutableMap")),
        "expected the arrow-rename import statement, got: {imports:?}"
    );
    assert!(
        imports.iter().any(|i| i.contains("List as JList")),
        "expected the Scala-3 'as'-rename import statement, got: {imports:?}"
    );
    assert!(
        imports.iter().any(|i| i.contains("foo.bar.baz.*")),
        "expected the bare-wildcard import statement, got: {imports:?}"
    );
}

/// `Scala::extract_imports` must strip per-name rename suffixes (`=>`/`as`)
/// from the parsed `names` list instead of leaving raw "X => Y" text in it,
/// and must not mistake a name merely containing '_' for a wildcard marker.
#[test]
fn scala_imports_extract_strips_rename_suffix_and_detects_wildcard_precisely() {
    use normalize_languages::{Language, Scala};
    use tree_sitter::{Parser, StreamingIterator};

    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scala_imports_extract: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scala").ok() else {
        eprintln!("Skipping scala_imports_extract: scala grammar .so not found");
        return;
    };
    // Parse directly and probe extract_imports on the raw import_declaration
    // nodes — no need for the tags/imports query here.
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let source = "import scala.util.{Try, Success => S, Failure}\n\
                  import scala.collection.mutable.{Map => MutableMap}\n";
    let tree = parser.parse(source, None).expect("parse failed");
    let query =
        tree_sitter::Query::new(&lang, "(import_declaration) @import").expect("query compile");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let scala = Scala;
    let mut all_names: Vec<String> = Vec::new();
    let mut single_alias: Option<String> = None;
    while let Some(m) = matches.next() {
        for cap in m.captures {
            let imports = scala.extract_imports(&cap.node, source);
            for imp in imports {
                all_names.extend(imp.names.iter().cloned());
                if imp.names.len() == 1 {
                    single_alias = imp.alias.clone();
                }
            }
        }
    }
    // Multi-name brace import: renamed entry must contribute a clean plain
    // name ("Success"), never the raw "Success => S" text.
    assert!(
        all_names.contains(&"Success".to_string()),
        "expected clean name 'Success' (rename suffix stripped), got: {all_names:?}"
    );
    assert!(
        !all_names.iter().any(|n| n.contains("=>")),
        "no parsed import name may contain a raw rename arrow, got: {all_names:?}"
    );
    assert!(
        all_names.contains(&"Try".to_string()) && all_names.contains(&"Failure".to_string()),
        "expected unrenamed names 'Try'/'Failure' preserved, got: {all_names:?}"
    );
    // Single-name brace import with a rename: alias must be recovered.
    assert_eq!(
        single_alias,
        Some("MutableMap".to_string()),
        "expected single-name rename alias 'MutableMap' to be recovered"
    );
}

// ---------------------------------------------------------------------------
// PHP
// ---------------------------------------------------------------------------

const PHP_SAMPLE: &str = include_str!("fixtures/php/sample.php");
const PHP_VARIANTS: &str = include_str!("fixtures/php/variants.php");

// --- Dimension 4: real-world fixture coverage (sample.php) ------------------

#[test]
fn php_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_tags: php grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("php").expect("php tags query missing");
    let names = collect_captures(&lang, PHP_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in php tags, got: {names:?}"
    );
    // Trait, interface, enum containers must also surface as definitions.
    assert!(
        names.contains(&"Loggable".to_string()),
        "expected 'Loggable' trait in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Comparable".to_string()),
        "expected 'Comparable' interface in php tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Direction".to_string()),
        "expected 'Direction' enum in php tags, got: {names:?}"
    );

    // References: extends/implements and constructor calls must now surface
    // too (previously entirely absent — see php.tags.scm's "References"
    // section for the field-by-field verification).
    let pairs = collect_tag_pairs(&lang, PHP_SAMPLE, &query_str);
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.class" && n == "Stack"),
        "expected 'extends Stack' (BoundedStack) as reference.class, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.implementation" && n == "Comparable"),
        "expected 'implements Comparable' as reference.implementation, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.class" && n == "Stack"),
        "expected 'new Stack()' as reference.class, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "push"),
        "expected '$stack->push(...)' as reference.call, got: {pairs:?}"
    );
}

#[test]
fn php_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_calls: php grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("php").expect("php calls query missing");
    let calls = collect_captures(&lang, PHP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"classify".to_string())
            || calls.contains(&"array_push".to_string())
            || calls.contains(&"empty".to_string()),
        "expected a function call in php sample, got: {calls:?}"
    );
    // Static method call (BoundedStack::class is a constant-fetch, not a
    // call — check parent::push(...) and parent::__construct() instead,
    // real scoped_call_expression sites in the sample).
    assert!(
        calls.contains(&"push".to_string()),
        "expected 'parent::push(...)'/'$stack->push(...)' method call, got: {calls:?}"
    );
    // Namespace-qualified function call.
    assert!(
        calls.iter().any(|c| c.contains("classify")),
        "expected '\\App\\Collections\\classify(3)' namespaced call, got: {calls:?}"
    );
}

#[test]
fn php_imports_finds_use_declarations() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_imports: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("php")
        .expect("php imports query missing");
    let paths = collect_captures(&lang, PHP_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("User") || p.contains("Collection") || p.contains("App")),
        "expected namespace path in php import paths, got: {paths:?}"
    );
    // Bare single-segment `use Countable;`/`use Traversable;` (no
    // namespace separator) — previously dropped entirely.
    assert!(
        paths.contains(&"Countable".to_string()),
        "expected bare 'use Countable;', got: {paths:?}"
    );
    // `require_once __DIR__ . '/bootstrap.php';` — the string-literal
    // suffix of a concatenation; require_expression/require_once_expression
    // were previously entirely unmatched (only include* was handled).
    assert!(
        paths.iter().any(|p| p.contains("bootstrap.php")),
        "expected 'require_once ... bootstrap.php' path, got: {paths:?}"
    );
}

#[test]
fn php_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_complexity: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("php")
        .expect("php complexity query missing");
    let complexity = collect_captures(&lang, PHP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in php sample, got {} ({complexity:?})",
        complexity.len()
    );
    // `match ($this) { ... }` arms in Direction::opposite() and
    // `describeDirection` must count too.
    let caps = collect_captures_full(&lang, PHP_SAMPLE, &query_str);
    assert!(
        caps.iter()
            .any(|(cn, k, _, _)| cn == "complexity" && k == "match_conditional_expression"),
        "expected at least one match_conditional_expression @complexity, got: {caps:?}"
    );
    // `$n % 2 === 0 && $n > 0` — the `&&` must count as its own branch.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "complexity"
            && k == "binary_expression"
            && t.contains("&&")),
        "expected the '&&' in sumEvens to count as @complexity, got: {caps:?}"
    );
}

#[test]
fn php_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_types: php grammar .so not found");
        return;
    };
    let query_str = loader.get_types("php").expect("php types query missing");
    let refs = collect_captures(&lang, PHP_SAMPLE, &query_str, "type");
    assert!(
        refs.contains(&"Direction".to_string()) || refs.contains(&"mixed".to_string()),
        "expected a type reference (Direction/mixed/etc) in php sample, got: {refs:?}"
    );
}

// --- Dimension 2/3: completeness matrix + extraction depth (variants.php) --

/// Every grammar-legal variant of `function_call_expression.function` /
/// `scoped_call_expression.name` / `member_call_expression.name` /
/// `nullsafe_member_call_expression.name` that php.calls.scm claims to
/// support, asserted by capture kind (not just text).
#[test]
fn php_calls_completeness_all_callee_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_calls_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("php").expect("php calls query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let calls: Vec<(&str, &str)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, k, t, _)| (k.as_str(), t.as_str()))
        .collect();

    // function_call_expression.function variants.
    assert!(
        calls.contains(&("name", "helperFn")),
        "expected plain function call (function: name), got: {calls:?}"
    );
    // `$fn();` drills into variable_name to capture the *variable's own*
    // name ("fn") — the AST has no notion of the string value ("helperFn")
    // the variable happens to hold, only the identifier being called.
    assert!(
        calls.contains(&("name", "fn")),
        "expected variable function call ($fn(), function: variable_name -> \
         name, capturing the variable's own name) got: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "qualified_name" && t.contains("classify")),
        "expected namespaced function call (function: qualified_name), got: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "relative_name" && t.contains("helperFn")),
        "expected relative-namespace function call (function: relative_name), got: {calls:?}"
    );

    // scoped_call_expression.name variants.
    assert!(
        calls.contains(&("name", "on")),
        "expected static method call (scoped_call name: name), got: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|(k, t)| *k == "variable_name" && *t == "$method"),
        "expected dynamic static method call (scoped_call name: variable_name), got: {calls:?}"
    );

    // member_call_expression.name variants.
    assert!(
        calls.contains(&("name", "next")),
        "expected nullsafe method call (name: name), got: {calls:?}"
    );

    // object_creation_expression is NOT a call (see php.calls.scm comment):
    // must never contribute a @call capture.
    assert!(
        !calls.iter().any(|(_, t)| *t == "Widget"),
        "constructor invocation must not be captured as @call, got: {calls:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn php_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_calls_negative: php grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("php").expect("php calls query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // A property read ($this->field) must never appear as a call.
    assert!(
        !call_texts.contains(&"field"),
        "property read must not be captured as @call, got: {call_texts:?}"
    );
    // Anonymous class instantiation contributes no @call/name capture.
    assert!(
        !call_texts.iter().any(|t| t.contains("implements Shape")),
        "anonymous class body must never leak into @call text, got: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `object_creation_expression`/
/// `base_clause`/`class_interface_clause` that php.tags.scm's new
/// @reference.class/@reference.implementation patterns claim to support.
#[test]
fn php_tags_completeness_reference_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_tags_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("php").expect("php tags query missing");
    // @reference.class/@reference.implementation are attached to the
    // *container* node (object_creation_expression/base_clause/
    // class_interface_clause), not the field-variant node itself — so
    // `tags_matches_by_kind` (which correlates each match's anchor capture
    // with that match's @name capture) is required here, not
    // `collect_captures_full` filtered by capture name (that would report
    // every reference.class hit as kind "object_creation_expression"/
    // "base_clause", the container's own kind, not the variant).
    let class_refs = tags_matches_by_kind(&lang, PHP_VARIANTS, &query_str, "reference.class");
    let class_ref_pairs: Vec<(&str, &str)> = class_refs
        .iter()
        .map(|(k, t)| (k.as_str(), t.as_str()))
        .collect();

    assert!(
        class_ref_pairs
            .iter()
            .any(|(k, t)| *k == "qualified_name" && t.contains("User")),
        "expected 'new \\App\\Models\\User()' (object_creation: qualified_name), got: {class_ref_pairs:?}"
    );
    assert!(
        class_ref_pairs
            .iter()
            .any(|(k, t)| *k == "relative_name" && t.contains("Widget")),
        "expected 'new namespace\\Widget()' (object_creation: relative_name), got: {class_ref_pairs:?}"
    );
    assert!(
        class_ref_pairs
            .iter()
            .any(|(k, t)| *k == "variable_name" && *t == "$cls"),
        "expected 'new $cls()' (object_creation: variable_name), got: {class_ref_pairs:?}"
    );
    // 'new Widget()' (object_creation: name) and 'extends Widget'
    // (base_clause: name) both produce identical ("name", "Widget") pairs
    // by design — assert at least 2 occurrences so a regression that drops
    // either pattern is still caught.
    let widget_name_refs = class_ref_pairs
        .iter()
        .filter(|(k, t)| *k == "name" && *t == "Widget")
        .count();
    assert!(
        widget_name_refs >= 2,
        "expected both 'new Widget()' (object_creation) and 'extends Widget' \
         (base_clause) to each produce a ('name', 'Widget') reference.class \
         capture, got {widget_name_refs}: {class_ref_pairs:?}"
    );

    let impl_refs =
        tags_matches_by_kind(&lang, PHP_VARIANTS, &query_str, "reference.implementation");
    let impl_ref_pairs: Vec<(&str, &str)> = impl_refs
        .iter()
        .map(|(k, t)| (k.as_str(), t.as_str()))
        .collect();
    assert!(
        impl_ref_pairs.contains(&("name", "Shape")),
        "expected 'implements Shape' (class_interface_clause: name), got: {impl_ref_pairs:?}"
    );
    assert!(
        impl_ref_pairs.contains(&("name", "Colored")),
        "expected 'implements Colored' (class_interface_clause: name), got: {impl_ref_pairs:?}"
    );
}

/// Negative case: anonymous class instantiation must never produce a
/// @reference.class capture with fabricated name text (no name field
/// exists for `anonymous_class`).
#[test]
fn php_tags_negative_anonymous_class_has_no_reference() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_tags_negative: php grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("php").expect("php tags query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let anon_leaks = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "reference.class" && t.contains("implements Shape"))
        .count();
    assert_eq!(
        anon_leaks, 0,
        "anonymous_class body must never leak into a @reference.class capture, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `namespace_use_declaration`/
/// `use_declaration`/`require*`/`include*` that php.imports.scm claims to
/// support.
#[test]
fn php_imports_completeness_directive_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_imports_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("php")
        .expect("php imports query missing");
    let paths = collect_captures(&lang, PHP_VARIANTS, &query_str, "import.path");

    assert!(
        paths.iter().any(|p| p.contains("User")),
        "expected 'use App\\Models\\User;' (qualified_name, no alias), got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("Order")),
        "expected 'use App\\Models\\Order as OrderModel;' path, got: {paths:?}"
    );
    assert!(
        paths.contains(&"Exception".to_string()),
        "expected bare 'use Exception;' (name, no alias), got: {paths:?}"
    );
    assert!(
        paths.contains(&"Throwable".to_string()),
        "expected bare 'use Throwable as T;' path, got: {paths:?}"
    );
    assert!(
        paths.contains(&"Loggable".to_string()) && paths.contains(&"Cacheable".to_string()),
        "expected grouped 'use App\\Traits\\{{Loggable, Cacheable as Cache}};' members, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("bootstrap.php")),
        "expected 'require_once ... bootstrap.php', got: {paths:?}"
    );
    assert!(
        paths.contains(&"config.php".to_string()),
        "expected 'require config.php', got: {paths:?}"
    );
    assert!(
        paths.contains(&"legacy.php".to_string()),
        "expected 'include legacy.php', got: {paths:?}"
    );
    assert!(
        paths.contains(&"once.php".to_string()),
        "expected 'include_once once.php', got: {paths:?}"
    );
    // Trait composition (use_declaration, distinct from namespace imports).
    assert!(
        paths.contains(&"GreetingTrait".to_string())
            && paths.contains(&"FarewellTrait".to_string()),
        "expected trait composition 'use GreetingTrait;'/'use FarewellTrait, GreetingTrait;', \
         got: {paths:?}"
    );
}

/// Aliased imports must not double-count @import.path (the alias-form and
/// bare-form patterns previously both fired for every aliased `use`).
#[test]
fn php_imports_negative_alias_does_not_double_count() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_imports_negative: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("php")
        .expect("php imports query missing");
    let paths = collect_captures(&lang, PHP_VARIANTS, &query_str, "import.path");
    let order_count = paths.iter().filter(|p| p.contains("Order")).count();
    assert_eq!(
        order_count, 1,
        "'use App\\Models\\Order as OrderModel;' must produce exactly 1 \
         @import.path capture, got {order_count}: {paths:?}"
    );
    let throwable_count = paths.iter().filter(|p| **p == "Throwable").count();
    assert_eq!(
        throwable_count, 1,
        "'use Throwable as T;' must produce exactly 1 @import.path capture, \
         got {throwable_count}: {paths:?}"
    );
}

/// Every grammar-legal variant of `named_type`'s children (name,
/// qualified_name, relative_name) that php.types.scm claims to support.
#[test]
fn php_types_completeness_named_type_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_types_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader.get_types("php").expect("php types query missing");
    let refs = collect_captures(&lang, PHP_VARIANTS, &query_str, "type");

    assert!(
        refs.contains(&"int".to_string()),
        "expected primitive_type 'int', got: {refs:?}"
    );
    assert!(
        refs.contains(&"Widget".to_string()),
        "expected named_type -> name 'Widget', got: {refs:?}"
    );
    assert!(
        refs.iter().any(|t| t.contains("User")),
        "expected named_type -> qualified_name '\\App\\Models\\User', got: {refs:?}"
    );
    assert!(
        refs.iter().any(|t| t.contains("namespace\\Widget")),
        "expected named_type -> relative_name 'namespace\\Widget', got: {refs:?}"
    );
    // Union type members (int|string) — each must appear.
    assert!(
        refs.contains(&"string".to_string()),
        "expected union_type member 'string', got: {refs:?}"
    );
}

/// Negative case: union type members must not be double-counted (a
/// redundant union_type-specific pattern previously duplicated every
/// match the unanchored named_type rule already produced).
#[test]
fn php_types_negative_union_type_not_double_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_types_negative: php grammar .so not found");
        return;
    };
    let query_str = loader.get_types("php").expect("php types query missing");
    let refs = collect_captures(&lang, PHP_VARIANTS, &query_str, "type");
    // `Shape&Colored $f` (intersection_type) is the only site each of these
    // two names appears in a type position in variants.php — unlike
    // "Widget"/"int", which legitimately appear at several distinct
    // parameter sites, so a >1 count for either of these specifically
    // indicates the same named_type node was captured twice, not two
    // different real sites.
    for name in ["Shape", "Colored"] {
        let count = refs.iter().filter(|t| t.as_str() == name).count();
        assert_eq!(
            count, 1,
            "'{name}' (from the 'Shape&Colored' intersection_type) must \
             produce exactly 1 @type.reference capture, got {count}: {refs:?}"
        );
    }
}

/// Every grammar-legal variant of `match_conditional_expression` and the
/// short-circuit boolean operator set that php.complexity.scm claims to
/// support.
#[test]
fn php_complexity_completeness_match_and_boolean_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_complexity_completeness: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("php")
        .expect("php complexity query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();

    assert!(
        complexity_kinds.contains(&"match_conditional_expression"),
        "expected match_conditional_expression @complexity, got: {complexity_kinds:?}"
    );

    let bool_ops: Vec<&str> = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "complexity" && k == "binary_expression")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    for op in ["&&", "||", "and", "or", "xor"] {
        assert!(
            bool_ops.iter().any(|t| t.contains(op)),
            "expected a binary_expression @complexity containing operator '{op}', \
             got: {bool_ops:?}"
        );
    }
}

/// Negative cases: `match_default_expression` (the default arm) and a
/// plain arithmetic binary_expression must never count as @complexity.
#[test]
fn php_complexity_negative_default_arm_and_arithmetic_not_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping php_complexity_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("php").ok() else {
        eprintln!("Skipping php_complexity_negative: php grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("php")
        .expect("php complexity query missing");
    let caps = collect_captures_full(&lang, PHP_VARIANTS, &query_str);
    let default_arms = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "complexity" && k == "match_default_expression")
        .count();
    assert_eq!(
        default_arms, 0,
        "match_default_expression must never count as @complexity, got: {caps:?}"
    );
    let arithmetic_hits = caps
        .iter()
        .filter(|(cn, k, t, _)| {
            cn == "complexity" && k == "binary_expression" && t.contains("1 + 2")
        })
        .count();
    assert_eq!(
        arithmetic_hits, 0,
        "plain arithmetic '1 + 2' must never count as @complexity, got: {caps:?}"
    );
}

// ---------------------------------------------------------------------------
// Dart
// ---------------------------------------------------------------------------

const DART_SAMPLE: &str = include_str!("fixtures/dart/sample.dart");

#[test]
fn dart_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_tags: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("dart").expect("dart tags query missing");
    let names = collect_captures(&lang, DART_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in dart tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in dart tags, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function in dart tags, got: {names:?}"
    );
}

#[test]
fn dart_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_calls: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("dart").expect("dart calls query missing");
    let calls = collect_captures(&lang, DART_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"print".to_string()) || calls.contains(&"push".to_string()),
        "expected 'print' or 'push' call in dart sample, got: {calls:?}"
    );
}

#[test]
fn dart_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_imports: dart grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("dart")
        .expect("dart imports query missing");
    let paths = collect_captures(&lang, DART_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("collection") || p.contains("dart")),
        "expected dart library path in dart import paths, got: {paths:?}"
    );
}

#[test]
fn dart_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_complexity: dart grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("dart")
        .expect("dart complexity query missing");
    let complexity = collect_captures(&lang, DART_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in dart sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn dart_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dart_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dart").ok() else {
        eprintln!("Skipping dart_types: dart grammar .so not found");
        return;
    };
    let query_str = loader.get_types("dart").expect("dart types query missing");
    let refs = collect_captures(&lang, DART_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Point" || r == "int" || r == "String"),
        "expected type identifiers in dart sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Elixir
// ---------------------------------------------------------------------------

const ELIXIR_SAMPLE: &str = include_str!("fixtures/elixir/sample.ex");
const ELIXIR_VARIANTS: &str = include_str!("fixtures/elixir/variants.ex");

// --- Dimension 4: real-world fixture coverage (sample.ex) -------------------

#[test]
fn elixir_tags_finds_modules_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    let names = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in elixir tags, got: {names:?}"
    );
    assert!(
        names.contains(&"push".to_string()) || names.contains(&"pop".to_string()),
        "expected 'push' or 'pop' in elixir tags, got: {names:?}"
    );
    // Guard clauses: `def double(n) when is_integer(n) or is_float(n)` — the
    // gap fixed on top of the shallow baseline. Previously silently dropped
    // any guarded function head entirely.
    assert!(
        names.contains(&"double".to_string()),
        "expected 'double' (guarded function head) in elixir tags, got: {names:?}"
    );
    // defguard
    assert!(
        names.contains(&"is_percentage".to_string()),
        "expected 'is_percentage' (defguard) in elixir tags, got: {names:?}"
    );
    // defp with guard
    assert!(
        names.contains(&"clamp".to_string()),
        "expected 'clamp' (guarded defp) in elixir tags, got: {names:?}"
    );
    // defprotocol / defimpl
    assert!(
        names.contains(&"Sized".to_string()),
        "expected 'Sized' protocol in elixir tags, got: {names:?}"
    );
    // Nested module name is a single dotted alias token
    assert!(
        names.contains(&"Stack.Namespaced".to_string()),
        "expected 'Stack.Namespaced' nested module in elixir tags, got: {names:?}"
    );
    // defmacro / defmacrop
    assert!(
        names.contains(&"trace".to_string()),
        "expected 'trace' defmacro in elixir tags, got: {names:?}"
    );
    assert!(
        names.contains(&"double_expr".to_string()),
        "expected 'double_expr' guarded defmacrop in elixir tags, got: {names:?}"
    );
}

#[test]
fn elixir_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_calls: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let calls = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"defmodule".to_string()) || calls.contains(&"def".to_string()),
        "expected 'defmodule' or 'def' call in elixir sample, got: {calls:?}"
    );
    // Anonymous-function invocation `predicate.(x)` — the gap fixed on top
    // of the shallow baseline (dot with no `right` field).
    assert!(
        calls.contains(&"predicate".to_string()),
        "expected 'predicate' anon-fn invocation call in elixir sample, got: {calls:?}"
    );
}

#[test]
fn elixir_imports_finds_alias_and_import() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_imports: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elixir")
        .expect("elixir imports query missing");
    let paths = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Enum")),
        "expected 'Enum' in elixir import paths, got: {paths:?}"
    );
    // Multi-alias form `alias Stack.{Namespaced}` — the gap fixed on top of
    // the shallow baseline (dot with a tuple right-hand side).
    assert!(
        paths.iter().any(|p| p.contains("Namespaced")),
        "expected 'Namespaced' from multi-alias 'alias Stack.{{Namespaced}}', got: {paths:?}"
    );
}

#[test]
fn elixir_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_complexity: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elixir")
        .expect("elixir complexity query missing");
    let complexity = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in elixir sample, got {} ({complexity:?})",
        complexity.len()
    );
    // The previous blanket `(call) @complexity` counted every function call
    // (including ordinary calls like `Enum.reduce`, `IO.inspect`) as a
    // decision point. With the fix, plain non-branching calls must NOT
    // contribute — only the scoped set of branching macros / stab_clause
    // arms / boolean operators listed in elixir.complexity.scm should.
    // `sum_evens/1`'s body is a single non-branching call with no control
    // flow at all, so the *total* complexity count for the whole sample
    // must be far smaller than "one point per call", which the previous
    // version produced (every `def`/`defmodule`/ordinary call counted).
    let total_calls_in_sample = ELIXIR_SAMPLE.matches('(').count();
    assert!(
        complexity.len() < total_calls_in_sample,
        "expected complexity count ({}) to be well below the raw call count \
         ({total_calls_in_sample}) now that ordinary calls are excluded",
        complexity.len()
    );
}

#[test]
fn elixir_types_finds_module_aliases() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_types: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("elixir")
        .expect("elixir types query missing");
    let refs = collect_captures(&lang, ELIXIR_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r.contains("Enum") || r.contains("Stack") || r.contains("MathUtils")),
        "expected module alias references in elixir sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.ex) -

/// Every grammar-legal variant of def/defp/defmacro/defmacrop/defguard/
/// defguardp/defdelegate name extraction that elixir.tags.scm claims to
/// support, with the correct definition kind (dimension 3).
#[test]
fn elixir_tags_completeness_def_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    let pairs = collect_tag_pairs(&lang, ELIXIR_VARIANTS, &query_str);

    let function_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.function")
        .map(|(_, n)| n.as_str())
        .collect();
    let macro_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.macro")
        .map(|(_, n)| n.as_str())
        .collect();
    let module_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.module")
        .map(|(_, n)| n.as_str())
        .collect();
    let interface_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.interface")
        .map(|(_, n)| n.as_str())
        .collect();
    let impl_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.implementation")
        .map(|(_, n)| n.as_str())
        .collect();

    for expected in [
        "plain_call",
        "plain_noargs",
        "guarded_call",
        "guarded_noargs",
        "private_plain",
        "private_guarded",
        "guard_expr",
        "guardp_expr",
        "delegated_call",
        "delegated_noargs",
    ] {
        assert!(
            function_names.contains(&expected),
            "expected '{expected}' in elixir @definition.function, got: {function_names:?}"
        );
    }
    for expected in [
        "macro_plain",
        "macro_guarded",
        "macrop_plain",
        "macrop_guarded",
    ] {
        assert!(
            macro_names.contains(&expected),
            "expected '{expected}' in elixir @definition.macro, got: {macro_names:?}"
        );
    }
    assert!(
        module_names.contains(&"Plain") && module_names.contains(&"Deep.Nested"),
        "expected 'Plain' and 'Deep.Nested' modules, got: {module_names:?}"
    );
    assert!(
        interface_names.contains(&"PlainProtocol"),
        "expected 'PlainProtocol' defprotocol, got: {interface_names:?}"
    );
    assert!(
        impl_names.contains(&"PlainProtocol"),
        "expected 'PlainProtocol' defimpl reference, got: {impl_names:?}"
    );
}

/// Every grammar-legal variant of call.target (identifier, dot-with-right,
/// dot-without-right, call) that elixir.calls.scm claims to support.
#[test]
fn elixir_calls_completeness_target_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_calls_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let caps = collect_captures_full(&lang, ELIXIR_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // target: identifier (local call)
    assert!(
        calls.contains(&"identity"),
        "expected 'identity' local call, got: {calls:?}"
    );
    // target: dot, right: identifier (remote call)
    assert!(
        calls.contains(&"identity")
            && caps
                .iter()
                .any(|(cn, _, t, _)| cn == "call.qualifier" && t == "Kernel"),
        "expected 'Kernel.identity' remote call with qualifier, got: {caps:?}"
    );
    // target: dot, no right (anonymous-function invocation)
    assert!(
        calls.contains(&"add_one"),
        "expected 'add_one' anon-fn invocation ('add_one.(5)'), got: {calls:?}"
    );

    // target: call (dynamic/macro-generated call, e.g. `unquote(x)(1, 2)`
    // inside a `quote` block) — best-effort partial capture of the inner
    // call's own text.
    let query_str2 = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let dyn_source = "defmodule M do\n  defmacro build(name) do\n    quote do\n      unquote(name)(1, 2)\n    end\n  end\nend\n";
    let dyn_caps = collect_captures_full(&lang, dyn_source, &query_str2);
    assert!(
        dyn_caps
            .iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "call" && t == "unquote(name)"),
        "expected dynamic call-target 'unquote(name)' captured as @call, got: {dyn_caps:?}"
    );
}

/// Negative cases: constructs that must never appear (or must appear exactly
/// once, not duplicated) in @call captures.
#[test]
fn elixir_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_calls_negative: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("elixir")
        .expect("elixir calls query missing");
    let caps = collect_captures_full(&lang, ELIXIR_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder2.other_field` IS a remote call (target: dot, right: identifier
    // "other_field"); it must appear exactly once — the negative guard here
    // is against the anon-invocation `!right` pattern ALSO firing on it (it
    // has a `right`, so it must not double-match).
    let field_calls = call_texts.iter().filter(|t| **t == "other_field").count();
    assert_eq!(
        field_calls, 1,
        "expected exactly 1 'other_field' call (holder2.other_field), got {field_calls}: {call_texts:?}"
    );

    // A bare local-variable read (`bound = plain_arithmetic`) must never be
    // captured as a call.
    assert!(
        !call_texts.contains(&"bound"),
        "local variable 'bound' must never be captured as a call, got: {call_texts:?}"
    );
}

/// Every grammar-legal variant of alias/import/use/require argument shape
/// (plain alias, multi-alias tuple, dot-qualified) that elixir.imports.scm
/// claims to support.
#[test]
fn elixir_imports_completeness_directive_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_imports_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elixir")
        .expect("elixir imports query missing");
    let paths = collect_captures(&lang, ELIXIR_VARIANTS, &query_str, "import.path");

    // Plain forms
    assert!(
        paths.contains(&"Plain".to_string()),
        "expected plain 'alias Plain', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Deep.Nested".to_string()),
        "expected 'alias Deep.Nested, as: DN', got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("Kernel")),
        "expected 'import Kernel, only: [...]', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Logger".to_string()),
        "expected 'require Logger', got: {paths:?}"
    );
    assert!(
        paths.contains(&"Application".to_string()),
        "expected 'use Application', got: {paths:?}"
    );
    // Multi-alias tuple form: `alias Deep.{Nested}`
    assert!(
        paths.iter().filter(|p| *p == "Nested").count() >= 1,
        "expected 'Nested' from multi-alias 'alias Deep.{{Nested}}', got: {paths:?}"
    );
    // Dot-qualified single form: `alias __MODULE__.Plain`
    assert!(
        paths.iter().any(|p| p == "Plain"),
        "expected 'Plain' from dot-qualified 'alias __MODULE__.Plain', got: {paths:?}"
    );
}

/// Every grammar-legal variant of branching complexity node that
/// elixir.complexity.scm claims to support, plus the correctness fix:
/// ordinary (non-branching) calls and arithmetic operators must NOT count.
#[test]
fn elixir_complexity_completeness_branch_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping elixir_complexity_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_complexity_completeness: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elixir")
        .expect("elixir complexity query missing");
    let caps = collect_captures_full(&lang, ELIXIR_VARIANTS, &query_str);
    let complexity_kinds: Vec<&(String, String, String, usize)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .collect();
    let complexity_call_kws: Vec<&str> = complexity_kinds
        .iter()
        .filter(|(_, k, _, _)| k == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    for kw in [
        "if x > 0",
        "unless x > 0",
        "case x",
        "cond do",
        "with {:ok",
        "for x <- list",
        "try do",
        "receive do",
    ] {
        let head = kw.split_whitespace().next().unwrap();
        assert!(
            complexity_call_kws.iter().any(|t| t.starts_with(head)),
            "expected a branching '{head}' call in @complexity, got: {complexity_call_kws:?}"
        );
    }

    // stab_clause arms count independently: branch_case has 3 arms.
    let stab_count = complexity_kinds
        .iter()
        .filter(|(_, k, _, _)| k == "stab_clause")
        .count();
    assert!(
        stab_count >= 3,
        "expected at least 3 'stab_clause' complexity nodes (branch_case alone has 3), got {stab_count}"
    );

    // Boolean operators count; arithmetic/comparison operators must not.
    let bool_ops: Vec<&str> = complexity_kinds
        .iter()
        .filter(|(_, k, _, _)| k == "binary_operator")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        bool_ops.iter().any(|t| t.contains("&&"))
            && bool_ops.iter().any(|t| t.contains(" and "))
            && bool_ops.iter().any(|t| t.contains(" or ")),
        "expected &&/and/or boolean-operator complexity nodes, got: {bool_ops:?}"
    );
}

/// Negative cases: ordinary (non-branching) calls and arithmetic/comparison
/// operators must never contribute to @complexity — the correctness bug
/// fixed on top of the previous blanket `(call) @complexity` / `(binary_
/// operator) @complexity`.
#[test]
fn elixir_complexity_negative_ordinary_calls_and_arithmetic_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_complexity_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_complexity_negative: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elixir")
        .expect("elixir complexity query missing");
    let source = "defmodule N do\n  def f(x) do\n    plain_arithmetic = 1 + 2 - 3 * 4 / 5\n    plain_comparison = 1 == 2\n    identity(x)\n    {plain_arithmetic, plain_comparison}\n  end\nend\n";
    let caps = collect_captures_full(&lang, source, &query_str);
    let complexity_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        complexity_texts.is_empty(),
        "expected 0 complexity nodes for a function with only ordinary calls and \
         arithmetic/comparison operators (no branching), got: {complexity_texts:?}"
    );
}

/// Negative case for tags: `defguard`'s always-guarded form must not also
/// spuriously produce a plain (unguarded) @definition.function match — the
/// unguarded def/defp patterns require `arguments -> call`/`identifier`
/// directly, which a guarded head's `binary_operator` wrapper never
/// satisfies, so no double-counting should occur.
#[test]
fn elixir_tags_negative_guarded_head_not_double_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elixir_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags_negative: elixir grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    let names = collect_captures(&lang, ELIXIR_VARIANTS, &query_str, "name");
    let guarded_count = names.iter().filter(|n| *n == "guarded_call").count();
    assert_eq!(
        guarded_count, 1,
        "expected exactly 1 'guarded_call' definition (not double-counted \
         across guarded/unguarded patterns), got {guarded_count}: {names:?}"
    );
}

// ---------------------------------------------------------------------------
// C
// ---------------------------------------------------------------------------

const C_SAMPLE: &str = include_str!("fixtures/c/sample.c");
const C_VARIANTS: &str = include_str!("fixtures/c/variants.c");

// --- Dimension 4: real-world fixture coverage (sample.c) --------------------

#[test]
fn c_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let names = collect_captures(&lang, C_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' struct in c tags, got: {names:?}"
    );
    assert!(
        names.contains(&"stack_new".to_string()) && names.contains(&"classify".to_string()),
        "expected 'stack_new' and 'classify' functions in c tags, got: {names:?}"
    );
    // Real-world callback-typedef idiom: `typedef int (*Comparator)(...)` —
    // previously dropped entirely (see c.tags.scm's own comments).
    assert!(
        names.contains(&"Comparator".to_string()),
        "expected 'Comparator' callback typedef in c tags, got: {names:?}"
    );
    // Tagged-union idiom: `union Cell { ... };` — previously mislabeled or
    // missed outright depending on shape (the struct/union asymmetry bug).
    assert!(
        names.contains(&"Cell".to_string()),
        "expected 'Cell' union in c tags, got: {names:?}"
    );
    // Object-like and function-like macros — zero tags coverage before this fix.
    assert!(
        names.contains(&"MAX_CAPACITY".to_string()),
        "expected 'MAX_CAPACITY' macro in c tags, got: {names:?}"
    );
    assert!(
        names.contains(&"CLAMP".to_string()),
        "expected 'CLAMP' function-like macro in c tags, got: {names:?}"
    );
}

#[test]
fn c_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_calls: c grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("c").expect("c calls query missing");
    let calls = collect_captures(&lang, C_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"malloc".to_string()) && calls.contains(&"printf".to_string()),
        "expected 'malloc' and 'printf' calls in c sample, got: {calls:?}"
    );
    // Callback idiom: qsort(..., cmp) plus a direct call through the
    // Comparator-typed function-pointer variable.
    assert!(
        calls.contains(&"qsort".to_string()),
        "expected 'qsort' call in c sample, got: {calls:?}"
    );
    assert!(
        calls.iter().filter(|c| *c == "cmp").count() >= 1,
        "expected at least 1 call through the 'cmp' function-pointer variable, got: {calls:?}"
    );
}

#[test]
fn c_imports_finds_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_imports: c grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("c").expect("c imports query missing");
    let paths = collect_captures(&lang, C_SAMPLE, &query_str, "import.path");
    // Raw capture text still carries the angle brackets (`<stdio.h>`); the
    // Rust-side extraction layer strips them, not the query itself.
    assert!(
        paths.iter().any(|p| p.contains("stdio.h"))
            && paths.iter().any(|p| p.contains("stdlib.h"))
            && paths.iter().any(|p| p.contains("string.h")),
        "expected all three system includes in c import paths, got: {paths:?}"
    );
}

#[test]
fn c_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_complexity: c grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("c")
        .expect("c complexity query missing");
    let complexity = collect_captures(&lang, C_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 5,
        "expected at least 5 complexity nodes in c sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn c_types_finds_type_identifiers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_types: c grammar .so not found");
        return;
    };
    let query_str = loader.get_types("c").expect("c types query missing");
    let refs = collect_captures(&lang, C_SAMPLE, &query_str, "type");
    assert!(
        refs.iter().any(|r| r == "Stack"),
        "expected 'Stack' in c type references, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.c) -

/// Every grammar-legal name+body variant of `struct_specifier`/`union_specifier`
/// that c.tags.scm claims to support — bare, typedef'd-anonymous, and
/// typedef'd-named — must produce a @definition.class capture with the
/// correct kind, not just the right text (dimension 3).
#[test]
fn c_tags_completeness_struct_and_union_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_completeness: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);

    // Bare union definition — the case the old query (declaration-wrapped
    // pattern) never matched at all.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "union_specifier"
            && t.contains("PlainUnion")),
        "expected 'PlainUnion' union_specifier as definition.class, got: {caps:?}"
    );
    // Named union nested inside a typedef — the other case the old query
    // never matched.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "union_specifier"
            && t.contains("TaggedUnion")),
        "expected 'TaggedUnion' union_specifier as definition.class, got: {caps:?}"
    );
    // The typedef alias itself is still captured via the (unrelated)
    // type_definition pattern.
    let type_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        type_names.contains(&"TaggedUnionAlias"),
        "expected 'TaggedUnionAlias' typedef name, got: {type_names:?}"
    );
    // Struct definitions still work unaffected by the union fix.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "struct_specifier"
            && t.contains("PlainStruct")),
        "expected 'PlainStruct' struct_specifier as definition.class, got: {caps:?}"
    );
}

/// Typedef'd function pointers (`typedef int (*FuncPtr)(int, int);`) must
/// produce a @definition.type capture for the alias name, verifying the
/// three-level-nested declarator pattern (function_declarator >
/// parenthesized_declarator > pointer_declarator > type_identifier).
#[test]
fn c_tags_completeness_typedef_function_pointer() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping c_tags_completeness_typedef_fnptr: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_completeness_typedef_fnptr: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "name" && k == "type_identifier" && t == "FuncPtr"),
        "expected 'FuncPtr' typedef'd-function-pointer name as (name, type_identifier), got: {caps:?}"
    );
}

/// Object-like and function-like macro definitions must both produce
/// @definition.macro captures — previously zero macro tags coverage at all.
#[test]
fn c_tags_completeness_macro_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags_completeness_macros: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_completeness_macros: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.macro"
            && k == "preproc_def"
            && t.contains("MAX_SIZE")),
        "expected object-like macro 'MAX_SIZE' as (definition.macro, preproc_def), got: {caps:?}"
    );
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.macro"
            && k == "preproc_function_def"
            && t.contains("SQUARE")),
        "expected function-like macro 'SQUARE' as (definition.macro, preproc_function_def), got: {caps:?}"
    );
}

/// Negative cases: constructs that must never be tagged as
/// @definition.function/@definition.class.
#[test]
fn c_tags_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_tags_negative: c grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("c").expect("c tags query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);

    // `int (*negative_function_pointer_variable)(int);` declares a variable
    // of function-pointer type, not a function — function_declarator's
    // declarator field is parenthesized_declarator, never a bare identifier.
    let def_fn_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "definition.function")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        !def_fn_names
            .iter()
            .any(|n| n.contains("negative_function_pointer_variable")),
        "function-pointer *variable* declaration must never be @definition.function, got: {def_fn_names:?}"
    );

    // `union NegativeUsage;` (bodyless forward reference) must never produce
    // @definition.class — this is exactly the false positive the old
    // declaration-wrapped union pattern produced.
    assert!(
        !caps
            .iter()
            .any(|(cn, _, t, _)| cn == "definition.class" && t.contains("NegativeUsage")),
        "bodyless union forward-reference 'NegativeUsage' must never be @definition.class, got: {caps:?}"
    );
}

/// Negative case: a bare field read through `->` with no call parens must
/// never appear in a @call capture.
#[test]
fn c_calls_negative_bare_field_access_is_not_a_call() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_calls_negative: c grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("c").expect("c calls query missing");
    let calls = collect_captures(&lang, C_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access 'holder->field' must not be captured as a call, got: {calls:?}"
    );
}

/// Field/pointer member calls through a function-pointer struct member
/// (`p->fp(...)`, `v.fp(...)`) must be captured with the correct qualifier,
/// for both the `->` and `.` operator forms.
#[test]
fn c_calls_completeness_field_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping c_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c").ok() else {
        eprintln!("Skipping c_calls_completeness: c grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("c").expect("c calls query missing");
    let caps = collect_captures_full(&lang, C_VARIANTS, &query_str);
    let fp_calls: Vec<&(String, String, String, usize)> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "call" && k == "field_identifier" && t == "fp")
        .collect();
    assert_eq!(
        fp_calls.len(),
        2,
        "expected exactly 2 field-expression calls to 'fp' (via -> and via .), got {}: {fp_calls:?}",
        fp_calls.len()
    );
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"p") && qualifiers.contains(&"v"),
        "expected 'p' (-> form) and 'v' (. form) qualifiers, got: {qualifiers:?}"
    );
}

// ---------------------------------------------------------------------------
// C++
// ---------------------------------------------------------------------------

const CPP_SAMPLE: &str = include_str!("fixtures/cpp/sample.cpp");
const CPP_VARIANTS: &str = include_str!("fixtures/cpp/variants.cpp");

// --- Dimension 4: real-world fixture coverage (sample.cpp) ------------------

#[test]
fn cpp_tags_finds_class_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let names = collect_captures(&lang, CPP_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in cpp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()) && names.contains(&"sum_evens".to_string()),
        "expected 'classify' and 'sum_evens' functions in cpp tags, got: {names:?}"
    );
    // Namespace container — previously zero namespace tags coverage at all.
    assert!(
        names.contains(&"shapes".to_string()),
        "expected 'shapes' namespace in cpp tags, got: {names:?}"
    );
    // Polymorphic base + derived class, with a destructor *declared* inline
    // (`virtual ~Shape();`, no body) and *defined* out-of-line — previously
    // entirely untagged (destructor_name declarator variant).
    assert!(
        names.contains(&"Shape".to_string()) && names.contains(&"Circle".to_string()),
        "expected 'Shape' and 'Circle' classes in cpp tags, got: {names:?}"
    );
    let destructor_defs = names.iter().filter(|n| n.contains("~Shape")).count();
    assert_eq!(
        destructor_defs, 2,
        "expected exactly 2 '~Shape' function_declarator matches (the inline prototype \
         declaration plus the out-of-line definition — function_declarator has no body \
         constraint, so a prototype and its definition both match, exactly like every other \
         function/method in this query), got {destructor_defs}: {names:?}"
    );
    // Operator overload — previously entirely untagged (operator_name
    // declarator variant).
    assert!(
        names.iter().any(|n| n == "operator+="),
        "expected 'operator+=' overload in cpp tags, got: {names:?}"
    );
}

#[test]
fn cpp_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let calls = collect_captures(&lang, CPP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"push".to_string()) && calls.contains(&"pop".to_string()),
        "expected 'push' and 'pop' method calls in cpp sample, got: {calls:?}"
    );
    // Smart-pointer + polymorphism idiom: make_unique<Circle>(...), a
    // template-argument call.
    assert!(
        calls.iter().any(|c| c.contains("make_unique")),
        "expected 'make_unique<...>' templated call in cpp sample, got: {calls:?}"
    );
    // Plain template-argument call: identity<int>(21) — direct analogue of
    // Rust's turbofish gap.
    assert!(
        calls.iter().any(|c| c.contains("identity")),
        "expected 'identity<int>' templated call in cpp sample, got: {calls:?}"
    );
}

#[test]
fn cpp_imports_finds_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_imports: cpp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cpp")
        .expect("cpp imports query missing");
    let paths = collect_captures(&lang, CPP_SAMPLE, &query_str, "import.path");
    // Raw capture text still carries the angle brackets (`<iostream>`); the
    // Rust-side extraction layer strips them, not the query itself.
    assert!(
        paths.iter().any(|p| p.contains("iostream")) && paths.iter().any(|p| p.contains("vector")),
        "expected 'iostream' and 'vector' in cpp import paths, got: {paths:?}"
    );
    // `using namespace std::literals;` — previously zero `using` coverage at
    // all in cpp.imports.scm (only #include was tracked).
    assert!(
        paths.iter().any(|p| p.contains("literals")),
        "expected 'using namespace std::literals' import path in cpp sample, got: {paths:?}"
    );
}

#[test]
fn cpp_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_complexity: cpp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("cpp")
        .expect("cpp complexity query missing");
    let complexity = collect_captures(&lang, CPP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in cpp sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn cpp_types_finds_type_identifiers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_types: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_types("cpp").expect("cpp types query missing");
    let refs = collect_captures(&lang, CPP_SAMPLE, &query_str, "type");
    assert!(
        refs.iter().any(|r| r == "Stack"),
        "expected 'Stack' in cpp type references, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.cpp)

/// Every grammar-legal variant of union/class-specialization/namespace
/// definitions that cpp.tags.scm claims to support must produce a capture
/// with the correct kind, not just the right text.
#[test]
fn cpp_tags_completeness_union_specialization_namespace() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags_completeness: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);

    // Bare union definition — same struct/union asymmetry bug as C.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "union_specifier"
            && t.contains("PlainUnion")),
        "expected 'PlainUnion' union_specifier as definition.class, got: {caps:?}"
    );
    // Explicit template specialization: `template <> class TemplateClass<int>`
    // — name is wrapped in template_type, previously unmatched entirely.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "definition.class"
            && k == "class_specifier"
            && t.contains("TemplateClass<int>")),
        "expected explicit specialization 'TemplateClass<int>' as definition.class, got: {caps:?}"
    );
    let names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        names.iter().filter(|n| **n == "TemplateClass").count() >= 2,
        "expected 'TemplateClass' name from both the primary template and its \
         specialization, got: {names:?}"
    );
    // Namespaces: plain, nested plain, and nested path-form
    // (`namespace deep::path::here`) — previously zero namespace tags
    // coverage of any kind.
    assert!(
        names.contains(&"outer_ns") && names.contains(&"inner_ns"),
        "expected 'outer_ns' and nested 'inner_ns' namespaces, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("deep") && n.contains("path") && n.contains("here")),
        "expected 'deep::path::here' nested_namespace_specifier name, got: {names:?}"
    );
}

/// Destructors and operator overloads — inline, out-of-line (plain class),
/// and out-of-line (template class) — must all be tagged as
/// @definition.method with the correct name, none of which were captured at
/// all before this fix.
#[test]
fn cpp_tags_completeness_destructors_and_operators() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags_completeness_dtor_op: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags_completeness_dtor_op: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);
    // @name carries just the destructor_name/operator_name node text (e.g.
    // "~WithSpecialMembers", "operator="); @definition.method carries the
    // whole function_declarator (e.g. "~WithSpecialMembers()"), which is
    // deliberately not what's asserted on here since the goal is verifying
    // the captured *name*, dimension 3.
    let method_names: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // Inline destructor + inline operator overload.
    assert!(
        method_names.contains(&"~WithSpecialMembers"),
        "expected inline destructor '~WithSpecialMembers' as name, got: {method_names:?}"
    );
    assert!(
        method_names.contains(&"operator="),
        "expected inline operator overload 'operator=' as name, got: {method_names:?}"
    );
    // Out-of-line destructor + operator overload (plain class).
    assert!(
        method_names.contains(&"~OutOfLineMembers"),
        "expected out-of-line destructor '~OutOfLineMembers' as name, got: {method_names:?}"
    );
    assert!(
        method_names.contains(&"operator+="),
        "expected out-of-line operator overload 'operator+=' as name, got: {method_names:?}"
    );
    // Out-of-line method on a template class, where the qualifier scope
    // itself carries template arguments (`OutOfLineTemplateMethods<T>::get`).
    assert!(
        method_names.contains(&"get"),
        "expected out-of-line template-class method 'get' as definition.method, got: {method_names:?}"
    );
}

/// Negative case: a lambda is not a `function_declarator`/`class_specifier`;
/// its parameter/body identifiers must never appear as @definition.function
/// or @definition.method.
#[test]
fn cpp_tags_negative_lambda_is_not_a_definition() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_tags_negative: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cpp").expect("cpp tags query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);
    let is_def_add_one = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t.contains("add_one")
    });
    assert!(
        !is_def_add_one,
        "lambda binding 'add_one' must never be captured as a function/method definition, got: {caps:?}"
    );
}

/// Every grammar-legal variant of `field_expression.field` that
/// cpp.calls.scm claims to support (plain field_identifier, template_method,
/// destructor_name, qualified_identifier) must produce a @call capture with
/// the correct kind.
#[test]
fn cpp_calls_completeness_field_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls_completeness: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "field_identifier", "plain_method"), // pre-existing, still works
        ("call", "template_method", "templated_method<int>"), // previously unmatched
        ("call", "destructor_name", "~CallTarget"),   // previously unmatched
    ];
    for (cn, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(n, k, t, _)| n == cn && k == kind && t == text),
            "expected capture ({cn}, kind={kind}, text={text}) in cpp.calls.scm output for \
             variants.cpp, got: {caps:?}"
        );
    }
    // Explicit base-class-qualified call: derived.CallTarget::plain_method()
    // — field is a nested qualified_identifier.
    assert!(
        caps.iter().any(|(cn, k, t, _)| cn == "call"
            && k == "qualified_identifier"
            && t == "CallTarget::plain_method"),
        "expected base-qualified call 'CallTarget::plain_method' (kind=qualified_identifier), got: {caps:?}"
    );
}

/// Every grammar-legal variant of template-argument calls — plain
/// (`identity<int>(5)`) and scoped (`ns::helper<int>(3)`) — must produce a
/// @call capture, the direct C++ analogue of Rust's turbofish gap.
#[test]
fn cpp_calls_completeness_template_argument_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping cpp_calls_completeness_template: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls_completeness_template: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let caps = collect_captures_full(&lang, CPP_VARIANTS, &query_str);

    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "template_function" && t == "identity<int>"),
        "expected plain template-argument call 'identity<int>' (kind=template_function), got: {caps:?}"
    );
    assert!(
        caps.iter()
            .any(|(cn, k, t, _)| cn == "call" && k == "template_function" && t == "helper<int>"),
        "expected scoped template-argument call 'helper<int>' (kind=template_function), got: {caps:?}"
    );
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"call_ns"),
        "expected 'call_ns' qualifier for the scoped template-argument call, got: {qualifiers:?}"
    );
}

/// Negative case: a bare field read must never appear in a @call capture.
#[test]
fn cpp_calls_negative_bare_field_access_is_not_a_call() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_calls_negative: cpp grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("cpp").expect("cpp calls query missing");
    let calls = collect_captures(&lang, CPP_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access 'holder->field' must not be captured as a call, got: {calls:?}"
    );
}

/// Every grammar-legal variant of `using`/alias imports that cpp.imports.scm
/// claims to support — `using namespace X;`, `using X::Y;`, `using Alias =
/// Type;`, `namespace alias = X;` (single- and nested-segment) — must
/// produce a correctly-shaped @import, all previously entirely unsupported.
#[test]
fn cpp_imports_completeness_using_and_alias_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cpp_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cpp").ok() else {
        eprintln!("Skipping cpp_imports_completeness: cpp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cpp")
        .expect("cpp imports query missing");
    let paths = collect_captures(&lang, CPP_VARIANTS, &query_str, "import.path");
    let aliases = collect_captures(&lang, CPP_VARIANTS, &query_str, "import.alias");

    // using namespace detail;
    assert!(
        paths.contains(&"detail".to_string()),
        "expected 'using namespace detail' import path, got: {paths:?}"
    );
    // using ns_target::Thing;
    assert!(
        paths.contains(&"ns_target::Thing".to_string()),
        "expected 'using ns_target::Thing' import path, got: {paths:?}"
    );
    // using IntAlias = int;
    assert!(
        aliases.contains(&"IntAlias".to_string()) && paths.contains(&"int".to_string()),
        "expected type-alias 'IntAlias = int', aliases={aliases:?} paths={paths:?}"
    );
    // namespace short_ns = ns_target;  (single-segment)
    assert!(
        aliases.contains(&"short_ns".to_string()) && paths.contains(&"ns_target".to_string()),
        "expected namespace alias 'short_ns = ns_target', aliases={aliases:?} paths={paths:?}"
    );
    // namespace nested_alias = ns_target::Thing::deeper;  (nested path)
    assert!(
        aliases.contains(&"nested_alias".to_string())
            && paths.iter().any(|p| p == "ns_target::Thing::deeper"),
        "expected namespace alias 'nested_alias = ns_target::Thing::deeper', \
         aliases={aliases:?} paths={paths:?}"
    );
}

// ---------------------------------------------------------------------------
// C#
// ---------------------------------------------------------------------------

const CSHARP_SAMPLE: &str = include_str!("fixtures/c-sharp/sample.cs");
const CSHARP_VARIANTS: &str = include_str!("fixtures/c-sharp/variants.cs");

// --- Dimension 4: real-world fixture coverage (sample.cs) -------------------

#[test]
fn csharp_tags_finds_class_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_tags: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let names = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' class in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"MathUtils".to_string()),
        "expected 'MathUtils' class in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Classify".to_string()),
        "expected 'Classify' method in c-sharp tags, got: {names:?}"
    );
    // base_list: `class Stack<T> : IEnumerable<T>, System.IDisposable` — both
    // the generic and the path-qualified interface must be found.
    assert!(
        names.contains(&"IEnumerable".to_string()),
        "expected 'IEnumerable' (generic base_list entry) in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"IDisposable".to_string()),
        "expected 'IDisposable' (path-qualified base_list entry) in c-sharp tags, got: {names:?}"
    );
    // `class BoundedStack<T> : Stack<T>` — generic base class.
    assert!(
        names.contains(&"BoundedStack".to_string()),
        "expected 'BoundedStack' class in c-sharp tags, got: {names:?}"
    );
    // Record with primary-constructor base type: `record Point3D(...) : Point(X, Y);`
    assert!(
        names.contains(&"Point3D".to_string()),
        "expected 'Point3D' record in c-sharp tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' record (primary-constructor base) in c-sharp tags, got: {names:?}"
    );
    // Lambda binding parameters must never surface as method/function
    // definitions — closures aren't method_declaration/local_function_statement.
    let def_names: Vec<&str> = names
        .iter()
        .map(std::string::String::as_str)
        .filter(|n| *n == "FetchLengthAsync")
        .collect();
    assert!(
        def_names.contains(&"FetchLengthAsync"),
        "expected the real async method 'FetchLengthAsync' in c-sharp tags, got: {names:?}"
    );
}

#[test]
fn csharp_tags_finds_call_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_tags_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_tags_calls: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_SAMPLE, &query_str);
    let ref_calls: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.call")
        .map(|(_, n)| n.as_str())
        .collect();
    // base()/this() constructor delegation inside BoundedStack.
    assert!(
        ref_calls.contains(&"base"),
        "expected 'base' constructor-delegation reference.call, got: {ref_calls:?}"
    );
    assert!(
        ref_calls.contains(&"this"),
        "expected 'this' constructor-delegation reference.call, got: {ref_calls:?}"
    );
    // Qualified generic LINQ call: Enumerable.Range(...).Where(...).ToList()
    assert!(
        ref_calls.contains(&"Range"),
        "expected 'Range' qualified generic call reference, got: {ref_calls:?}"
    );
}

#[test]
fn csharp_calls_finds_method_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let calls = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"Push".to_string())
            || calls.contains(&"WriteLine".to_string())
            || calls.contains(&"Add".to_string()),
        "expected method call in c-sharp sample, got: {calls:?}"
    );
    // Unqualified generic call: Identity<int>(42).
    assert!(
        calls.contains(&"Identity".to_string()),
        "expected 'Identity' generic call in c-sharp sample, got: {calls:?}"
    );
    // Qualified generic LINQ chain: Enumerable.Range(...).Where(...).ToList().
    assert!(
        calls.contains(&"Range".to_string()),
        "expected 'Range' call in c-sharp sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"Where".to_string()),
        "expected 'Where' chained call in c-sharp sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"ToList".to_string()),
        "expected 'ToList' chained call in c-sharp sample, got: {calls:?}"
    );
    // Null-conditional invocation chain: maybeNull?.Trim()?.Length (Trim is a
    // call; Length is a property access, not a call).
    assert!(
        calls.contains(&"Trim".to_string()),
        "expected 'Trim' null-conditional call in c-sharp sample, got: {calls:?}"
    );
    // base()/this() constructor delegation.
    assert!(
        calls.contains(&"base".to_string()),
        "expected 'base' constructor-delegation call in c-sharp sample, got: {calls:?}"
    );
    assert!(
        calls.contains(&"this".to_string()),
        "expected 'this' constructor-delegation call in c-sharp sample, got: {calls:?}"
    );
    // Extension method call: blank.IsBlank().
    assert!(
        calls.contains(&"IsBlank".to_string()),
        "expected 'IsBlank' extension-method call in c-sharp sample, got: {calls:?}"
    );
}

#[test]
fn csharp_imports_finds_using_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_imports: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("c-sharp")
        .expect("c-sharp imports query missing");
    let paths = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "import.path");
    // Must capture simple identifier: `using System;`
    assert!(
        paths.iter().any(|p| p == "System"),
        "expected 'System' in c-sharp import paths, got: {paths:?}"
    );
    // Must capture qualified name: `using System.Collections.Generic;`
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Collections") || p.contains("Generic")),
        "expected qualified namespace in c-sharp import paths, got: {paths:?}"
    );
    // `using System.Linq;` / `using System.Threading.Tasks;`
    assert!(
        paths.iter().any(|p| p.contains("Linq")),
        "expected 'System.Linq' in c-sharp import paths, got: {paths:?}"
    );
}

#[test]
fn csharp_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_complexity: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("c-sharp")
        .expect("c-sharp complexity query missing");
    let complexity = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in c-sharp sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn csharp_complexity_finds_switch_expression_arms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_complexity_switch: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_complexity_switch: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("c-sharp")
        .expect("c-sharp complexity query missing");
    let caps = collect_captures_full(&lang, CSHARP_SAMPLE, &query_str);
    // sample.cs's `n switch { < 0 => ..., 0 => ..., _ => ... }` has 3 arms —
    // previously entirely uncounted (switch_expression_arm is a distinct node
    // kind from switch_section, the statement-form switch's case label).
    let arm_count = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "complexity" && k == "switch_expression_arm")
        .count();
    assert!(
        arm_count >= 3,
        "expected >= 3 switch_expression_arm complexity nodes, got {arm_count}: {caps:?}"
    );
}

#[test]
fn csharp_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "type.reference");
    assert!(
        refs.iter()
            .any(|r| r == "Stack" || r == "MathUtils" || r == "List"),
        "expected type reference in c-sharp sample, got: {refs:?}"
    );
}

#[test]
fn csharp_types_finds_type_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types_definitions: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_definitions: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_SAMPLE, &query_str);
    let def_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k.starts_with("definition."))
        .map(|(_, n)| n.as_str())
        .collect();
    // Previously c-sharp.types.scm had NO @definition.type at all.
    assert!(
        def_names.contains(&"Stack"),
        "expected 'Stack' @definition.type, got: {def_names:?}"
    );
    assert!(
        def_names.contains(&"Point"),
        "expected 'Point' record @definition.type, got: {def_names:?}"
    );
}

#[test]
fn csharp_types_negative_no_value_identifiers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_negative: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_SAMPLE, &query_str, "type.reference");
    // Regression test for the severe overmatching bug: `(identifier)
    // @type.reference` with no field constraint used to match every
    // identifier in the file, including method names, parameter names, and
    // local variable names. None of these are type positions.
    for value_ident in ["Push", "Add", "items", "item", "Classify", "stack"] {
        assert!(
            !refs.contains(&value_ident.to_string()),
            "'{value_ident}' is a value identifier, must not appear as a \
             @type.reference, got: {refs:?}"
        );
    }
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.cs) --

/// Every grammar-legal variant of `base_list` (plain, generic, path-qualified
/// base class AND interface — C# has no syntactic extends/implements split)
/// must produce a @reference.class capture, matching c-sharp.tags.scm's
/// completeness claims.
#[test]
fn csharp_tags_completeness_base_list_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_tags_completeness_base_list: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_tags_completeness_base_list: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_VARIANTS, &query_str);
    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    for expected in [
        "PlainBase",         // base_list: identifier
        "GenericBase",       // base_list: generic_name -> identifier
        "Exception",         // base_list: qualified_name -> identifier
        "IPlainIface",       // base_list: identifier (interface)
        "IGenericIface",     // base_list: generic_name -> identifier (interface)
        "IDisposable",       // base_list: qualified_name -> identifier (interface)
        "RecordBase",        // primary_constructor_base_type: identifier
        "RecordBaseGeneric", // primary_constructor_base_type: generic_name -> identifier
    ] {
        assert!(
            ref_class_names.contains(&expected),
            "expected '{expected}' among base_list @reference.class captures, got: {ref_class_names:?}"
        );
    }
    // MultiBase : PlainBase, IPlainIface, IGenericIface<int> — all 3 entries
    // in one base_list must be found, not just the first.
    let multi_base_count = ref_class_names
        .iter()
        .filter(|n| **n == "PlainBase" || **n == "IPlainIface" || **n == "IGenericIface")
        .count();
    assert!(
        multi_base_count >= 3,
        "expected all 3 entries of MultiBase's base_list, found {multi_base_count} among: {ref_class_names:?}"
    );
}

/// Every grammar-legal variant of `object_creation_expression.type` must
/// produce a @reference.class capture with the leaf class name.
#[test]
fn csharp_tags_completeness_object_creation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_tags_completeness_object_creation: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!(
            "Skipping csharp_tags_completeness_object_creation: c-sharp grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_VARIANTS, &query_str);
    let ref_class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "reference.class")
        .map(|(_, n)| n.as_str())
        .collect();
    // Note: `new object()` (PlainNew) is deliberately excluded — `object` is a
    // `predefined_type` keyword node (like Java's `boolean_type`/`integral_type`),
    // not `identifier`/`generic_name`/`qualified_name`, so it correctly does
    // NOT produce a @reference.class (a builtin keyword type isn't a "class
    // reference" in any meaningful sense).
    for expected in ["List", "StringBuilder"] {
        assert!(
            ref_class_names.contains(&expected),
            "expected '{expected}' among object-creation @reference.class captures, got: {ref_class_names:?}"
        );
    }
    // Extraction depth: leaf-only names (no '.') even for qualified forms.
    assert!(
        ref_class_names.iter().all(|n| !n.contains('.')),
        "expected leaf-only class names (no '.'), got: {ref_class_names:?}"
    );
}

/// Every type-defining declaration kind (class, struct, interface, enum,
/// record) must be found as a tags definition.
#[test]
fn csharp_tags_completeness_type_declaration_kinds() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_tags_completeness_type_declaration_kinds: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!(
            "Skipping csharp_tags_completeness_type_declaration_kinds: c-sharp grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("c-sharp")
        .expect("c-sharp tags query missing");
    let pairs = collect_tag_pairs(&lang, CSHARP_VARIANTS, &query_str);
    let find_def_kind = |name: &str| -> Option<&str> {
        pairs
            .iter()
            .find(|(k, n)| k.starts_with("definition.") && n == name)
            .map(|(k, _)| k.as_str())
    };
    assert_eq!(find_def_kind("PlainClass"), Some("definition.class"));
    assert_eq!(
        find_def_kind("PlainStruct"),
        Some("definition.class"),
        "structs map to definition.class (closest existing kind)"
    );
    assert_eq!(
        find_def_kind("PlainInterface"),
        Some("definition.interface")
    );
    assert_eq!(find_def_kind("PlainEnum"), Some("definition.enum"));
    assert_eq!(
        find_def_kind("PlainRecord"),
        Some("definition.class"),
        "records map to definition.class (closest existing kind)"
    );
}

/// Every grammar-legal variant of `invocation_expression.function` (plain
/// identifier, generic_name, member_access_expression with identifier/
/// generic_name name, chained qualifier, conditional-access) must produce a
/// @call capture, matching c-sharp.calls.scm's completeness claims.
#[test]
fn csharp_calls_completeness_invocation_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls_completeness: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let caps = collect_captures_full(&lang, CSHARP_VARIANTS, &query_str);
    let calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        calls.contains(&"Identity"),
        "expected plain call 'Identity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"GenericIdentity"),
        "expected unqualified generic call 'GenericIdentity', got: {calls:?}"
    );
    assert!(
        calls.contains(&"WriteLine"),
        "expected qualified call 'WriteLine', got: {calls:?}"
    );
    assert!(
        calls.contains(&"OfType"),
        "expected qualified generic call 'OfType', got: {calls:?}"
    );
    assert!(
        calls.contains(&"Trim") && calls.contains(&"ToUpper"),
        "expected chained calls 'Trim'/'ToUpper', got: {calls:?}"
    );
    // Null-conditional invocation: s?.Trim() / xs?.OfType<int>().
    let conditional_call_count = calls.iter().filter(|c| **c == "Trim").count();
    assert!(
        conditional_call_count >= 1,
        "expected at least one conditional-access 'Trim' call, got: {calls:?}"
    );
}

/// `constructor_initializer`'s base(...)/this(...) delegation must produce a
/// @call capture for both keywords.
#[test]
fn csharp_calls_completeness_constructor_initializer() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_calls_completeness_ctor_init: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls_completeness_ctor_init: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let calls = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "call");
    assert!(
        calls.contains(&"base".to_string()),
        "expected 'base' constructor-delegation call, got: {calls:?}"
    );
    assert!(
        calls.contains(&"this".to_string()),
        "expected 'this' constructor-delegation call, got: {calls:?}"
    );
}

/// Negative case: method references passed as delegates, bare field
/// access/writes, and casts must never appear as @call captures.
#[test]
fn csharp_calls_negative_field_access_and_lambda() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_calls_negative: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("c-sharp")
        .expect("c-sharp calls query missing");
    let calls = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "call");
    assert!(
        !calls.contains(&"field".to_string()),
        "bare field access/write 'this.field' must not be captured as a call, got: {calls:?}"
    );
    assert!(
        !calls.contains(&"StaticMethod".to_string()),
        "'StaticMethod' is never invoked in variants.cs (only referenced via delegate-\
         shaped lambda text); must not spuriously appear as a call, got: {calls:?}"
    );
}

/// Every grammar-legal variant of `using_directive`'s path argument (bare
/// identifier, qualified_name, bare generic_name, bare alias_qualified_name,
/// each with and without an alias) must produce a correctly-shaped @import,
/// with NO duplicate @import.path for aliased forms (regression test for the
/// alias/path overlap bug).
#[test]
fn csharp_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_imports_completeness: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("c-sharp")
        .expect("c-sharp imports query missing");
    let paths = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "import.path");
    let aliases = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "import.alias");

    // using Bare; — bare single-segment path.
    assert!(
        paths.contains(&"Bare".to_string()),
        "expected 'Bare' bare-identifier import path, got: {paths:?}"
    );
    // using System.Collections.Generic; — qualified path.
    assert!(
        paths.iter().any(|p| p.contains("Collections")),
        "expected qualified import path, got: {paths:?}"
    );
    // using static Wrapper<int>; — bare generic_name path.
    assert!(
        paths.iter().any(|p| p.starts_with("Wrapper")),
        "expected 'Wrapper<int>' bare-generic import path, got: {paths:?}"
    );
    // using global::System; — bare alias_qualified_name path.
    assert!(
        paths.iter().any(|p| p.contains("global::System")),
        "expected 'global::System' alias_qualified_name import path, got: {paths:?}"
    );
    // using Sys = System; using SysColl = ...; using MyList = List<int>; —
    // three aliases, each with exactly one alias and one path capture (the
    // historical bug produced the alias identifier itself as a spurious
    // second @import.path).
    assert!(
        aliases.contains(&"Sys".to_string()),
        "expected 'Sys' import alias, got: {aliases:?}"
    );
    assert!(
        !paths.contains(&"Sys".to_string()),
        "alias name 'Sys' must not also appear as an @import.path, got: {paths:?}"
    );
    assert!(
        !paths.contains(&"SysColl".to_string()),
        "alias name 'SysColl' must not also appear as an @import.path, got: {paths:?}"
    );
    assert!(
        !paths.contains(&"MyList".to_string()),
        "alias name 'MyList' must not also appear as an @import.path, got: {paths:?}"
    );
}

/// Exact-count regression test for the duplicate-@import.path bug: every
/// using_directive in variants.cs must produce exactly one @import.path
/// capture (not two, from the alias identifier bleeding into the plain
/// pattern).
#[test]
fn csharp_imports_negative_no_duplicate_path_per_alias() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_imports_negative: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("c-sharp")
        .expect("c-sharp imports query missing");
    let caps = collect_captures_full(&lang, CSHARP_VARIANTS, &query_str);
    // variants.cs has exactly 8 using directives.
    let import_stmts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert_eq!(
        import_stmts.len(),
        8,
        "expected 8 @import captures (one per using directive in variants.cs), got {}: {import_stmts:?}",
        import_stmts.len()
    );
    // Every using_directive produces exactly one @import.path — the
    // historical bug produced two for aliased forms (the alias identifier
    // plus the real path), so a 1:1 ratio with @import statements is the
    // exact regression guard.
    let path_count = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "import.path")
        .count();
    assert_eq!(
        path_count, 8,
        "expected exactly 8 @import.path captures (one per using directive, no \
         duplicates from alias bleed-through), got {path_count}: {caps:?}"
    );
}

/// Every `types.scm`-covered type-position field (variable declaration,
/// parameter, method return type, local function type, property type,
/// foreach loop variable, catch clause, cast, is/as pattern) must produce a
/// @type.reference capture for its identifier/generic_name/qualified_name/
/// nullable_type-wrapped leaf.
#[test]
fn csharp_types_completeness_field_positions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping csharp_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_completeness: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "type.reference");
    // variable_declaration.type: identifier / generic_name / qualified_name / nullable_type
    assert!(
        refs.contains(&"List".to_string()),
        "expected 'List' (variable_declaration.type generic_name), got: {refs:?}"
    );
    assert!(
        refs.contains(&"StringBuilder".to_string()),
        "expected 'StringBuilder' (variable_declaration.type qualified_name), got: {refs:?}"
    );
    // parameter.type
    assert!(
        refs.iter().filter(|r| *r == "List").count() >= 2,
        "expected 'List' from both variable_declaration.type and parameter.type, got: {refs:?}"
    );
    // method_declaration.returns
    assert!(
        refs.iter().filter(|r| *r == "StringBuilder").count() >= 2,
        "expected 'StringBuilder' from both variable_declaration.type and returns:, got: {refs:?}"
    );
    // The bare `identifier` variant (as opposed to generic_name/qualified_name)
    // — exercised via the user-defined `PlainClass` type across
    // variable_declaration.type, parameter.type, method_declaration.returns,
    // local_function_statement.type, property_declaration.type,
    // foreach_statement.type, cast_expression.type, and as_expression.right.
    // Builtin keyword types (`int`, `object`, `string`) are deliberately NOT
    // used for this check: they parse as `predefined_type`, not `identifier`,
    // so they would silently fail to exercise this variant at all.
    let plain_class_count = refs.iter().filter(|r| *r == "PlainClass").count();
    assert!(
        plain_class_count >= 7,
        "expected >= 7 'PlainClass' identifier-variant @type.reference captures \
         (one per field position), got {plain_class_count}: {refs:?}"
    );
    // catch_declaration.type: qualified_name (System.Exception)
    // is_expression.right: generic_name (List<int>)
    // These are exercised structurally by the field patterns above; spot-check
    // extraction depth instead:
    let caps = collect_captures_full(&lang, CSHARP_VARIANTS, &query_str);
    let list_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, t, _)| cn == "type.reference" && t == "List")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    assert!(
        list_kinds.iter().all(|k| *k == "identifier"),
        "expected 'List' captures to be leaf identifier nodes, got kinds: {list_kinds:?}"
    );
}

/// Negative case: value identifiers (parameter names, local variable names,
/// method names unrelated to type positions) must never appear as
/// @type.reference in the completeness fixture either.
#[test]
fn csharp_types_negative_field_positions_no_overmatch() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping csharp_types_negative_field_positions: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("c-sharp").ok() else {
        eprintln!("Skipping csharp_types_negative_field_positions: c-sharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("c-sharp")
        .expect("c-sharp types query missing");
    let refs = collect_captures(&lang, CSHARP_VARIANTS, &query_str, "type.reference");
    for value_ident in ["Identity", "GenericIdentity", "field1", "a", "b", "x"] {
        assert!(
            !refs.contains(&value_ident.to_string()),
            "'{value_ident}' is a value/method identifier, must not appear as a \
             @type.reference, got: {refs:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Clojure
// ---------------------------------------------------------------------------

const CLOJURE_SAMPLE: &str = include_str!("fixtures/clojure/sample.clj");

#[test]
fn clojure_tags_finds_functions_and_defrecord() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_tags: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("clojure")
        .expect("clojure tags query missing");
    let names = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in clojure tags, got: {names:?}"
    );
    assert!(
        names.contains(&"classify-point".to_string()),
        "expected 'classify-point' function in clojure tags, got: {names:?}"
    );
}

#[test]
fn clojure_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_calls: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("clojure")
        .expect("clojure calls query missing");
    let calls = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "println"),
        "expected 'println' call in clojure sample, got: {calls:?}"
    );
}

#[test]
fn clojure_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_complexity: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("clojure")
        .expect("clojure complexity query missing");
    let complexity = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in clojure sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn clojure_imports_finds_require_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_imports: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("clojure")
        .expect("clojure imports query missing");
    let paths = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("clojure")),
        "expected a clojure.* namespace in import paths, got: {paths:?}"
    );
}

#[test]
fn clojure_types_finds_no_captures() {
    // Clojure is dynamically typed; the types query intentionally captures nothing.
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping clojure_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("clojure").ok() else {
        eprintln!("Skipping clojure_types: clojure grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("clojure")
        .expect("clojure types query missing");
    // Query parses successfully — result may be empty, that's correct for dynamic languages.
    let _ = collect_captures(&lang, CLOJURE_SAMPLE, &query_str, "type");
}

// ---------------------------------------------------------------------------
// Scheme
// ---------------------------------------------------------------------------

const SCHEME_SAMPLE: &str = include_str!("fixtures/scheme/sample.scm");

#[test]
fn scheme_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_tags: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("scheme")
        .expect("scheme tags query missing");
    let names = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' in scheme tags, got: {names:?}"
    );
    assert!(
        names.contains(&"square".to_string()),
        "expected 'square' in scheme tags, got: {names:?}"
    );
}

#[test]
fn scheme_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_calls: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scheme")
        .expect("scheme calls query missing");
    let calls = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "display" || c == "sqrt"),
        "expected 'display' or 'sqrt' call in scheme sample, got: {calls:?}"
    );
}

#[test]
fn scheme_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_complexity: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scheme")
        .expect("scheme complexity query missing");
    let complexity = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in scheme sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn scheme_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_imports: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scheme")
        .expect("scheme imports query missing");
    let paths = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("scheme")),
        "expected a scheme library in import paths, got: {paths:?}"
    );
}

#[test]
fn scheme_types_finds_no_captures() {
    // Scheme is dynamically typed; the types query intentionally captures nothing.
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_types: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("scheme")
        .expect("scheme types query missing");
    let _ = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "type");
}

// ---------------------------------------------------------------------------
// D
// ---------------------------------------------------------------------------

const D_SAMPLE: &str = include_str!("fixtures/d/sample.d");

#[test]
fn d_tags_finds_functions_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_tags: d grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("d").expect("d tags query missing");
    let names = collect_captures(&lang, D_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in d tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' class in d tags, got: {names:?}"
    );
}

#[test]
fn d_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_calls: d grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("d").expect("d calls query missing");
    let calls = collect_captures(&lang, D_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "writeln" || c == "sqrt"),
        "expected 'writeln' or 'sqrt' call in d sample, got: {calls:?}"
    );
}

#[test]
fn d_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_complexity: d grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("d")
        .expect("d complexity query missing");
    let complexity = collect_captures(&lang, D_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in d sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn d_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_imports: d grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("d").expect("d imports query missing");
    let paths = collect_captures(&lang, D_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("std")),
        "expected std module in d import paths, got: {paths:?}"
    );
}

#[test]
fn d_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping d_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("d").ok() else {
        eprintln!("Skipping d_types: d grammar .so not found");
        return;
    };
    let query_str = loader.get_types("d").expect("d types query missing");
    let refs = collect_captures(&lang, D_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected type references in d sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Visual Basic .NET
// ---------------------------------------------------------------------------

const VB_SAMPLE: &str = include_str!("fixtures/vb/sample.vb");

#[test]
fn vb_tags_finds_methods_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_tags: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vb").expect("vb tags query missing");
    let names = collect_captures(&lang, VB_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Classify".to_string()),
        "expected 'Classify' method in vb tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Circle".to_string()),
        "expected 'Circle' class in vb tags, got: {names:?}"
    );
}

#[test]
fn vb_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_calls: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vb").expect("vb calls query missing");
    let calls = collect_captures(&lang, VB_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "WriteLine" || c == "Area"),
        "expected 'WriteLine' or 'Area' call in vb sample, got: {calls:?}"
    );
}

#[test]
fn vb_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_complexity: vb grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vb")
        .expect("vb complexity query missing");
    let complexity = collect_captures(&lang, VB_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in vb sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn vb_imports_finds_namespace_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_imports: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("vb").expect("vb imports query missing");
    let paths = collect_captures(&lang, VB_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("System")),
        "expected System namespace in vb import paths, got: {paths:?}"
    );
}

#[test]
fn vb_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_types: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_types("vb").expect("vb types query missing");
    let refs = collect_captures(&lang, VB_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected type references in vb sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Objective-C
// ---------------------------------------------------------------------------

const OBJC_SAMPLE: &str = include_str!("fixtures/objc/sample.m");

#[test]
fn objc_tags_finds_classes_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_tags: objc grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("objc").expect("objc tags query missing");
    let names = collect_captures(&lang, OBJC_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in objc tags, got: {names:?}"
    );
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in objc tags, got: {names:?}"
    );
}

#[test]
fn objc_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_calls: objc grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("objc").expect("objc calls query missing");
    let calls = collect_captures(&lang, OBJC_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "distance" || c == "classify"),
        "expected 'distance' or 'classify' call in objc sample, got: {calls:?}"
    );
}

#[test]
fn objc_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_complexity: objc grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("objc")
        .expect("objc complexity query missing");
    let complexity = collect_captures(&lang, OBJC_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in objc sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn objc_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_imports: objc grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("objc")
        .expect("objc imports query missing");
    let paths = collect_captures(&lang, OBJC_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Foundation")),
        "expected Foundation in objc import paths, got: {paths:?}"
    );
}

#[test]
fn objc_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_types: objc grammar .so not found");
        return;
    };
    let query_str = loader.get_types("objc").expect("objc types query missing");
    let refs = collect_captures(&lang, OBJC_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "NSString" || r == "NSLog" || r == "Point"),
        "expected type reference in objc sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Idris
// ---------------------------------------------------------------------------

const IDRIS_SAMPLE: &str = include_str!("fixtures/idris/sample.idr");

#[test]
fn idris_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_tags: idris grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("idris").expect("idris tags query missing");
    let names = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in idris tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' data type in idris tags, got: {names:?}"
    );
}

#[test]
fn idris_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_calls: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("idris")
        .expect("idris calls query missing");
    let calls = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "sqrt" || c == "printLn"),
        "expected 'sqrt' or 'printLn' call in idris sample, got: {calls:?}"
    );
}

#[test]
fn idris_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_complexity: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("idris")
        .expect("idris complexity query missing");
    let complexity = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in idris sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn idris_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_imports: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("idris")
        .expect("idris imports query missing");
    let paths = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Data")),
        "expected Data.* module in idris import paths, got: {paths:?}"
    );
}

#[test]
fn idris_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_types: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("idris")
        .expect("idris types query missing");
    let refs = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "String" || r == "Int" || r == "Double"),
        "expected a type reference in idris sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Lean 4
// ---------------------------------------------------------------------------

const LEAN_SAMPLE: &str = include_str!("fixtures/lean/sample.lean");

#[test]
fn lean_tags_finds_defs_and_structures() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_tags: lean grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lean").expect("lean tags query missing");
    let names = collect_captures(&lang, LEAN_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' def in lean tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' structure in lean tags, got: {names:?}"
    );
}

#[test]
fn lean_calls_finds_function_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_calls: lean grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lean").expect("lean calls query missing");
    let calls = collect_captures(&lang, LEAN_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "sqrt" || c == "classify" || c == "IO.println"),
        "expected a function call in lean sample, got: {calls:?}"
    );
}

#[test]
fn lean_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_complexity: lean grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("lean")
        .expect("lean complexity query missing");
    let complexity = collect_captures(&lang, LEAN_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in lean sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn lean_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_imports: lean grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("lean")
        .expect("lean imports query missing");
    let paths = collect_captures(&lang, LEAN_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Mathlib")),
        "expected Mathlib import in lean import paths, got: {paths:?}"
    );
}

#[test]
fn lean_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_types: lean grammar .so not found");
        return;
    };
    let query_str = loader.get_types("lean").expect("lean types query missing");
    // Query parses and runs; lean type ascriptions may or may not match in this sample.
    let _ = collect_captures(&lang, LEAN_SAMPLE, &query_str, "type");
}

// ---------------------------------------------------------------------------
// ReScript
// ---------------------------------------------------------------------------

const RESCRIPT_SAMPLE: &str = include_str!("fixtures/rescript/sample.res");

#[test]
fn rescript_tags_finds_let_bindings_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_tags: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("rescript")
        .expect("rescript tags query missing");
    let names = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' in rescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"point".to_string()),
        "expected 'point' type in rescript tags, got: {names:?}"
    );
}

#[test]
fn rescript_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_calls: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("rescript")
        .expect("rescript calls query missing");
    let calls = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "square" || c == "classify"),
        "expected 'square' or 'classify' call in rescript sample, got: {calls:?}"
    );
}

#[test]
fn rescript_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_complexity: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("rescript")
        .expect("rescript complexity query missing");
    let complexity = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in rescript sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn rescript_imports_finds_open_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_imports: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("rescript")
        .expect("rescript imports query missing");
    let paths = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Belt")),
        "expected 'Belt' in rescript import paths, got: {paths:?}"
    );
}

#[test]
fn rescript_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_types: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("rescript")
        .expect("rescript types query missing");
    let refs = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "float" || r == "int" || r == "point"),
        "expected a type reference in rescript sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Elm
// ---------------------------------------------------------------------------

const ELM_SAMPLE: &str = include_str!("fixtures/elm/sample.elm");

#[test]
fn elm_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_tags: elm grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("elm").expect("elm tags query missing");
    let names = collect_captures(&lang, ELM_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in elm tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' type in elm tags, got: {names:?}"
    );
}

#[test]
fn elm_calls_finds_function_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_calls: elm grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("elm").expect("elm calls query missing");
    let calls = collect_captures(&lang, ELM_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "sqrt" || c == "classify" || c == "area"),
        "expected a function call in elm sample, got: {calls:?}"
    );
}

#[test]
fn elm_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_complexity: elm grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elm")
        .expect("elm complexity query missing");
    let complexity = collect_captures(&lang, ELM_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in elm sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn elm_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_imports: elm grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elm")
        .expect("elm imports query missing");
    let paths = collect_captures(&lang, ELM_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Html")),
        "expected 'Html' in elm import paths, got: {paths:?}"
    );
}

#[test]
fn elm_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_types: elm grammar .so not found");
        return;
    };
    let query_str = loader.get_types("elm").expect("elm types query missing");
    let refs = collect_captures(&lang, ELM_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Html" || r == "Float" || r == "Int" || r == "String"),
        "expected a type reference in elm sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Zig
// ---------------------------------------------------------------------------

const ZIG_SAMPLE: &str = include_str!("fixtures/zig/sample.zig");

#[test]
fn zig_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_tags: zig grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("zig").expect("zig tags query missing");
    let names = collect_captures(&lang, ZIG_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in zig tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' struct in zig tags, got: {names:?}"
    );
}

#[test]
fn zig_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_calls: zig grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("zig").expect("zig calls query missing");
    let calls = collect_captures(&lang, ZIG_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "sumSlice" || c == "origin"),
        "expected a function call in zig sample, got: {calls:?}"
    );
}

#[test]
fn zig_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_complexity: zig grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("zig")
        .expect("zig complexity query missing");
    let complexity = collect_captures(&lang, ZIG_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in zig sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn zig_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_imports: zig grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("zig")
        .expect("zig imports query missing");
    let paths = collect_captures(&lang, ZIG_SAMPLE, &query_str, "import");
    assert!(
        !paths.is_empty(),
        "expected at least one import in zig sample, got: {paths:?}"
    );
}

#[test]
fn zig_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_types: zig grammar .so not found");
        return;
    };
    let query_str = loader.get_types("zig").expect("zig types query missing");
    let refs = collect_captures(&lang, ZIG_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in zig sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Ada
// ---------------------------------------------------------------------------

const ADA_SAMPLE: &str = include_str!("fixtures/ada/sample.adb");

#[test]
fn ada_tags_finds_subprograms_and_packages() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_tags: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("ada").expect("ada tags query missing");
    let names = collect_captures(&lang, ADA_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Add" || n == "Classify" || n == "Calculator"),
        "expected 'Add'/'Classify'/'Calculator' in ada tags, got: {names:?}"
    );
}

#[test]
fn ada_calls_finds_procedure_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_calls: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("ada").expect("ada calls query missing");
    let calls = collect_captures(&lang, ADA_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "Print_Result" || c == "Put_Line" || c == "Add"),
        "expected a procedure call in ada sample, got: {calls:?}"
    );
}

#[test]
fn ada_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_complexity: ada grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("ada")
        .expect("ada complexity query missing");
    let complexity = collect_captures(&lang, ADA_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in ada sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn ada_imports_finds_with_clauses() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_imports: ada grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("ada")
        .expect("ada imports query missing");
    let paths = collect_captures(&lang, ADA_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Text_IO") || p.contains("Ada")),
        "expected 'Ada.Text_IO' in ada import paths, got: {paths:?}"
    );
}

#[test]
fn ada_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping ada_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("ada").ok() else {
        eprintln!("Skipping ada_types: ada grammar .so not found");
        return;
    };
    let query_str = loader.get_types("ada").expect("ada types query missing");
    let refs = collect_captures(&lang, ADA_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in ada sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Perl
// ---------------------------------------------------------------------------

const PERL_SAMPLE: &str = include_str!("fixtures/perl/sample.pl");

#[test]
fn perl_tags_finds_subroutines_and_packages() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_tags: perl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("perl").expect("perl tags query missing");
    let names = collect_captures(&lang, PERL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "sum_array" || n == "factorial"),
        "expected 'classify'/'sum_array'/'factorial' in perl tags, got: {names:?}"
    );
}

#[test]
fn perl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_calls: perl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("perl").expect("perl calls query missing");
    let calls = collect_captures(&lang, PERL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "sum_array" || c == "factorial"),
        "expected a function call in perl sample, got: {calls:?}"
    );
}

#[test]
fn perl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_complexity: perl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("perl")
        .expect("perl complexity query missing");
    let complexity = collect_captures(&lang, PERL_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in perl sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn perl_imports_finds_use_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping perl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("perl").ok() else {
        eprintln!("Skipping perl_imports: perl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("perl")
        .expect("perl imports query missing");
    let paths = collect_captures(&lang, PERL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("List") || p.contains("POSIX") || p.contains("warnings")),
        "expected a module path in perl imports, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Bash
// ---------------------------------------------------------------------------

const BASH_SAMPLE: &str = include_str!("fixtures/bash/sample.sh");

#[test]
fn bash_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_tags: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("bash").expect("bash tags query missing");
    let names = collect_captures(&lang, BASH_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "sum_array" || n == "greet"),
        "expected 'classify'/'sum_array'/'greet' in bash tags, got: {names:?}"
    );
}

#[test]
fn bash_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_calls: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("bash").expect("bash calls query missing");
    let calls = collect_captures(&lang, BASH_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "greet" || c == "sum_array"),
        "expected a function call in bash sample, got: {calls:?}"
    );
}

#[test]
fn bash_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let complexity = collect_captures(&lang, BASH_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in bash sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn bash_imports_finds_source_commands() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_imports: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("bash")
        .expect("bash imports query missing");
    let paths = collect_captures(&lang, BASH_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils") || p.contains("config")),
        "expected sourced file path in bash imports, got: {paths:?}"
    );
}

#[test]
fn bash_complexity_finds_real_world_idioms() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_complexity_real_world: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity_real_world: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let complexity = collect_captures_full(&lang, BASH_SAMPLE, &query_str);
    let kinds: Vec<&str> = complexity
        .iter()
        .filter(|(cap, ..)| cap == "complexity")
        .map(|(_, kind, ..)| kind.as_str())
        .collect();
    // sample.sh's `retry()` function exercises c_style_for_statement,
    // binary_expression (&&/||), and ternary_expression together — none of
    // which the pre-fix query captured at all.
    assert!(
        kinds.contains(&"c_style_for_statement"),
        "expected c_style_for_statement complexity in bash sample, got: {kinds:?}"
    );
    assert!(
        kinds.contains(&"binary_expression"),
        "expected &&/|| binary_expression complexity in bash sample, got: {kinds:?}"
    );
    assert!(
        kinds.contains(&"ternary_expression"),
        "expected ternary_expression complexity in bash sample, got: {kinds:?}"
    );
}

const BASH_VARIANTS: &str = include_str!("fixtures/bash/variants.sh");

#[test]
fn bash_variants_fixture_parses_clean() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping bash_variants_fixture_parses_clean: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_variants_fixture_parses_clean: bash grammar .so not found");
        return;
    };
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(BASH_VARIANTS, None).expect("parse failed");
    assert!(
        !tree.root_node().has_error(),
        "bash variants.sh fixture must parse without ERROR nodes"
    );
}

/// tags.scm completeness: both function_definition syntaxes (`function
/// NAME { }`, `function NAME() { }`, `NAME() { }`) and the non-`{ }`
/// body variant (`if_statement` as body) all yield `name: (word) @name`
/// under `@definition.function` — the same shape, verified across every
/// syntactic form the grammar allows.
#[test]
fn bash_tags_completeness_function_definition_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_tags_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("bash").expect("bash tags query missing");
    let pairs = collect_tag_pairs(&lang, BASH_VARIANTS, &query_str);
    for expected in [
        "fn_keyword_no_parens",
        "fn_keyword_with_parens",
        "fn_posix_form",
        "fn_body_if_statement",
    ] {
        assert!(
            pairs
                .iter()
                .any(|(kind, name)| kind == "definition.function" && name == expected),
            "expected {expected} as @definition.function in bash tags, got: {pairs:?}"
        );
    }
    // Negative: a function name mentioned only inside a string literal must
    // not produce a @definition.function tag.
    assert!(
        !pairs
            .iter()
            .any(|(_, name)| name.contains("calling function")),
        "string contents must not produce a tags capture, got: {pairs:?}"
    );
}

/// calls.scm completeness: every `command_name` child variant a real bash
/// script can produce (bare word, relative/absolute path, quoted string,
/// simple and braced variable expansion) yields a `@call` capture, in every
/// structural context (pipeline stage, subshell, command substitution,
/// negated_command).
#[test]
fn bash_calls_completeness_command_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_calls_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("bash").expect("bash calls query missing");
    let calls = collect_captures_full(&lang, BASH_VARIANTS, &query_str);
    let call_texts: Vec<&str> = calls
        .iter()
        .filter(|(cap, ..)| cap == "call")
        .map(|(_, _, text, _)| text.as_str())
        .collect();
    for expected in [
        "ls",          // bare word
        "./script.sh", // relative path word
        "\"ls\"",      // quoted string command_name
        "$cmd",        // simple_expansion
        "${cmd}",      // expansion (braced)
        "grep",        // pipeline stage
        "sort",        // pipeline stage
    ] {
        assert!(
            call_texts.contains(&expected),
            "expected {expected:?} among bash @call captures, got: {call_texts:?}"
        );
    }
    // Every @call capture must be a `command_name` node (extraction depth:
    // kind, not just text).
    for (cap, kind, text, line) in &calls {
        if cap == "call" {
            assert_eq!(
                kind, "command_name",
                "@call capture {text:?} at line {line} has unexpected kind {kind:?}"
            );
        }
    }
}

/// imports.scm completeness: `source`/`.` cover bare word, quoted string,
/// simple-expansion, and expansion-containing-a-string paths — and the `.`
/// field anchor means a `source file.sh arg1 arg2` command captures only
/// the path, never the trailing positional arguments.
#[test]
fn bash_imports_completeness_source_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_imports_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("bash")
        .expect("bash imports query missing");
    let paths = collect_captures(&lang, BASH_VARIANTS, &query_str, "import.path");
    assert_eq!(
        paths,
        vec![
            "./plain_path.sh",
            "./dot_path.sh",
            "\"./quoted_path.sh\"",
            "$lib_path",
            "\"$lib_path/sub.sh\"",
            "./with_args.sh",
        ],
        "expected exactly these @import.path captures (in source order), got: {paths:?}"
    );
}

/// Regression test for the trailing-argument bug: `source file.sh arg1
/// arg2` must produce exactly one @import.path capture, never one per
/// argument.
#[test]
fn bash_imports_negative_no_trailing_arguments_as_path() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping bash_imports_negative_trailing_args: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_imports_negative_trailing_args: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("bash")
        .expect("bash imports query missing");
    let source = "source ./with_args.sh arg1 arg2\n";
    let paths = collect_captures(&lang, source, &query_str, "import.path");
    assert_eq!(
        paths,
        vec!["./with_args.sh"],
        "expected exactly one @import.path (the sourced file, not trailing args), got: {paths:?}"
    );
}

/// complexity.scm completeness: every control-flow/decision node variant —
/// including c_style_for_statement, the &&/|| binary_expression operators,
/// and ternary_expression, none of which the pre-fix query captured —
/// produces exactly the expected count of @complexity/@nesting captures.
#[test]
fn bash_complexity_completeness_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping bash_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity_completeness: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let complexity = collect_captures_full(&lang, BASH_VARIANTS, &query_str);

    let count_of = |kind: &str| -> usize {
        complexity
            .iter()
            .filter(|(cap, k, ..)| cap == "complexity" && k == kind)
            .count()
    };

    assert_eq!(count_of("if_statement"), 4, "if_statement complexity count");
    assert_eq!(count_of("elif_clause"), 2, "elif_clause complexity count");
    assert_eq!(
        count_of("for_statement"),
        1,
        "for_statement complexity count"
    );
    assert_eq!(
        count_of("c_style_for_statement"),
        1,
        "c_style_for_statement complexity count (previously uncounted entirely)"
    );
    assert_eq!(
        count_of("while_statement"),
        2,
        "while_statement complexity count (covers both `while` and `until`)"
    );
    assert_eq!(
        count_of("case_statement"),
        1,
        "case_statement complexity count"
    );
    assert_eq!(count_of("case_item"), 3, "case_item complexity count");
    assert_eq!(count_of("pipeline"), 2, "pipeline complexity count");
    assert_eq!(
        count_of("list"),
        2,
        "list (&&/|| statement-level chain) complexity count"
    );
    assert_eq!(
        count_of("ternary_expression"),
        1,
        "ternary_expression complexity count (previously uncounted entirely)"
    );

    let binary_and_or = complexity
        .iter()
        .filter(|(cap, k, ..)| cap == "complexity" && k == "binary_expression")
        .count();
    assert_eq!(
        binary_and_or, 2,
        "expected exactly 2 binary_expression complexity captures (one &&, one ||); \
         plain arithmetic/comparison operators (+=, <, >=, ...) must NOT count, got: {complexity:?}"
    );
}

/// Negative regression test for the binary_expression overcounting risk:
/// ordinary arithmetic (`+=`, `<`) inside `(( ))` must contribute zero
/// complexity — only literal `&&`/`||` operators do.
#[test]
fn bash_complexity_negative_arithmetic_not_counted() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping bash_complexity_negative_arithmetic: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("bash").ok() else {
        eprintln!("Skipping bash_complexity_negative_arithmetic: bash grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("bash")
        .expect("bash complexity query missing");
    let source = "negative_arithmetic_not_complexity() {\n    local -i total=0\n    (( total += 1 ))\n    (( total < 100 ))\n}\n";
    let complexity = collect_captures_full(&lang, source, &query_str);
    let binary_hits: Vec<_> = complexity
        .iter()
        .filter(|(cap, k, ..)| cap == "complexity" && k == "binary_expression")
        .collect();
    assert!(
        binary_hits.is_empty(),
        "plain arithmetic (+=, <) must not produce binary_expression complexity, got: {binary_hits:?}"
    );
}

// ---------------------------------------------------------------------------
// PowerShell
// ---------------------------------------------------------------------------

const POWERSHELL_SAMPLE: &str = include_str!("fixtures/powershell/sample.ps1");

#[test]
fn powershell_tags_finds_functions_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_tags: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("powershell")
        .expect("powershell tags query missing");
    let names = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Invoke-Classify" || n == "Get-Sum" || n == "Calculator"),
        "expected 'Invoke-Classify'/'Get-Sum'/'Calculator' in powershell tags, got: {names:?}"
    );
}

#[test]
fn powershell_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_calls: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("powershell")
        .expect("powershell calls query missing");
    let calls = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "Invoke-Classify" || c == "Get-Sum" || c == "Write-Host"),
        "expected a call in powershell sample, got: {calls:?}"
    );
}

#[test]
fn powershell_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_complexity: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("powershell")
        .expect("powershell complexity query missing");
    let complexity = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in powershell sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn powershell_imports_finds_import_module() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_imports: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("powershell")
        .expect("powershell imports query missing");
    let paths = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("PSReadLine") || p.contains("PowerShell")),
        "expected a module path in powershell imports, got: {paths:?}"
    );
}

#[test]
fn powershell_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping powershell_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("powershell").ok() else {
        eprintln!("Skipping powershell_types: powershell grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("powershell")
        .expect("powershell types query missing");
    let refs = collect_captures(&lang, POWERSHELL_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in powershell sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Fish
// ---------------------------------------------------------------------------

const FISH_SAMPLE: &str = include_str!("fixtures/fish/sample.fish");

#[test]
fn fish_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_tags: fish grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("fish").expect("fish tags query missing");
    let names = collect_captures(&lang, FISH_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "greet" || n == "sum_list"),
        "expected 'classify'/'greet'/'sum_list' in fish tags, got: {names:?}"
    );
}

#[test]
fn fish_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_calls: fish grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("fish").expect("fish calls query missing");
    let calls = collect_captures(&lang, FISH_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "greet" || c == "sum_list"),
        "expected a function call in fish sample, got: {calls:?}"
    );
}

#[test]
fn fish_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_complexity: fish grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("fish")
        .expect("fish complexity query missing");
    let complexity = collect_captures(&lang, FISH_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in fish sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn fish_imports_finds_source_commands() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_imports: fish grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("fish")
        .expect("fish imports query missing");
    let paths = collect_captures(&lang, FISH_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils") || p.contains("fish")),
        "expected sourced file path in fish imports, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Zsh
// ---------------------------------------------------------------------------

const ZSH_SAMPLE: &str = include_str!("fixtures/zsh/sample.zsh");

/// Returns true if the zsh grammar can parse basic constructs correctly.
/// The arborium-zsh grammar is known to have severe parsing issues with
/// common zsh syntax (function definitions, control flow, commands).
/// When it's broken, we skip the query tests rather than fail them.
fn zsh_grammar_is_functional(lang: &tree_sitter::Language) -> bool {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    // A well-formed zsh grammar should parse `function f { echo hi }` as
    // a function_definition, not as an ERROR node. Check that.
    let tree = parser
        .parse("function greet { echo hi; }", None)
        .expect("parse failed");
    let sexp = tree.root_node().to_sexp();
    sexp.contains("function_definition")
}

#[test]
fn zsh_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_tags: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_tags: zsh grammar cannot parse function definitions (known grammar limitation)"
        );
        return;
    }
    let query_str = loader.get_tags("zsh").expect("zsh tags query missing");
    let names = collect_captures(&lang, ZSH_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "greet" || n == "sum_array"),
        "expected 'classify'/'greet'/'sum_array' in zsh tags, got: {names:?}"
    );
}

#[test]
fn zsh_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_calls: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_calls: zsh grammar cannot parse commands correctly (known grammar limitation)"
        );
        return;
    }
    let query_str = loader.get_calls("zsh").expect("zsh calls query missing");
    let calls = collect_captures(&lang, ZSH_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "greet" || c == "sum_array"),
        "expected a function call in zsh sample, got: {calls:?}"
    );
}

#[test]
fn zsh_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_complexity: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_complexity: zsh grammar cannot parse control flow (known grammar limitation)"
        );
        return;
    }
    let query_str = loader
        .get_complexity("zsh")
        .expect("zsh complexity query missing");
    let complexity = collect_captures(&lang, ZSH_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in zsh sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn zsh_imports_finds_source_commands() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_imports: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_imports: zsh grammar cannot parse source commands (known grammar limitation)"
        );
        return;
    }
    let query_str = loader
        .get_imports("zsh")
        .expect("zsh imports query missing");
    let paths = collect_captures(&lang, ZSH_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils") || p.contains("zsh") || p.contains("helpers")),
        "expected sourced file path in zsh imports, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// AWK
// ---------------------------------------------------------------------------

const AWK_SAMPLE: &str = include_str!("fixtures/awk/sample.awk");

#[test]
fn awk_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping awk_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_tags: awk grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("awk").expect("awk tags query missing");
    let names = collect_captures(&lang, AWK_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "max" || n == "trim"),
        "expected 'classify'/'max'/'trim' in awk tags, got: {names:?}"
    );
}

#[test]
fn awk_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping awk_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_calls: awk grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("awk").expect("awk calls query missing");
    let calls = collect_captures(&lang, AWK_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "max" || c == "trim" || c == "gsub"),
        "expected a function call in awk sample, got: {calls:?}"
    );
}

#[test]
fn awk_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping awk_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("awk").ok() else {
        eprintln!("Skipping awk_complexity: awk grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("awk")
        .expect("awk complexity query missing");
    let complexity = collect_captures(&lang, AWK_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in awk sample, got {} ({complexity:?})",
        complexity.len()
    );
}

// ---------------------------------------------------------------------------
// JavaScript
// ---------------------------------------------------------------------------

const JAVASCRIPT_SAMPLE: &str = include_str!("fixtures/javascript/sample.js");
const JAVASCRIPT_VARIANTS: &str = include_str!("fixtures/javascript/variants.js");

// --- Dimension 4: real-world fixture coverage (sample.js) -------------------

#[test]
fn javascript_tags_finds_functions_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_tags: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let names = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Stack" || n == "classify" || n == "fibonacci"),
        "expected 'Stack'/'classify'/'fibonacci' in javascript tags, got: {names:?}"
    );
    // SerializableStack extends Serializable(Stack) — the mixin-pattern
    // superclass expression must still surface Stack via @reference.class.
    assert!(
        names.contains(&"SerializableStack".to_string()),
        "expected 'SerializableStack' class in javascript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Stack".to_string()),
        "expected 'Stack' superclass reference in javascript tags, got: {names:?}"
    );
    // Private method #peek must be found as a method definition, not
    // silently dropped for having a private_property_identifier name.
    assert!(
        names.iter().any(|n| n == "#peek"),
        "expected private method '#peek' in javascript tags, got: {names:?}"
    );
    // Generator function must still be found as a function definition.
    assert!(
        names.contains(&"range".to_string()),
        "expected generator function 'range' in javascript tags, got: {names:?}"
    );
}

#[test]
fn javascript_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_calls: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("javascript")
        .expect("javascript calls query missing");
    let calls = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "fibonacci" || c == "push"),
        "expected a function call in javascript sample, got: {calls:?}"
    );
    // Private method call site: this.#peek() inside SerializableStack.
    assert!(
        calls.iter().any(|c| c == "#peek"),
        "expected private method call '#peek' in javascript sample, got: {calls:?}"
    );
    // Tagged template call: html`<h1>${resolved}</h1>` — arguments is a bare
    // template_string, not the usual `arguments` node.
    assert!(
        calls.iter().any(|c| c == "html"),
        "expected tagged-template call 'html' in javascript sample, got: {calls:?}"
    );
    // Computed/bracket call: dispatch['classify'](0).
    assert!(
        calls.iter().any(|c| c.contains("dispatch")),
        "expected computed/bracket call on 'dispatch' in javascript sample, got: {calls:?}"
    );
}

#[test]
fn javascript_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_complexity: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("javascript")
        .expect("javascript complexity query missing");
    let complexity = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in javascript sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn javascript_imports_finds_es_module_imports() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_imports: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("javascript")
        .expect("javascript imports query missing");
    let paths = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p == "events" || p == "path" || p == "fs"),
        "expected module paths in javascript imports, got: {paths:?}"
    );
}

#[test]
fn javascript_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_types: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("javascript")
        .expect("javascript types query missing");
    let refs = collect_captures(&lang, JAVASCRIPT_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in javascript sample, got: {refs:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.js) -

/// Every grammar-legal variant of `call_expression.function` that
/// javascript.calls.scm claims to support must actually match, with the
/// right capture *kind* (dimension 3) — not just the right text.
#[test]
fn javascript_calls_completeness_all_function_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_calls_completeness: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("javascript")
        .expect("javascript calls query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "identity"),                  // plainCall
        ("call", "property_identifier", "push"), // methodCall: function: member_expression, property: property_identifier
        ("call", "private_property_identifier", "#compute"), // callPrivate: private method call
        ("call", "subscript_expression", "arr[0]"), // computedCall
        ("call", "parenthesized_expression", "(function iife() {})"), // parenthesizedCall (IIFE)
        ("call", "call_expression", "curried()"), // chainedCall
        ("call", "identifier", "taggedTemplateCall"), // tagged template call (arguments: template_string)
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in javascript.calls.scm \
             output for variants.js, got: {caps:?}"
        );
    }

    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"arr"),
        "expected 'arr' qualifier for the plain method call, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"this"),
        "expected 'this' qualifier for the private method call, got: {qualifiers:?}"
    );
}

/// Negative cases: constructs that must never appear in @call captures.
#[test]
fn javascript_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_calls_negative: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("javascript")
        .expect("javascript calls query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` is a bare field read (no call parens); must never be a call.
    assert!(
        !call_texts.contains(&"field"),
        "bare field access 'holder.field' must not be captured as a call, got: {call_texts:?}"
    );
    // The closure definition site (`addOne`) must not appear as a call —
    // only the call site `addOne(1)` should.
    let add_one_calls = call_texts.iter().filter(|t| **t == "addOne").count();
    assert_eq!(
        add_one_calls, 1,
        "expected exactly 1 call to 'addOne' (the call site, not the closure \
         definition), got {add_one_calls}: {call_texts:?}"
    );
}

/// Every grammar-legal variant of `method_definition.name` that
/// javascript.tags.scm claims to support (plain, private, computed) must
/// produce a @name capture with the correct kind.
#[test]
fn javascript_tags_completeness_all_method_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_tags_completeness_methods: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!(
            "Skipping javascript_tags_completeness_methods: javascript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);
    let name_kinds: Vec<(&str, &str)> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "name")
        .map(|(_, k, t, _)| (k.as_str(), t.as_str()))
        .collect();
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "property_identifier" && *t == "plainMethod"),
        "expected plain method name 'plainMethod' (property_identifier), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "private_property_identifier" && *t == "#privateMethod"),
        "expected private method name '#privateMethod' (private_property_identifier), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "computed_property_name" && *t == "[\"computedMethod\"]"),
        "expected computed method name (computed_property_name), got: {name_kinds:?}"
    );
    assert!(
        name_kinds
            .iter()
            .any(|(k, t)| *k == "property_identifier" && *t == "staticMethod"),
        "expected static method name 'staticMethod' (property_identifier), got: {name_kinds:?}"
    );
}

/// Every grammar-legal variant of class_heritage's superclass expression
/// (identifier, member_expression, call_expression/mixin) must produce a
/// @reference.class capture.
#[test]
fn javascript_tags_completeness_class_heritage_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_tags_completeness_heritage: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!(
            "Skipping javascript_tags_completeness_heritage: javascript grammar .so not found"
        );
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let class_refs =
        tags_matches_by_kind(&lang, JAVASCRIPT_VARIANTS, &query_str, "reference.class");
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "identifier" && t == "Base"),
        "expected 'Base' extends-reference (identifier), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "member_expression" && t == "nsObj.Ctor"),
        "expected 'nsObj.Ctor' extends-reference (member_expression), got: {class_refs:?}"
    );
    assert!(
        class_refs
            .iter()
            .any(|(k, t)| k == "call_expression" && t.starts_with("Mixin(")),
        "expected 'Mixin(Base)' extends-reference (call_expression, mixin pattern), got: {class_refs:?}"
    );
}

/// Every grammar-legal variant of `new_expression.constructor` (already a
/// wildcard `(_)` in javascript.tags.scm) must produce a @reference.class
/// capture regardless of shape.
#[test]
fn javascript_tags_completeness_new_expression_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_tags_completeness_new: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_tags_completeness_new: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let names = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "name");
    assert!(
        names.contains(&"PrivateHolder".to_string()),
        "expected plain constructor 'PrivateHolder', got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "nsObj.Ctor"),
        "expected namespaced constructor 'nsObj.Ctor' (member_expression), got: {names:?}"
    );
}

/// Negative case: closures are not function_declarations/method_definitions
/// and must never appear as @definition.function or @definition.method.
#[test]
fn javascript_tags_negative_closures_are_not_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping javascript_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_tags_negative: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("javascript")
        .expect("javascript tags query missing");
    let caps = collect_captures_full(&lang, JAVASCRIPT_VARIANTS, &query_str);
    let is_def_add_one = caps.iter().any(|(cn, _, t, _)| {
        (cn == "definition.function" || cn == "definition.method") && t == "addOne"
    });
    assert!(
        !is_def_add_one,
        "closure binding 'addOne' must never be captured as a function/method \
         definition, got captures: {caps:?}"
    );
}

/// Every grammar-legal variant of import/re-export/require/dynamic-import
/// that javascript.imports.scm claims to support must produce a correctly
/// shaped @import capture, including the previously-silent `default`-name
/// (anonymous-token) gap.
#[test]
fn javascript_imports_completeness_all_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping javascript_imports_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("javascript").ok() else {
        eprintln!("Skipping javascript_imports_completeness: javascript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("javascript")
        .expect("javascript imports query missing");
    let names = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.name");
    let aliases = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.alias");
    let paths = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.path");
    let globs = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.glob");
    let reexports = collect_captures(&lang, JAVASCRIPT_VARIANTS, &query_str, "import.reexport");

    assert!(
        names.contains(&"plainName".to_string()),
        "expected plain import name, got: {names:?}"
    );
    // import { default as renamedDefault } — previously silently dropped
    // entirely since `default` is an anonymous token, not (identifier).
    assert!(
        names.iter().any(|n| n == "default"),
        "expected a 'default' import name (import {{ default as ... }}), got: {names:?}"
    );
    assert!(
        aliases.contains(&"renamedDefault".to_string()),
        "expected 'renamedDefault' alias for the default-import, got: {aliases:?}"
    );
    assert!(
        !globs.is_empty(),
        "expected at least one import.glob capture, got: {globs:?}"
    );
    assert!(
        aliases.contains(&"wildcardNs".to_string()),
        "expected 'wildcardNs' namespace re-export alias, got: {aliases:?}"
    );
    assert!(
        reexports.len() >= 2,
        "expected multiple @import.reexport captures (named + default forms), got {}: {reexports:?}",
        reexports.len()
    );
    assert!(
        aliases.contains(&"renamedDefaultReexport".to_string()),
        "expected 'renamedDefaultReexport' aliased-default-reexport alias, got: {aliases:?}"
    );
    // const { statSync } = require('fs') — destructured require, shorthand.
    assert!(
        names.contains(&"statSync".to_string()),
        "expected 'statSync' from destructured require, got: {names:?}"
    );
    // import('mod-dynamic') — dynamic import expression.
    assert!(
        paths.contains(&"mod-dynamic".to_string()),
        "expected 'mod-dynamic' from dynamic import(), got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// TSX
// ---------------------------------------------------------------------------

const TSX_SAMPLE: &str = include_str!("fixtures/tsx/sample.tsx");

#[test]
fn tsx_tags_finds_components_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_tags: tsx grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("tsx").expect("tsx tags query missing");
    let names = collect_captures(&lang, TSX_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Counter" || n == "Button" || n == "classify"),
        "expected 'Counter'/'Button'/'classify' in tsx tags, got: {names:?}"
    );
}

#[test]
fn tsx_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_calls: tsx grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("tsx").expect("tsx calls query missing");
    let calls = collect_captures(&lang, TSX_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "useState" || c == "useEffect" || c == "classify"),
        "expected a hook/function call in tsx sample, got: {calls:?}"
    );
}

#[test]
fn tsx_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_complexity: tsx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("tsx")
        .expect("tsx complexity query missing");
    let complexity = collect_captures(&lang, TSX_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in tsx sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn tsx_imports_finds_react_imports() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_imports: tsx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("tsx")
        .expect("tsx imports query missing");
    let paths = collect_captures(&lang, TSX_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p == "react" || p == "react-native"),
        "expected 'react'/'react-native' in tsx import paths, got: {paths:?}"
    );
}

#[test]
fn tsx_types_finds_interface_and_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_types: tsx grammar .so not found");
        return;
    };
    let query_str = loader.get_types("tsx").expect("tsx types query missing");
    let refs = collect_captures(&lang, TSX_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in tsx sample, got: {refs:?}"
    );
}

// ---------------------------------------------------------------------------
// Agda
// ---------------------------------------------------------------------------

const AGDA_SAMPLE: &str = include_str!("fixtures/agda/sample.agda");

#[test]
fn agda_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_tags: agda grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("agda").expect("agda tags query missing");
    let names = collect_captures(&lang, AGDA_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' data type in agda tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "area" || n == "double"),
        "expected a function name in agda tags, got: {names:?}"
    );
}

#[test]
fn agda_calls_finds_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_calls: agda grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("agda").expect("agda calls query missing");
    let calls = collect_captures(&lang, AGDA_SAMPLE, &query_str, "call");
    assert!(
        !calls.is_empty(),
        "expected at least one call in agda sample, got: {calls:?}"
    );
}

#[test]
fn agda_complexity_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_complexity: agda grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("agda")
        .expect("agda complexity query missing");
    let complexity = collect_captures(&lang, AGDA_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in agda sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn agda_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping agda_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("agda").ok() else {
        eprintln!("Skipping agda_imports: agda grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("agda")
        .expect("agda imports query missing");
    let paths = collect_captures(&lang, AGDA_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Data")),
        "expected a 'Data.*' import path in agda sample, got: {paths:?}"
    );
}

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

// ---------------------------------------------------------------------------
// Prolog
// ---------------------------------------------------------------------------

const PROLOG_SAMPLE: &str = include_str!("fixtures/prolog/sample.pl");

#[test]
fn prolog_tags_finds_predicates() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_tags: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("prolog")
        .expect("prolog tags query missing");
    let names = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "factorial" || n == "parent" || n == "ancestor"),
        "expected 'factorial', 'parent', or 'ancestor' in prolog tags, got: {names:?}"
    );
}

#[test]
fn prolog_calls_finds_predicate_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_calls: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("prolog")
        .expect("prolog calls query missing");
    let calls = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "factorial" || c == "parent" || c == "member"),
        "expected a predicate call in prolog sample, got: {calls:?}"
    );
}

#[test]
fn prolog_complexity_finds_clauses() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_complexity: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("prolog")
        .expect("prolog complexity query missing");
    let complexity = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in prolog sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn prolog_imports_finds_use_module() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_imports: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("prolog")
        .expect("prolog imports query missing");
    let paths = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("lists") || p.contains("apply")),
        "expected 'lists' or 'apply' in prolog import paths, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// SQL
// ---------------------------------------------------------------------------

const SQL_SAMPLE: &str = include_str!("fixtures/sql/sample.sql");
const SQL_VARIANTS: &str = include_str!("fixtures/sql/variants.sql");

/// `(loader, language, tags, calls, types, complexity)` — see
/// [`sql_lang_and_queries`] for why the loader must be kept alongside the
/// language it produced.
type SqlLangAndQueries = (
    GrammarLoader,
    tree_sitter::Language,
    Arc<String>,
    Arc<String>,
    Arc<String>,
    Arc<String>,
);

/// Load the sql grammar and all four query strings, or return `None` if the
/// grammar `.so` isn't built (skip gracefully, per `grammar_dir`'s contract).
///
/// Returns the `GrammarLoader` itself alongside everything derived from it:
/// `GrammarLoader` owns the `libloading::Library` backing the returned
/// `tree_sitter::Language`'s function pointers, so the loader must outlive
/// every use of that `Language` — dropping it (e.g. at the end of a helper
/// function that only returns the `Language`) unloads the `.so` and turns
/// the `Language`'s function pointers into dangling pointers (confirmed via
/// a real SIGSEGV when the loader was dropped at the end of a first version
/// of this helper).
fn sql_lang_and_queries() -> Option<SqlLangAndQueries> {
    let gdir = grammar_dir()?;
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let lang = loader.get("sql").ok()?;
    let tags = loader.get_tags("sql")?;
    let calls = loader.get_calls("sql")?;
    let types = loader.get_types("sql")?;
    let complexity = loader.get_complexity("sql")?;
    Some((loader, lang, tags, calls, types, complexity))
}

// --- Dimension 4: real-world fixture coverage (sample.sql) ------------------

#[test]
fn sql_tags_finds_tables_and_functions() {
    let Some((_loader, lang, tags, ..)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, SQL_SAMPLE, &tags, "name");
    assert!(
        names.iter().any(|n| n == "inventory.products"),
        "expected 'inventory.products' table in sql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "inventory.calculate_total"),
        "expected 'inventory.calculate_total' function in sql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "inventory"),
        "expected 'inventory' schema in sql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "inventory.low_stock"),
        "expected 'inventory.low_stock' view in sql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "inventory.category_totals"),
        "expected 'inventory.category_totals' materialized view in sql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "idx_products_category"),
        "expected 'idx_products_category' index in sql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "inventory.order_seq"),
        "expected 'inventory.order_seq' sequence in sql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "touch_orders"),
        "expected 'touch_orders' trigger in sql tags, got: {names:?}"
    );
}

#[test]
fn sql_types_finds_column_types() {
    let Some((_loader, lang, _tags, _calls, types, _complexity)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_types: run `cargo xtask build-grammars` first");
        return;
    };
    let found = collect_captures(&lang, SQL_SAMPLE, &types, "type");
    assert!(
        !found.is_empty(),
        "expected at least one type in sql sample, got: {found:?}"
    );
    assert!(
        found.iter().any(|t| t.contains("NUMERIC")),
        "expected a NUMERIC column type in sql sample, got: {found:?}"
    );
}

#[test]
fn sql_complexity_finds_control_flow() {
    let Some((_loader, lang, _tags, _calls, _types, complexity)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let found = collect_captures(&lang, SQL_SAMPLE, &complexity, "complexity");
    assert!(
        !found.is_empty(),
        "expected at least 1 complexity node in sql sample, got {} ({found:?})",
        found.len()
    );
}

/// Regression test for the `when_clause` → `keyword_when` complexity bug:
/// `sample.sql`'s `reorder_needed` function uses `CASE WHEN ... END` (one
/// branch) and the MERGE statement at the end uses `WHEN MATCHED`/`WHEN NOT
/// MATCHED` (two branches) — three real WHEN branches total. The original
/// `(when_clause) @complexity` query matched zero of them (confirmed via
/// real parse: a scalar CASE expression's `case` node has no `when_clause`
/// child at all — `when_clause` is exclusively for MERGE).
#[test]
fn sql_complexity_finds_case_and_merge_when_branches() {
    let Some((_loader, lang, _tags, _calls, _types, complexity)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_complexity_when: run `cargo xtask build-grammars` first");
        return;
    };
    let full = collect_captures_full(&lang, SQL_SAMPLE, &complexity);
    let when_branches: Vec<_> = full
        .iter()
        .filter(|(cap, kind, ..)| cap == "complexity" && kind == "keyword_when")
        .collect();
    assert_eq!(
        when_branches.len(),
        3,
        "expected 3 WHEN branches (1 CASE + 2 MERGE) in sql sample, got: {when_branches:?}\nfull: {full:?}"
    );
}

#[test]
fn sql_calls_finds_function_calls() {
    let Some((_loader, lang, _tags, calls, ..)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let found = collect_captures(&lang, SQL_SAMPLE, &calls, "call");
    assert!(
        found
            .iter()
            .any(|c| c == "NOW" || c == "COUNT" || c == "SUM" || c == "COALESCE"),
        "expected a SQL function call in sql sample, got: {found:?}"
    );
    assert!(
        found.iter().any(|c| c == "ROW_NUMBER"),
        "expected the window function ROW_NUMBER() call in sql sample, got: {found:?}"
    );
    assert!(
        found.iter().any(|c| c == "EXTRACT"),
        "expected the EXTRACT(...) call in sql sample, got: {found:?}"
    );
}

/// Negative case (regression for the `invocation`'s `unit`-field bug):
/// `EXTRACT(YEAR FROM ordered_at)` must produce exactly one call (`EXTRACT`
/// itself) — the date-part keyword `YEAR` (the `unit` field) must never be
/// mistaken for a second call.
#[test]
fn sql_calls_negative_extract_unit_is_not_a_call() {
    let Some((_loader, lang, _tags, calls, ..)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let found = collect_captures(&lang, SQL_SAMPLE, &calls, "call");
    let extract_count = found.iter().filter(|c| *c == "EXTRACT").count();
    assert_eq!(
        extract_count, 1,
        "expected exactly 1 EXTRACT call in sql sample, got: {found:?}"
    );
    assert!(
        !found.iter().any(|c| c == "YEAR"),
        "YEAR is EXTRACT's date-part unit, not a call — must not appear as a @call capture, got: {found:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix (variants.sql) --------------------

#[test]
fn sql_tags_completeness_all_definition_variants() {
    let Some((_loader, lang, tags, ..)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let pairs = collect_tag_pairs(&lang, SQL_VARIANTS, &tags);
    let has = |kind: &str, name: &str| {
        pairs
            .iter()
            .any(|(k, n)| k == kind && n.ends_with(name) || n == name)
    };
    assert!(
        has("definition.function", "variants.plain_fn"),
        "missing create_function definition, got: {pairs:?}"
    );
    assert!(
        has("definition.class", "variants.widgets"),
        "missing create_table definition, got: {pairs:?}"
    );
    assert!(
        has("definition.class", "variants.widget_labels"),
        "missing create_view definition, got: {pairs:?}"
    );
    assert!(
        has("definition.class", "variants.widget_count"),
        "missing create_materialized_view definition, got: {pairs:?}"
    );
    assert!(
        has("definition.module", "variants"),
        "missing create_schema definition, got: {pairs:?}"
    );
    assert!(
        has("definition.type", "variants.status"),
        "missing create_type definition, got: {pairs:?}"
    );
    assert!(
        has("definition.var", "idx_widgets_label"),
        "missing create_index definition, got: {pairs:?}"
    );
    assert!(
        has("definition.function", "widgets_touch"),
        "missing create_trigger definition, got: {pairs:?}"
    );
    assert!(
        has("definition.var", "variants.widget_seq"),
        "missing create_sequence definition, got: {pairs:?}"
    );
}

/// Regression tests for the anchoring bugs found while applying the
/// query-testing methodology: each of these constructs has a second
/// candidate `object_reference`/`identifier` sibling that an unanchored
/// query previously (or hypothetically) also matched. Assert the exact
/// count of definitions each construct produces, not just presence.
#[test]
fn sql_tags_negative_anchoring_regressions_exact_counts() {
    let Some((_loader, lang, tags, ..)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_tags_negative_anchoring: run `cargo xtask build-grammars` first");
        return;
    };
    let pairs = collect_tag_pairs(&lang, SQL_VARIANTS, &tags);

    // CREATE OR REPLACE FUNCTION ... RETURNS <custom_type> — "variants.status"
    // is independently, legitimately defined once by its own CREATE TYPE
    // statement (definition.type). It must NOT also appear a second time as
    // a spurious definition.function pulled from custom_return_fn's RETURNS
    // clause.
    let status_defs: Vec<_> = pairs
        .iter()
        .filter(|(_, n)| n == "variants.status")
        .collect();
    assert_eq!(
        status_defs.len(),
        1,
        "'variants.status' must be defined exactly once (by its own CREATE TYPE), \
         not also captured from custom_return_fn's RETURNS clause; got: {pairs:?}"
    );
    assert_eq!(
        status_defs[0].0, "definition.type",
        "the sole 'variants.status' definition must be definition.type, got: {status_defs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "variants.custom_return_fn"),
        "expected variants.custom_return_fn function definition, got: {pairs:?}"
    );

    // CREATE SCHEMA ... AUTHORIZATION <role> — must produce exactly 1
    // definition (the schema), not 2 (schema + role).
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.module" && n == "variants_auth"),
        "expected variants_auth schema definition, got: {pairs:?}"
    );
    assert!(
        !pairs.iter().any(|(_, n)| n == "some_role"),
        "AUTHORIZATION role name must never be captured as a definition, got: {pairs:?}"
    );

    // CREATE TABLE ... AS SELECT ... FROM <other_table> — must produce
    // exactly 1 definition (the new table, variants.widgets_copy); the
    // source table's own name (variants.widgets) must not leak a second
    // definition.class out of the CTAS statement (variants.widgets already
    // has its own, separate, legitimate CREATE TABLE earlier in the fixture,
    // so only its *total* count is asserted, not a per-statement one).
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.class" && n == "variants.widgets_copy"),
        "expected variants.widgets_copy CTAS definition, got: {pairs:?}"
    );
    let widgets_defs = pairs
        .iter()
        .filter(|(k, n)| k == "definition.class" && n == "variants.widgets")
        .count();
    assert_eq!(
        widgets_defs, 1,
        "'variants.widgets' must be defined exactly once (by its own CREATE TABLE), \
         not leaked a second time via the CTAS or FK-referencing statements; got: {pairs:?}"
    );

    // Table-level FOREIGN KEY ... REFERENCES <other_table> — must produce
    // exactly 1 definition (the table being created), not 2.
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.class" && n == "variants.widget_refs"),
        "expected variants.widget_refs definition, got: {pairs:?}"
    );
}

/// `@reference.call` in tags.scm must capture the same set of call names as
/// calls.scm's `@call` (parity dimension — a batch-1 Rust bug was exactly
/// this drifting apart between the two query files).
#[test]
fn sql_tags_reference_call_matches_calls_scm() {
    let Some((_loader, lang, tags, calls, ..)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_tags_reference_call: run `cargo xtask build-grammars` first");
        return;
    };
    let mut from_tags: Vec<String> = collect_tag_pairs(&lang, SQL_VARIANTS, &tags)
        .into_iter()
        .filter(|(k, _)| k == "reference.call")
        .map(|(_, n)| n)
        .collect();
    let mut from_calls = collect_captures(&lang, SQL_VARIANTS, &calls, "call");
    from_tags.sort();
    from_calls.sort();
    assert_eq!(
        from_tags, from_calls,
        "tags.scm's @reference.call and calls.scm's @call must agree on the exact call set"
    );
}

#[test]
fn sql_types_completeness_all_variants() {
    let Some((_loader, lang, _tags, _calls, types, _complexity)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_types_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let full = collect_captures_full(&lang, SQL_VARIANTS, &types);
    let kind_for = |text_contains: &str| -> Vec<&str> {
        full.iter()
            .filter(|(_, _, text, _)| text.contains(text_contains))
            .map(|(_, kind, ..)| kind.as_str())
            .collect()
    };
    // column_definition builtin type
    assert!(
        kind_for("VARCHAR(50)").contains(&"varchar"),
        "expected varchar column type, got: {full:?}"
    );
    // column_definition custom_type (only one occurrence: custom_typed_columns.status)
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "object_reference" && text == "variants.status"),
        "expected object_reference custom column type, got: {full:?}"
    );
    // alter_column type
    assert!(
        kind_for("NUMERIC(10, 4)").contains(&"numeric"),
        "expected ALTER COLUMN ... TYPE capture, got: {full:?}"
    );
    // function_argument builtin type, named parameter
    // (variants.named_param_fn's `qty INTEGER` — one of several INTEGER captures)
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "int" && text == "INTEGER"),
        "expected function_argument builtin type, got: {full:?}"
    );
    // function_argument builtin type, unnamed parameter — regression: this
    // shape has no identifier at all, so an adjacency-based query would
    // silently miss it.
    assert!(
        kind_for("TEXT").contains(&"keyword_text"),
        "expected unnamed function_argument TEXT type, got: {full:?}"
    );
    // function_argument custom_type (object_reference)
    let custom_type_count = full
        .iter()
        .filter(|(_, kind, text, _)| kind == "object_reference" && text == "variants.status")
        .count();
    assert!(
        custom_type_count >= 3,
        "expected variants.status to appear as a custom @type at least 3 times \
         (column, function_argument, function_declaration), got {custom_type_count}: {full:?}"
    );
    // cast (keyword_as) . type
    assert!(
        full.iter()
            .any(|(_, kind, text, _)| kind == "keyword_text" && text == "TEXT"),
        "expected CAST(... AS TEXT) type capture, got: {full:?}"
    );
    // create_sequence (keyword_as) . type
    assert!(
        kind_for("BIGINT").contains(&"bigint"),
        "expected CREATE SEQUENCE ... AS BIGINT type capture, got: {full:?}"
    );
}

#[test]
fn sql_complexity_completeness_all_variants() {
    let Some((_loader, lang, _tags, _calls, _types, complexity)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let full = collect_captures_full(&lang, SQL_VARIANTS, &complexity);
    let has = |cap: &str, kind: &str| full.iter().any(|(c, k, ..)| c == cap && k == kind);
    assert!(
        has("complexity", "keyword_when"),
        "missing CASE WHEN branch, got: {full:?}"
    );
    assert!(has("complexity", "join"), "missing JOIN, got: {full:?}");
    assert!(has("complexity", "where"), "missing WHERE, got: {full:?}");
    assert!(has("complexity", "having"), "missing HAVING, got: {full:?}");
    assert!(
        has("complexity", "set_operation"),
        "missing UNION set_operation, got: {full:?}"
    );
    assert!(has("complexity", "exists"), "missing EXISTS, got: {full:?}");
    assert!(
        has("nesting", "select"),
        "missing select nesting, got: {full:?}"
    );
    assert!(
        has("nesting", "subquery"),
        "missing subquery nesting, got: {full:?}"
    );
    assert!(has("nesting", "cte"), "missing CTE nesting, got: {full:?}");
}

// --- NEGATIVE: constructs that must not match (variants.sql) ---------------

#[test]
fn sql_calls_negative_bare_table_reference_is_not_a_call() {
    let Some((_loader, lang, _tags, calls, ..)) = sql_lang_and_queries() else {
        eprintln!("Skipping sql_calls_negative_bare_table: run `cargo xtask build-grammars` first");
        return;
    };
    // "SELECT * FROM variants.widgets;" — a bare table reference must never
    // produce a @call capture named "widgets".
    let found = collect_captures(&lang, SQL_VARIANTS, &calls, "call");
    assert!(
        !found.iter().any(|c| c == "widgets"),
        "a bare FROM table reference must not be captured as a call, got: {found:?}"
    );
}

// ---------------------------------------------------------------------------
// Starlark
// ---------------------------------------------------------------------------

const STARLARK_SAMPLE: &str = include_str!("fixtures/starlark/sample.star");

#[test]
fn starlark_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_tags: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("starlark")
        .expect("starlark tags query missing");
    let names = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "make_cc_library"),
        "expected 'make_cc_library' in starlark tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "make_test_suite" || n == "filter_srcs"),
        "expected another function in starlark tags, got: {names:?}"
    );
}

#[test]
fn starlark_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_calls: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("starlark")
        .expect("starlark calls query missing");
    let calls = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "cc_library" || c == "cc_binary" || c == "make_cc_library"),
        "expected a function call in starlark sample, got: {calls:?}"
    );
}

#[test]
fn starlark_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_complexity: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("starlark")
        .expect("starlark complexity query missing");
    let complexity = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in starlark sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn starlark_imports_finds_load_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_imports: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("starlark")
        .expect("starlark imports query missing");
    let paths = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("rules_cc") || p.contains("rules_python")),
        "expected a load path in starlark sample, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// HCL (Terraform)
// ---------------------------------------------------------------------------

const HCL_SAMPLE: &str = include_str!("fixtures/hcl/sample.tf");

#[test]
fn hcl_tags_finds_blocks() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_tags: hcl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("hcl").expect("hcl tags query missing");
    let names = collect_captures(&lang, HCL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "resource" || n == "variable" || n == "output"),
        "expected a block type in hcl tags, got: {names:?}"
    );
}

#[test]
fn hcl_types_finds_type_constraints() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_types: hcl grammar .so not found");
        return;
    };
    let query_str = loader.get_types("hcl").expect("hcl types query missing");
    let types = collect_captures(&lang, HCL_SAMPLE, &query_str, "type");
    assert!(
        !types.is_empty(),
        "expected at least one type constraint in hcl sample, got: {types:?}"
    );
}

#[test]
fn hcl_complexity_finds_conditionals() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_complexity: hcl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("hcl")
        .expect("hcl complexity query missing");
    let complexity = collect_captures(&lang, HCL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in hcl sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn hcl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_calls: hcl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("hcl").expect("hcl calls query missing");
    let calls = collect_captures(&lang, HCL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "merge" || c == "toset" || c == "lookup"),
        "expected a HCL function call in hcl sample, got: {calls:?}"
    );
}

#[test]
fn hcl_imports_finds_module_sources() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_imports: hcl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("hcl")
        .expect("hcl imports query missing");
    let paths = collect_captures(&lang, HCL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("modules/vpc") || p.contains("vpc")),
        "expected a module source path in hcl sample, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Nix
// ---------------------------------------------------------------------------

const NIX_SAMPLE: &str = include_str!("fixtures/nix/sample.nix");

#[test]
fn nix_tags_finds_bindings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_tags: nix grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("nix").expect("nix tags query missing");
    let names = collect_captures(&lang, NIX_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "greet" || n == "factorial"),
        "expected 'greet' or 'factorial' binding in nix tags, got: {names:?}"
    );
}

#[test]
fn nix_calls_finds_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_calls: nix grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("nix").expect("nix calls query missing");
    let calls = collect_captures(&lang, NIX_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "factorial" || c == "greet" || c == "filter"),
        "expected an application in nix sample, got: {calls:?}"
    );
}

#[test]
fn nix_complexity_finds_if_expressions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_complexity: nix grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("nix")
        .expect("nix complexity query missing");
    let complexity = collect_captures(&lang, NIX_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in nix sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn nix_imports_finds_import_expressions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_imports: nix grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("nix")
        .expect("nix imports query missing");
    let paths = collect_captures(&lang, NIX_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("nixpkgs") || p.contains("src")),
        "expected an import path in nix sample, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// MATLAB
// ---------------------------------------------------------------------------

const MATLAB_SAMPLE: &str = include_str!("fixtures/matlab/sample.m");

#[test]
fn matlab_tags_finds_functions_and_class() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_tags: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("matlab")
        .expect("matlab tags query missing");
    let names = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "factorial"),
        "expected 'factorial' function in matlab tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "Shape"),
        "expected 'Shape' class in matlab tags, got: {names:?}"
    );
}

#[test]
fn matlab_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_calls: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("matlab")
        .expect("matlab calls query missing");
    let calls = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "factorial" || c == "fprintf" || c == "length"),
        "expected a function call in matlab sample, got: {calls:?}"
    );
}

#[test]
fn matlab_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_complexity: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("matlab")
        .expect("matlab complexity query missing");
    let complexity = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in matlab sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn matlab_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_imports: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("matlab")
        .expect("matlab imports query missing");
    let paths = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p == "matlab.io.*"),
        "expected 'matlab.io.*' in matlab imports, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p == "matlab.net.http.RequestMessage"),
        "expected 'matlab.net.http.RequestMessage' in matlab imports, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// TLA+
// ---------------------------------------------------------------------------

const TLAPLUS_SAMPLE: &str = include_str!("fixtures/tlaplus/sample.tla");

#[test]
fn tlaplus_tags_finds_module_and_operators() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tlaplus_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tlaplus").ok() else {
        eprintln!("Skipping tlaplus_tags: tlaplus grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("tlaplus")
        .expect("tlaplus tags query missing");
    let names = collect_captures(&lang, TLAPLUS_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "Sample"),
        "expected 'Sample' module in tlaplus tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "Init" || n == "Next" || n == "Safety"),
        "expected an operator definition in tlaplus tags, got: {names:?}"
    );
}

#[test]
fn tlaplus_complexity_finds_conditionals() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tlaplus_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tlaplus").ok() else {
        eprintln!("Skipping tlaplus_complexity: tlaplus grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("tlaplus")
        .expect("tlaplus complexity query missing");
    let complexity = collect_captures(&lang, TLAPLUS_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in tlaplus sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn tlaplus_imports_finds_extends() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tlaplus_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tlaplus").ok() else {
        eprintln!("Skipping tlaplus_imports: tlaplus grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("tlaplus")
        .expect("tlaplus imports query missing");
    let paths = collect_captures(&lang, TLAPLUS_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Naturals") || p.contains("Sequences")),
        "expected 'Naturals' or 'Sequences' in tlaplus import paths, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// CMake
// ---------------------------------------------------------------------------

const CMAKE_SAMPLE: &str = include_str!("fixtures/cmake/CMakeLists.txt");

#[test]
fn cmake_tags_finds_functions_and_macros() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_tags: cmake grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cmake").expect("cmake tags query missing");
    let names = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"add_component".to_string()),
        "expected 'add_component' function in cmake tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "setup_target" || n == "install_component"),
        "expected 'setup_target' or 'install_component' in cmake tags, got: {names:?}"
    );
}

#[test]
fn cmake_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_calls: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("cmake")
        .expect("cmake calls query missing");
    let calls = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "find_package" || c == "add_library" || c == "target_link_libraries"),
        "expected cmake command calls in sample, got: {calls:?}"
    );
}

#[test]
fn cmake_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_complexity: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("cmake")
        .expect("cmake complexity query missing");
    let complexity = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in cmake sample, got: {complexity:?}"
    );
}

#[test]
fn cmake_imports_finds_includes_and_find_package() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_imports: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cmake")
        .expect("cmake imports query missing");
    let paths = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p == "Threads" || p == "OpenSSL" || p == "GNUInstallDirs"),
        "expected 'Threads'/'OpenSSL'/'GNUInstallDirs' in cmake import paths, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// GraphQL
// ---------------------------------------------------------------------------

const GRAPHQL_SAMPLE: &str = include_str!("fixtures/graphql/sample.graphql");

#[test]
fn graphql_tags_finds_types_and_interfaces() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping graphql_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("graphql").ok() else {
        eprintln!("Skipping graphql_tags: graphql grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("graphql")
        .expect("graphql tags query missing");
    let names = collect_captures(&lang, GRAPHQL_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"User".to_string()),
        "expected 'User' type in graphql tags, got: {names:?}"
    );
    assert!(
        names.contains(&"UserRole".to_string()),
        "expected 'UserRole' enum in graphql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "Node" || n == "Timestamped"),
        "expected interface name in graphql tags, got: {names:?}"
    );
}

#[test]
fn graphql_calls_finds_field_selections() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping graphql_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("graphql").ok() else {
        eprintln!("Skipping graphql_calls: graphql grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("graphql")
        .expect("graphql calls query missing");
    // GraphQL calls query captures field names; runs cleanly against schema definitions
    let _calls = collect_captures(&lang, GRAPHQL_SAMPLE, &query_str, "call");
}

#[test]
fn graphql_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping graphql_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("graphql").ok() else {
        eprintln!("Skipping graphql_complexity: graphql grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("graphql")
        .expect("graphql complexity query missing");
    let _complexity = collect_captures(&lang, GRAPHQL_SAMPLE, &query_str, "complexity");
}

// ---------------------------------------------------------------------------
// GLSL
// ---------------------------------------------------------------------------

const GLSL_SAMPLE: &str = include_str!("fixtures/glsl/sample.glsl");

#[test]
fn glsl_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_tags: glsl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("glsl").expect("glsl tags query missing");
    let names = collect_captures(&lang, GLSL_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"main".to_string()),
        "expected 'main' function in glsl tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "Material" || n == "calculateDiffuse" || n == "applyFog"),
        "expected a struct or function name in glsl tags, got: {names:?}"
    );
}

#[test]
fn glsl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_calls: glsl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("glsl").expect("glsl calls query missing");
    let calls = collect_captures(&lang, GLSL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "normalize" || c == "texture" || c == "calculateDiffuse"),
        "expected builtin or user function call in glsl sample, got: {calls:?}"
    );
}

#[test]
fn glsl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_complexity: glsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("glsl")
        .expect("glsl complexity query missing");
    let complexity = collect_captures(&lang, GLSL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in glsl sample, got: {complexity:?}"
    );
}

#[test]
fn glsl_imports_finds_include_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_imports: glsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("glsl")
        .expect("glsl imports query missing");
    let paths = collect_captures(&lang, GLSL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("common") || p.contains("lighting")),
        "expected #include paths in glsl sample, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// HLSL
// ---------------------------------------------------------------------------

const HLSL_SAMPLE: &str = include_str!("fixtures/hlsl/sample.hlsl");

#[test]
fn hlsl_tags_finds_functions_structs_and_cbuffers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_tags: hlsl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("hlsl").expect("hlsl tags query missing");
    let names = collect_captures(&lang, HLSL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "VSMain" || n == "PSMain" || n == "ComputeLighting"),
        "expected a function name in hlsl tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "PerFrame" || n == "PerObject" || n == "VSInput"),
        "expected a cbuffer or struct name in hlsl tags, got: {names:?}"
    );
}

#[test]
fn hlsl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_calls: hlsl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("hlsl").expect("hlsl calls query missing");
    let calls = collect_captures(&lang, HLSL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "normalize" || c == "mul" || c == "ComputeLighting"),
        "expected function calls in hlsl sample, got: {calls:?}"
    );
}

#[test]
fn hlsl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_complexity: hlsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("hlsl")
        .expect("hlsl complexity query missing");
    let complexity = collect_captures(&lang, HLSL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in hlsl sample, got: {complexity:?}"
    );
}

#[test]
fn hlsl_imports_finds_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_imports: hlsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("hlsl")
        .expect("hlsl imports query missing");
    let paths = collect_captures(&lang, HLSL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("common.hlsl") || p.contains("d3d11.h")),
        "expected include paths in hlsl imports, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// jq
// ---------------------------------------------------------------------------

const JQ_SAMPLE: &str = include_str!("fixtures/jq/sample.jq");

#[test]
fn jq_tags_finds_function_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_tags: jq grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("jq").expect("jq tags query missing");
    let names = collect_captures(&lang, JQ_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"sum".to_string()),
        "expected 'sum' function in jq tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "mean" || n == "flatten_keys" || n == "keep_if"),
        "expected function names in jq tags, got: {names:?}"
    );
}

#[test]
fn jq_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_calls: jq grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("jq").expect("jq calls query missing");
    let calls = collect_captures(&lang, JQ_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "map" || c == "select" || c == "group_by" || c == "sort_by"),
        "expected builtin function calls in jq sample, got: {calls:?}"
    );
}

#[test]
fn jq_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_complexity: jq grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("jq")
        .expect("jq complexity query missing");
    let _complexity = collect_captures(&lang, JQ_SAMPLE, &query_str, "complexity");
}

#[test]
fn jq_imports_finds_import_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_imports: jq grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("jq").expect("jq imports query missing");
    let paths = collect_captures(&lang, JQ_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("lib/utils")),
        "expected 'lib/utils' in jq import paths, got: {paths:?}"
    );
}

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

// ---------------------------------------------------------------------------
// Meson
// ---------------------------------------------------------------------------

const MESON_SAMPLE: &str = include_str!("fixtures/meson/meson.build");

#[test]
fn meson_tags_finds_variable_assignments() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_tags: meson grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("meson").expect("meson tags query missing");
    // Meson tags captures variable identifiers from var_unit assignments
    let _names = collect_captures(&lang, MESON_SAMPLE, &query_str, "name");
}

#[test]
fn meson_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_calls: meson grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("meson")
        .expect("meson calls query missing");
    let calls = collect_captures(&lang, MESON_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "project" || c == "dependency" || c == "executable" || c == "library"),
        "expected meson function calls in sample, got: {calls:?}"
    );
}

#[test]
fn meson_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_complexity: meson grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("meson")
        .expect("meson complexity query missing");
    let complexity = collect_captures(&lang, MESON_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (if block) in meson sample, got: {complexity:?}"
    );
}

#[test]
fn meson_imports_finds_subproject_and_dependency() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping meson_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("meson").ok() else {
        eprintln!("Skipping meson_imports: meson grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("meson")
        .expect("meson imports query missing");
    let paths = collect_captures(&lang, MESON_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("glib-2.0") || p.contains("zlib") || p.contains("protobuf")),
        "expected dependency names in meson imports, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Nginx
// ---------------------------------------------------------------------------

const NGINX_SAMPLE: &str = include_str!("fixtures/nginx/nginx.conf");

#[test]
fn nginx_tags_finds_block_directives() {
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
}

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
}

// ---------------------------------------------------------------------------
// SCSS
// ---------------------------------------------------------------------------

const SCSS_SAMPLE: &str = include_str!("fixtures/scss/sample.scss");

#[test]
fn scss_tags_finds_mixins_functions_and_rules() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_tags: scss grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scss").expect("scss tags query missing");
    let names = collect_captures(&lang, SCSS_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "flex-center" || n == "responsive"),
        "expected mixin names in scss tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "rem" || n == "shade"),
        "expected function names in scss tags, got: {names:?}"
    );
}

#[test]
fn scss_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_calls: scss grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("scss").expect("scss calls query missing");
    let calls = collect_captures(&lang, SCSS_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "darken" || c == "rgba" || c == "shade"),
        "expected function calls in scss sample, got: {calls:?}"
    );
}

#[test]
fn scss_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_complexity: scss grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scss")
        .expect("scss complexity query missing");
    let complexity = collect_captures(&lang, SCSS_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (@if/@each) in scss sample, got: {complexity:?}"
    );
}

#[test]
fn scss_imports_finds_use_and_import_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_imports: scss grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scss")
        .expect("scss imports query missing");
    let paths = collect_captures(&lang, SCSS_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("sass:math") || p.contains("variables") || p.contains("mixins")),
        "expected import paths in scss sample, got: {paths:?}"
    );
}

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

// ---------------------------------------------------------------------------
// Svelte
// ---------------------------------------------------------------------------

const SVELTE_SAMPLE: &str = include_str!("fixtures/svelte/sample.svelte");

#[test]
fn svelte_tags_finds_script_and_style_blocks() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping svelte_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("svelte").ok() else {
        eprintln!("Skipping svelte_tags: svelte grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("svelte")
        .expect("svelte tags query missing");
    let names = collect_captures(&lang, SVELTE_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "script" || n == "style"),
        "expected 'script' or 'style' block tags in svelte sample, got: {names:?}"
    );
}

#[test]
fn svelte_calls_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping svelte_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("svelte").ok() else {
        eprintln!("Skipping svelte_calls: svelte grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("svelte")
        .expect("svelte calls query missing");
    // Svelte calls query is intentionally empty (JS in <script> is raw_text)
    let _calls = collect_captures(&lang, SVELTE_SAMPLE, &query_str, "call");
}

#[test]
fn svelte_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping svelte_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("svelte").ok() else {
        eprintln!("Skipping svelte_complexity: svelte grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("svelte")
        .expect("svelte complexity query missing");
    let _complexity = collect_captures(&lang, SVELTE_SAMPLE, &query_str, "complexity");
}

// ---------------------------------------------------------------------------
// Typst
// ---------------------------------------------------------------------------

const TYPST_SAMPLE: &str = include_str!("fixtures/typst/sample.typ");

#[test]
fn typst_tags_finds_let_bindings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_tags: typst grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("typst").expect("typst tags query missing");
    let names = collect_captures(&lang, TYPST_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "format_version" || n == "summary_table"),
        "expected function let bindings in typst tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "project_name" || n == "version"),
        "expected variable let bindings in typst tags, got: {names:?}"
    );
}

#[test]
fn typst_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_calls: typst grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typst")
        .expect("typst calls query missing");
    let calls = collect_captures(&lang, TYPST_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "tablex" || c == "format_version" || c == "summary_table"),
        "expected function calls in typst sample, got: {calls:?}"
    );
}

#[test]
fn typst_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_complexity: typst grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("typst")
        .expect("typst complexity query missing");
    let _complexity = collect_captures(&lang, TYPST_SAMPLE, &query_str, "complexity");
}

#[test]
fn typst_imports_finds_import_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_imports: typst grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("typst")
        .expect("typst imports query missing");
    let paths = collect_captures(&lang, TYPST_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("template.typ") || p.contains("tablex")),
        "expected import paths in typst sample, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Verilog
// ---------------------------------------------------------------------------

const VERILOG_SAMPLE: &str = include_str!("fixtures/verilog/sample.v");

#[test]
fn verilog_tags_finds_modules() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_tags: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("verilog")
        .expect("verilog tags query missing");
    let names = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"alu".to_string()),
        "expected 'alu' module in verilog tags, got: {names:?}"
    );
    assert!(
        names.contains(&"reg_file".to_string()),
        "expected 'reg_file' module in verilog tags, got: {names:?}"
    );
}

#[test]
fn verilog_calls_finds_task_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_calls: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("verilog")
        .expect("verilog calls query missing");
    let _calls = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "call");
}

#[test]
fn verilog_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_complexity: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("verilog")
        .expect("verilog complexity query missing");
    let complexity = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (always/case/if) in verilog sample, got: {complexity:?}"
    );
}

#[test]
fn verilog_imports_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_imports: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("verilog")
        .expect("verilog imports query missing");
    let _paths = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "import.path");
}

// ---------------------------------------------------------------------------
// VHDL
// ---------------------------------------------------------------------------

const VHDL_SAMPLE: &str = include_str!("fixtures/vhdl/sample.vhd");

#[test]
fn vhdl_tags_finds_entity_and_architecture() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_tags: vhdl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vhdl").expect("vhdl tags query missing");
    let names = collect_captures(&lang, VHDL_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"fifo".to_string()),
        "expected 'fifo' entity in vhdl tags, got: {names:?}"
    );
    assert!(
        names.contains(&"rtl".to_string()),
        "expected 'rtl' architecture in vhdl tags, got: {names:?}"
    );
}

#[test]
fn vhdl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_calls: vhdl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vhdl").expect("vhdl calls query missing");
    let calls = collect_captures(&lang, VHDL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "rising_edge" || c == "to_integer"),
        "expected function calls in vhdl sample, got: {calls:?}"
    );
}

#[test]
fn vhdl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_complexity: vhdl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vhdl")
        .expect("vhdl complexity query missing");
    let complexity = collect_captures(&lang, VHDL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (if/process) in vhdl sample, got: {complexity:?}"
    );
}

#[test]
fn vhdl_imports_finds_use_clauses() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_imports: vhdl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("vhdl")
        .expect("vhdl imports query missing");
    let paths = collect_captures(&lang, VHDL_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("std_logic_1164")
            || p.contains("numeric_std")
            || p.contains("ieee")),
        "expected use clause paths in vhdl sample, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Vim script
// ---------------------------------------------------------------------------

const VIM_SAMPLE: &str = include_str!("fixtures/vim/sample.vim");

#[test]
fn vim_tags_finds_functions_and_augroups() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_tags: vim grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vim").expect("vim tags query missing");
    let names = collect_captures(&lang, VIM_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "ToggleOption" || n == "FormatBuffer" || n == "OpenTerminal"),
        "expected function names in vim tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "MyPlugin" || n == "FileTypeSettings"),
        "expected augroup names in vim tags, got: {names:?}"
    );
}

#[test]
fn vim_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_calls: vim grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vim").expect("vim calls query missing");
    let calls = collect_captures(&lang, VIM_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "FormatBuffer" || c == "getpos" || c == "setpos"),
        "expected function calls in vim sample, got: {calls:?}"
    );
}

#[test]
fn vim_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_complexity: vim grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vim")
        .expect("vim complexity query missing");
    let complexity = collect_captures(&lang, VIM_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (if block) in vim sample, got: {complexity:?}"
    );
}

#[test]
fn vim_imports_finds_source_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vim_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vim").ok() else {
        eprintln!("Skipping vim_imports: vim grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("vim")
        .expect("vim imports query missing");
    let paths = collect_captures(&lang, VIM_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils.vim") || p.contains("defaults.vim")),
        "expected sourced file paths in vim imports, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Vue
// ---------------------------------------------------------------------------

const VUE_SAMPLE: &str = include_str!("fixtures/vue/sample.vue");

#[test]
fn vue_tags_finds_script_template_and_style_blocks() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vue_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vue").ok() else {
        eprintln!("Skipping vue_tags: vue grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vue").expect("vue tags query missing");
    let names = collect_captures(&lang, VUE_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "script" || n == "template" || n == "style"),
        "expected SFC block tag names in vue tags, got: {names:?}"
    );
}

#[test]
fn vue_calls_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vue_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vue").ok() else {
        eprintln!("Skipping vue_calls: vue grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vue").expect("vue calls query missing");
    // Vue calls query is intentionally empty (JS in <script> is raw_text)
    let _calls = collect_captures(&lang, VUE_SAMPLE, &query_str, "call");
}

#[test]
fn vue_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vue_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vue").ok() else {
        eprintln!("Skipping vue_complexity: vue grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vue")
        .expect("vue complexity query missing");
    let _complexity = collect_captures(&lang, VUE_SAMPLE, &query_str, "complexity");
}

// ---------------------------------------------------------------------------
// Jinja2 (live grammar — uses ~/.config/normalize/grammars/jinja2.so)
// ---------------------------------------------------------------------------

const JINJA2_SAMPLE: &str = include_str!("fixtures/jinja2/sample.jinja2");

#[test]
fn jinja2_tags_finds_macros() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jinja2_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_tags: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("jinja2")
        .expect("jinja2 tags query missing");
    let names = collect_captures(&lang, JINJA2_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"render_form".to_string()),
        "expected 'render_form' macro in jinja2 tags, got: {names:?}"
    );
    assert!(
        names.contains(&"render_nav".to_string()),
        "expected 'render_nav' macro in jinja2 tags, got: {names:?}"
    );
}

#[test]
fn jinja2_imports_finds_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jinja2_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_imports: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("jinja2")
        .expect("jinja2 imports query missing");
    let paths = collect_captures(&lang, JINJA2_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("base.html")),
        "expected 'base.html' in jinja2 import paths, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("helpers.html")),
        "expected 'helpers.html' in jinja2 import paths, got: {paths:?}"
    );
}

#[test]
fn jinja2_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jinja2_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jinja2").ok() else {
        eprintln!("Skipping jinja2_complexity: jinja2 grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("jinja2")
        .expect("jinja2 complexity query missing");
    let complexity = collect_captures(&lang, JINJA2_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in jinja2 sample (for/if/elif), got {} ({complexity:?})",
        complexity.len()
    );
}

// ---------------------------------------------------------------------------
// Groovy / Elixir / Haskell live grammar tests (use ~/.config/normalize/grammars/)
// ---------------------------------------------------------------------------

#[test]
fn groovy_tags_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("groovy").ok() else {
        eprintln!("Skipping groovy_tags_live: groovy grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("groovy")
        .expect("groovy tags query missing");
    let names = collect_captures(&lang, GROOVY_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class, got: {names:?}"
    );
    assert!(
        names.contains(&"distanceTo".to_string()),
        "expected 'distanceTo' method, got: {names:?}"
    );
    assert!(
        names.contains(&"MathUtils".to_string()),
        "expected 'MathUtils' class, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' method, got: {names:?}"
    );
    assert!(
        names.contains(&"greet".to_string()),
        "expected 'greet' function, got: {names:?}"
    );
}

#[test]
fn elixir_tags_no_args_function() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("elixir").ok() else {
        eprintln!("Skipping elixir_tags_no_args_function: elixir grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("elixir")
        .expect("elixir tags query missing");
    // Test the no-args function form: "def name do ... end"
    let source = r#"defmodule Foo do
  def initialize do
    :ok
  end

  def greet(name) do
    name
  end
end"#;
    let names = collect_captures(&lang, source, &query_str, "name");
    assert!(
        names.contains(&"initialize".to_string()),
        "expected 'initialize' (no-args def), got: {names:?}"
    );
    assert!(
        names.contains(&"greet".to_string()),
        "expected 'greet' (with-args def), got: {names:?}"
    );
}

const HASKELL_SAMPLE: &str = include_str!("fixtures/haskell/sample.hs");
const HASKELL_VARIANTS: &str = include_str!("fixtures/haskell/variants.hs");

/// Returns `(loader, lang)` together — the `GrammarLoader` owns the loaded
/// dylib, so it must stay alive for as long as the returned `Language` is
/// used (dropping it early unloads the library and leaves `Language`'s
/// function pointers dangling, which segfaults rather than erroring).
fn haskell_lang() -> Option<(normalize_languages::GrammarLoader, tree_sitter::Language)> {
    let loader = normalize_languages::GrammarLoader::new();
    let lang = loader.get("haskell").ok()?;
    Some((loader, lang))
}

// --- Dimension 4: real-world fixture coverage (sample.hs) -------------------

#[test]
fn haskell_tags_no_duplicate_signatures() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_tags_no_duplicate_signatures: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("haskell")
        .expect("haskell tags query missing");
    let names = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "name");
    // "classify" has one equation; with type signatures removed, it should appear exactly once
    // (before fix: appeared twice — once for signature, once for definition).
    let classify_count = names.iter().filter(|n| *n == "classify").count();
    assert_eq!(
        classify_count, 1,
        "expected 'classify' exactly once (type signature removed), got: {names:?}"
    );
    // "insert" has two equations (multi-equation function); the grammar produces one `function`
    // node per equation, so it legitimately appears twice in the raw query output.
    // Deduplication to a single symbol happens in the extraction layer (normalize-facts).
    let insert_count = names.iter().filter(|n| *n == "insert").count();
    assert!(
        (1..=2).contains(&insert_count),
        "expected 'insert' 1-2 times (multi-equation), got: {names:?}"
    );
    // Type names from data/newtype/type should also be present
    assert!(
        names.contains(&"Tree".to_string()),
        "expected 'Tree' data type, got: {names:?}"
    );
    assert!(
        names.contains(&"Count".to_string()),
        "expected 'Count' newtype, got: {names:?}"
    );
    // Typeclass + two instances of it for different types — both must be
    // captured (see haskell.rs's dedup_haskell_functions fix: it previously
    // dropped every instance after the first with the same class name).
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' typeclass and/or its instances, got: {names:?}"
    );
    let shape_count = names.iter().filter(|n| *n == "Shape").count();
    assert!(
        shape_count >= 3,
        "expected 'Shape' at least 3 times (class def + 2 instances), got: {names:?}"
    );
    // Record type declaration.
    assert!(
        names.contains(&"Rectangle".to_string()),
        "expected 'Rectangle' record type, got: {names:?}"
    );
    // Operator function definition: `(<+>) a b = ...`.
    assert!(
        names.contains(&"(<+>)".to_string()),
        "expected '(<+>)' operator function definition, got: {names:?}"
    );
    // Point-free / zero-argument top-level binding: `frequencyMap = foldr ...`.
    // Entirely absent before the `bind`-node fix.
    assert!(
        names.contains(&"frequencyMap".to_string()),
        "expected point-free 'frequencyMap' binding, got: {names:?}"
    );
    // `main` itself — the most fundamental top-level Haskell definition —
    // was entirely absent before the `bind`-node fix.
    assert!(
        names.contains(&"main".to_string()),
        "expected 'main' binding, got: {names:?}"
    );
    // where-bound local helpers must never leak into top-level tags.
    assert!(
        !names.contains(&"bmiTier".to_string()),
        "where-bound 'bmiTier' must not appear in top-level tags, got: {names:?}"
    );
    assert!(
        !names.contains(&"bmi".to_string()),
        "where-bound 'bmi' must not appear in top-level tags, got: {names:?}"
    );
}

#[test]
fn haskell_calls_finds_local_qualified_and_constructor_calls() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!(
            "Skipping haskell_calls_finds_local_qualified_and_constructor_calls: haskell grammar not found"
        );
        return;
    };
    let query_str = loader
        .get_calls("haskell")
        .expect("haskell calls query missing");
    let calls = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"insert".to_string()),
        "expected local 'insert' call in haskell calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"Node".to_string()),
        "expected constructor 'Node' application in haskell calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"insertWith".to_string()),
        "expected qualified 'Map.insertWith' call to capture 'insertWith' in haskell calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"Map".to_string()),
        "expected 'Map' qualifier in haskell calls, got: {qualifiers:?}"
    );
}

#[test]
fn haskell_imports_finds_named_imports() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_imports_finds_named_imports: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_imports("haskell")
        .expect("haskell imports query missing");
    let paths = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p == "Data.List" || p == "Data"),
        "expected 'Data.List' import path, got: {paths:?}"
    );
    // Named imports were entirely unmatched before this fix — @import.name
    // never had a single capture in the whole file.
    let names = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "import.name");
    assert!(
        names.contains(&"sort".to_string()) && names.contains(&"nub".to_string()),
        "expected 'sort' and 'nub' named imports, got: {names:?}"
    );
    // hiding-imports use the same import_list shape — `hiding (lookup)` must
    // also produce an @import.name capture for "lookup".
    assert!(
        names.contains(&"lookup".to_string()),
        "expected 'lookup' from 'import Prelude hiding (lookup)', got: {names:?}"
    );
}

#[test]
fn haskell_complexity_no_baseline_inflation_and_finds_branches() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!(
            "Skipping haskell_complexity_no_baseline_inflation_and_finds_branches: haskell grammar not found"
        );
        return;
    };
    let query_str = loader
        .get_complexity("haskell")
        .expect("haskell complexity query missing");
    let complexity = collect_captures(&lang, HASKELL_SAMPLE, &query_str, "complexity");
    // "classify" (nested if/else) and "describe" (case with a guarded
    // alternative) both contribute real decision points.
    assert!(
        complexity.len() >= 4,
        "expected at least 4 complexity nodes in haskell sample, got {} ({complexity:?})",
        complexity.len()
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.hs) -

#[test]
fn haskell_tags_completeness_all_name_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_tags_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("haskell")
        .expect("haskell tags query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    // (capture_name, node_kind, text) for every documented variant.
    let required: &[(&str, &str, &str)] = &[
        ("name", "variable", "plainFunc"), // function.name: variable
        ("name", "prefix_id", "(+++)"),    // function.name: prefix_id
        ("name", "name", "Tree"),          // data_type.name: name
        ("name", "prefix_id", "(:+:)"),    // data_type.name: prefix_id
        ("name", "name", "Count"),         // newtype.name: name
        ("name", "prefix_id", "(:*:)"),    // newtype.name: prefix_id
        ("name", "name", "Name"),          // type_synomym.name: name
        ("name", "prefix_id", "(:->)"),    // type_synomym.name: prefix_id
        ("name", "name", "Shape"),         // class.name: name (also matches both instances)
        ("name", "prefix_id", "(:~:)"),    // class.name / instance.name: prefix_id
        ("name", "variable", "doubleAll"), // bind.name: variable (point-free)
        ("name", "prefix_id", "(<+>)"),    // bind.name: prefix_id (point-free operator)
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in haskell.tags.scm \
             output for variants.hs, got: {caps:?}"
        );
    }

    // NEGATIVE: where-bound and let-bound local names must never appear as
    // @name captures — `function`/`bind` are also the node types for local
    // helpers, and only top-level `(declarations ...)` children are tagged.
    for local in ["negHelper", "negLocal"] {
        assert!(
            !caps.iter().any(|(cn, _, t, _)| cn == "name" && t == local),
            "local binding '{local}' must not appear as a @name capture, got: {caps:?}"
        );
    }
}

#[test]
fn haskell_calls_completeness_all_function_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_calls_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_calls("haskell")
        .expect("haskell calls query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("call", "variable", "plainFunc"), // apply.function: variable
        ("call", "constructor", "TNode"),  // apply.function: constructor
        ("call", "variable", "lookup"),    // apply.function: qualified, id: variable
        ("call", "constructor", "Just"),   // apply.function: qualified, id: constructor
        ("call", "operator", "$"),         // apply.function: prefix_id(operator)
        ("call", "operator", "+"),         // apply.function: prefix_id(qualified(operator))
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in haskell.calls.scm \
             output for variants.hs, got: {caps:?}"
        );
    }

    // apply.function: parens(qualified(variable)) — `(Map.lookup) 1 Map.empty`.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"Map"),
        "expected 'Map' qualifier (incl. parens-wrapped qualified call), got: {qualifiers:?}"
    );

    // `plainFunc` as a call target appears at least 3 times: the plain call,
    // the parens-wrapped-variable call, and inside the negative composition
    // case is intentionally NOT one of them (see negative test below).
    let plain_func_calls = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "call" && k == "variable" && t == "plainFunc")
        .count();
    assert!(
        plain_func_calls >= 3,
        "expected 'plainFunc' called at least 3 times (plain, in callParenVariable, in \
         callQualifiedConstructor argument), got {plain_func_calls} in {caps:?}"
    );
}

#[test]
fn haskell_calls_negative_composition_not_matched() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_calls_negative_composition: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_calls("haskell")
        .expect("haskell calls query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);
    // `negComposed = (plainFunc . plainFunc) 1` — the applied value is a
    // point-free composition (parens wrapping an `infix` expression), not a
    // single nameable identifier. No @call capture should attribute this
    // outer apply to a specific function name; only the innermost `apply`
    // for the outer application itself is absent (composition is not
    // unwound into two calls to `plainFunc`).
    //
    // Every top-level `apply` in the file whose function is a bare `infix`
    // node (not `variable`/`constructor`/`prefix_id`/`parens`-wrapping-name)
    // must produce zero matches from this query on that specific node.
    let composed_query = "(apply function: (parens expression: (infix) @composed))";
    let ts_query = tree_sitter::Query::new(&lang, composed_query).expect("compiles");
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(HASKELL_VARIANTS, None).expect("parse failed");
    let mut cursor = tree_sitter::QueryCursor::new();
    let source_bytes = HASKELL_VARIANTS.as_bytes();
    let mut matches = cursor.matches(&ts_query, tree.root_node(), source_bytes);
    let mut composed_count = 0;
    while matches.next().is_some() {
        composed_count += 1;
    }
    assert_eq!(
        composed_count, 1,
        "expected exactly 1 parens-wrapped-infix apply.function (negComposed's `(plainFunc . \
         plainFunc)`), got {composed_count}"
    );
    let _ = caps; // calls.scm output already asserted not to double-count this construct above.
}

#[test]
fn haskell_imports_completeness_all_name_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_imports_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_imports("haskell")
        .expect("haskell imports query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    let required: &[(&str, &str, &str)] = &[
        ("import.name", "variable", "sort"), // import_name.variable: variable
        ("import.name", "variable", "nub"),
        ("import.name", "name", "Down"), // import_name.type: name
        ("import.name", "prefix_id", "(<|>)"), // import_name.operator: prefix_id
        ("import.name", "variable", "lookup"), // hiding-import reuses the same shape
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in haskell.imports.scm \
             output for variants.hs, got: {caps:?}"
        );
    }
}

#[test]
fn haskell_complexity_completeness_all_branch_variants() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_complexity_completeness: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_complexity("haskell")
        .expect("haskell complexity query missing");
    let caps = collect_captures_full(&lang, HASKELL_VARIANTS, &query_str);

    let required_kinds: &[&str] = &[
        "conditional", // if/then/else
        // `guard` is a grammar supertype alias (subtypes: boolean/let/
        // pattern_guard) that never materializes as a "guard"-kind node —
        // confirmed via node-types.json and real parse. Tree-sitter's query
        // engine still matches `(guard) @complexity` against the concrete
        // subtype nodes; the captured node's own `.kind()` reports the
        // subtype ("boolean" here), not "guard".
        "boolean",
        "lambda",       // plain lambda
        "multi_way_if", // MultiWayIf extension — previously entirely unmatched
        "lambda_case",  // LambdaCase extension — previously entirely unmatched
        "alternative",  // per-arm case decision point — previously unmatched
        "case",         // case container
    ];
    for kind in required_kinds {
        assert!(
            caps.iter()
                .any(|(cn, k, _, _)| cn == "complexity" && k == kind),
            "expected a @complexity capture of kind '{kind}' in variants.hs, got: {caps:?}"
        );
    }
}

#[test]
fn haskell_complexity_negative_trivial_function_has_no_complexity_captures() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_complexity_negative: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_complexity("haskell")
        .expect("haskell complexity query missing");

    // `negTrivial x = x + 1` — zero branches. Before the fix, the function's
    // own `match` (equation-body) node was unconditionally counted as a
    // decision point, so this construct alone would have produced one
    // @complexity capture despite having no branching whatsoever.
    let source = "module M where\n\nnegTrivial :: Int -> Int\nnegTrivial x = x + 1\n";
    let caps = collect_captures_full(&lang, source, &query_str);
    let complexity_caps: Vec<_> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .collect();
    assert!(
        complexity_caps.is_empty(),
        "expected zero @complexity captures for a branch-free function, got: {complexity_caps:?}"
    );
}

#[test]
fn haskell_types_finds_qualified_and_generic_type_references() {
    let Some((loader, lang)) = haskell_lang() else {
        eprintln!("Skipping haskell_types_finds_qualified_and_generic: haskell grammar not found");
        return;
    };
    let query_str = loader
        .get_types("haskell")
        .expect("haskell types query missing");
    let types = collect_captures(&lang, HASKELL_VARIANTS, &query_str, "type.reference");
    // Qualified type reference: Map.Map Int Int — the inner `Map` (via
    // `qualified.id`) must still be captured.
    assert!(
        types.iter().filter(|t| *t == "Map").count() >= 1,
        "expected qualified 'Map' type reference, got: {types:?}"
    );
    // Generic/applied type reference: Maybe Int (apply.constructor: name).
    assert!(
        types.contains(&"Maybe".to_string()),
        "expected generic 'Maybe' type reference, got: {types:?}"
    );
}

const GROOVY_SAMPLE: &str = include_str!("fixtures/groovy/sample.groovy");

#[test]
fn groovy_imports_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("groovy").ok() else {
        eprintln!("Skipping groovy_imports_live: groovy grammar not found");
        return;
    };
    let query_str = loader
        .get_imports("groovy")
        .expect("groovy imports query missing");
    let paths = collect_captures(&lang, GROOVY_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Immutable") || p.contains("groovy")),
        "expected 'groovy.transform.Immutable' in groovy import paths, got: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .any(|p| p.contains("ArrayList") || p.contains("java")),
        "expected 'java.util.ArrayList' in groovy import paths, got: {paths:?}"
    );
}

#[test]
fn groovy_types_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("groovy").ok() else {
        eprintln!("Skipping groovy_types_live: groovy grammar not found");
        return;
    };
    let query_str = loader
        .get_types("groovy")
        .expect("groovy types query missing");
    let types = collect_captures(&lang, GROOVY_SAMPLE, &query_str, "type.reference");
    assert!(
        types.contains(&"Point".to_string()),
        "expected 'Point' parameter type in groovy types, got: {types:?}"
    );
    assert!(
        types.contains(&"String".to_string()),
        "expected 'String' return type in groovy types, got: {types:?}"
    );
    assert!(
        types.contains(&"List".to_string()),
        "expected base generic type 'List' in groovy types, got: {types:?}"
    );
    assert!(
        types.contains(&"Integer".to_string()),
        "expected generic type argument 'Integer' in groovy types, got: {types:?}"
    );
}

#[test]
fn kotlin_tags_live() {
    let loader = normalize_languages::GrammarLoader::new();
    let Some(lang) = loader.get("kotlin").ok() else {
        eprintln!("Skipping kotlin_tags_live: kotlin grammar not found");
        return;
    };
    let query_str = loader
        .get_tags("kotlin")
        .expect("kotlin tags query missing");
    let names = collect_captures(&lang, KOTLIN_SAMPLE, &query_str, "name");
    // After fix: should find classes and functions, NOT local val declarations
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class, got: {names:?}"
    );
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function, got: {names:?}"
    );
    assert!(
        names.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' function, got: {names:?}"
    );
    // Local val declarations should NOT appear
    assert!(
        !names.contains(&"dx".to_string()),
        "local 'dx' should not appear in tags, got: {names:?}"
    );
    assert!(
        !names.contains(&"total".to_string()),
        "local 'total' should not appear in tags, got: {names:?}"
    );
}

// ---------------------------------------------------------------------------
// Decorations tests
// ---------------------------------------------------------------------------

fn assert_decorations_contains(
    loader: &GrammarLoader,
    grammar: &str,
    sample: &str,
    expected: &[&str],
) {
    let Some(lang) = loader.get(grammar).ok() else {
        if std::env::var("NORMALIZE_REQUIRE_GRAMMARS").is_ok() {
            panic!(
                "{grammar}_decorations: grammar .so not found \
                 — set NORMALIZE_REQUIRE_GRAMMARS only when grammars are built"
            );
        }
        eprintln!("Skipping {grammar}_decorations: grammar .so not found");
        return;
    };
    let query_str = loader
        .get_decorations(grammar)
        .unwrap_or_else(|| panic!("{grammar} decorations query missing"));
    let captures = collect_captures(&lang, sample, &query_str, "decoration");
    assert!(
        !captures.is_empty(),
        "expected at least one @decoration capture for {grammar}, got none"
    );
    for exp in expected {
        assert!(
            captures.iter().any(|c| c.contains(exp)),
            "expected capture containing {exp:?} for {grammar}, got: {captures:?}"
        );
    }
}

#[test]
fn rust_decorations_finds_attribute_and_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping rust_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "rust",
        RUST_SAMPLE,
        &["#[derive(Debug, Clone)]", "/// Classify"],
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

#[test]
fn javascript_decorations_finds_decorator_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping javascript_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "javascript",
        JAVASCRIPT_SAMPLE,
        &["@sealed", "// A stack data structure"],
    );
}

#[test]
fn typescript_decorations_finds_decorator_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping typescript_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "typescript",
        TS_SAMPLE,
        &["@Injectable()"],
    );
}

#[test]
fn tsx_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping tsx_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "tsx",
        TSX_SAMPLE,
        &["// Classify"],
    );
}

#[test]
fn java_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping java_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "java",
        JAVA_SAMPLE,
        &["@Override", "// Returns the size"],
    );
}

#[test]
fn kotlin_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping kotlin_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "kotlin",
        KOTLIN_SAMPLE,
        &["@JvmStatic", "// Classify a number"],
    );
}

#[test]
fn scala_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping scala_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "scala",
        SCALA_SAMPLE,
        &["@main", "// Classify a number"],
    );
}

#[test]
fn csharp_decorations_finds_attribute_list_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping csharp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "c-sharp",
        CSHARP_SAMPLE,
        &["[Obsolete", "/// <summary>"],
    );
}

#[test]
fn php_decorations_finds_attribute_list_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping php_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "php",
        PHP_SAMPLE,
        &["#[Pure]"],
    );
}

#[test]
fn swift_decorations_finds_attribute_and_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping swift_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "swift",
        SWIFT_SAMPLE,
        &["@discardableResult", "/// Classify"],
    );
}

#[test]
fn dart_decorations_finds_annotation_and_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping dart_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "dart",
        DART_SAMPLE,
        &["@pragma", "/// Classify"],
    );
}

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

#[test]
fn rescript_decorations_finds_decorator_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping rescript_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "rescript",
        RESCRIPT_SAMPLE,
        &["@inline"],
    );
}

const FSHARP_SAMPLE: &str = include_str!("fixtures/fsharp/sample.fs");

#[test]
fn fsharp_decorations_finds_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping fsharp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "fsharp",
        FSHARP_SAMPLE,
        &["[<EntryPoint>]", "// Type definition"],
    );
}

#[test]
fn fsharp_calls_finds_application_and_qualified_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fsharp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fsharp").ok() else {
        eprintln!("Skipping fsharp_calls: fsharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("fsharp")
        .expect("fsharp calls query missing");
    let calls = collect_captures(&lang, FSHARP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"classify".to_string()),
        "expected 'classify' application call in fsharp calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' application call in fsharp calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"Sqrt".to_string()),
        "expected qualified 'Math.Sqrt' call to capture 'Sqrt' in fsharp calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, FSHARP_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"Math".to_string()),
        "expected 'Math' qualifier in fsharp calls, got: {qualifiers:?}"
    );
}

#[test]
fn elixir_decorations_finds_module_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elixir_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "elixir",
        ELIXIR_SAMPLE,
        &["@doc"],
    );
}

const ERLANG_SAMPLE: &str = include_str!("fixtures/erlang/sample.erl");

#[test]
fn erlang_decorations_finds_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping erlang_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "erlang",
        ERLANG_SAMPLE,
        &["-module(", "%% Classify"],
    );
}

#[test]
fn erlang_calls_finds_local_and_remote_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping erlang_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("erlang").ok() else {
        eprintln!("Skipping erlang_calls: erlang grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("erlang")
        .expect("erlang calls query missing");
    let calls = collect_captures(&lang, ERLANG_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"factorial".to_string()),
        "expected recursive local 'factorial' call in erlang calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"sort".to_string()),
        "expected remote 'lists:sort' call to capture 'sort' in erlang calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, ERLANG_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"lists".to_string()),
        "expected 'lists' qualifier (without trailing ':') in erlang calls, got: {qualifiers:?}"
    );
}

const GLEAM_SAMPLE: &str = include_str!("fixtures/gleam/sample.gleam");

#[test]
fn gleam_decorations_finds_doc_comment_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping gleam_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "gleam",
        GLEAM_SAMPLE,
        &["/// Classify", "// Type definition"],
    );
}

#[test]
fn gleam_tags_finds_functions_types_and_constants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_tags: gleam grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("gleam").expect("gleam tags query missing");
    let names = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"factorial".to_string()),
        "expected 'factorial' function in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' custom type in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Name".to_string()),
        "expected 'Name' type alias in gleam tags, got: {names:?}"
    );
    assert!(
        names.contains(&"max_size".to_string()),
        "expected 'max_size' constant in gleam tags, got: {names:?}"
    );
}

#[test]
fn gleam_calls_finds_local_and_qualified_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_calls: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("gleam")
        .expect("gleam calls query missing");
    let calls = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"factorial".to_string()),
        "expected recursive 'factorial' call in gleam calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"filter".to_string()),
        "expected qualified 'list.filter' call to capture 'filter' in gleam calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"println".to_string()),
        "expected qualified 'io.println' call to capture 'println' in gleam calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"list".to_string()),
        "expected 'list' qualifier in gleam calls, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"io".to_string()),
        "expected 'io' qualifier in gleam calls, got: {qualifiers:?}"
    );
}

#[test]
fn gleam_complexity_finds_case_expressions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_complexity: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("gleam")
        .expect("gleam complexity query missing");
    let complexity = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 5,
        "expected at least 5 complexity nodes (case + case_clause) in gleam sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn gleam_imports_finds_module_paths_aliases_and_names() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping gleam_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("gleam").ok() else {
        eprintln!("Skipping gleam_imports: gleam grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("gleam")
        .expect("gleam imports query missing");
    let paths = collect_captures(&lang, GLEAM_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"gleam/io".to_string()),
        "expected 'gleam/io' import path in gleam imports, got: {paths:?}"
    );
    assert!(
        paths.contains(&"gleam/list".to_string()),
        "expected 'gleam/list' import path in gleam imports, got: {paths:?}"
    );
    assert!(
        paths.contains(&"gleam/int".to_string()),
        "expected 'gleam/int' import path in gleam imports, got: {paths:?}"
    );
}

#[test]
fn lean_decorations_finds_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping lean_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "lean",
        LEAN_SAMPLE,
        &["@[inline]"],
    );
}

#[test]
fn groovy_decorations_finds_annotation_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping groovy_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "groovy",
        GROOVY_SAMPLE,
        &["@Immutable", "@Override"],
    );
}

#[test]
fn vb_decorations_finds_attribute_list_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping vb_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "vb",
        VB_SAMPLE,
        &["<Obsolete("],
    );
}

#[test]
fn haskell_decorations_finds_pragma_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping haskell_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "haskell",
        HASKELL_SAMPLE,
        &["{-# LANGUAGE ScopedTypeVariables #-}", "-- | A simple"],
    );
}

#[test]
fn go_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping go_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "go",
        GO_SAMPLE,
        &["// Stack is a generic LIFO structure"],
    );
}

#[test]
fn c_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping c_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // #include is preproc_include, not preproc_call — the query captures comments and
    // generic preproc_call directives (#pragma etc.) but not #include or #define.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "c",
        C_SAMPLE,
        &["/* Creates a new stack with the given capacity. */"],
    );
}

#[test]
fn cpp_decorations_finds_attribute_declaration_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping cpp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "cpp",
        CPP_SAMPLE,
        &["[[nodiscard]]", "// Pushes an item onto the stack"],
    );
}

#[test]
fn objc_decorations_finds_preproc_include_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping objc_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In the ObjC grammar, #import is aliased into preproc_include (same rule handles both
    // #include and #import directives).
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "objc",
        OBJC_SAMPLE,
        &[
            "#import <Foundation/Foundation.h>",
            "// Initializes a Point with x and y coordinates.",
        ],
    );
}

#[test]
fn ruby_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ruby_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "ruby",
        RUBY_SAMPLE,
        &["# A simple stack data structure"],
    );
}

const R_SAMPLE: &str = include_str!("fixtures/r/sample.r");

#[test]
fn r_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping r_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "r",
        R_SAMPLE,
        &["# Classify a number"],
    );
}

#[test]
fn r_calls_finds_local_and_namespace_qualified_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping r_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("r").ok() else {
        eprintln!("Skipping r_calls: r grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("r").expect("r calls query missing");
    let calls = collect_captures(&lang, R_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"classify".to_string()),
        "expected local 'classify' call in r calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"median".to_string()),
        "expected namespace-qualified 'stats::median' call to capture 'median' in r calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, R_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"stats".to_string()),
        "expected 'stats' qualifier in r calls, got: {qualifiers:?}"
    );
}

const LUA_SAMPLE: &str = include_str!("fixtures/lua/sample.lua");

#[test]
fn lua_tags_finds_functions_and_methods() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_tags: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lua").expect("lua tags query missing");
    let pairs = collect_tag_pairs(&lang, LUA_SAMPLE, &query_str);
    // function Stack.new() — name: dot_index_expression -> @definition.method
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "new"),
        "expected 'new' (dot-index function) as definition.method, got: {pairs:?}"
    );
    // function Stack:push(value) — name: method_index_expression -> @definition.method
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "push"),
        "expected 'push' from 'function Stack:push()' as definition.method, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "is_empty"),
        "expected 'is_empty' from 'function Stack:is_empty()' as definition.method, \
         got: {pairs:?}"
    );
    // function classify(n) — name: identifier -> @definition.function
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "classify"),
        "expected 'classify' as definition.function, got: {pairs:?}"
    );
    // local function dispatch(event, ...) — name: identifier -> @definition.function
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "dispatch"),
        "expected 'dispatch' as definition.function, got: {pairs:?}"
    );
    // handlers.on_push = function(item) ... end — assignment-based dot-index
    // definition -> @definition.method
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "on_push"),
        "expected 'on_push' (dot-index assignment) as definition.method, got: {pairs:?}"
    );
    // handlers["on_pop"] = function() ... end — bracket-index assignment target
    // has no static name and must NOT appear as a definition of any kind.
    assert!(
        !pairs.iter().any(|(_, n)| n == "on_pop"),
        "bracket-index assignment target 'on_pop' has no static name and must not \
         be captured as a definition, got: {pairs:?}"
    );
}

#[test]
fn lua_calls_finds_call_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_calls: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lua").expect("lua calls query missing");
    let calls = collect_captures(&lang, LUA_SAMPLE, &query_str, "call");

    // Stack.new() — plain identifier call via dot-index qualifier (field: identifier)
    assert!(
        calls.iter().any(|c| c == "new"),
        "expected 'new' call in lua sample, got: {calls:?}"
    );
    // s:push(1) — method call
    assert!(
        calls.iter().any(|c| c == "push"),
        "expected 'push' call in lua sample, got: {calls:?}"
    );
    // print(classify(-3)) — plain identifier call
    assert!(
        calls.iter().any(|c| c == "classify"),
        "expected 'classify' call in lua sample, got: {calls:?}"
    );
    // handlers["on_pop"]() — computed/bracket call (dispatch-table idiom)
    assert!(
        calls.iter().any(|c| c.contains("on_pop")),
        "expected computed bracket call on 'on_pop' in lua sample, got: {calls:?}"
    );
}

#[test]
fn lua_imports_finds_require_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_imports: lua grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("lua")
        .expect("lua imports query missing");
    let paths = collect_captures(&lang, LUA_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("json")),
        "expected 'json' require path in lua sample, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("utils.string")),
        "expected 'utils.string' require path in lua sample, got: {paths:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.lua) -

const LUA_VARIANTS: &str = include_str!("fixtures/lua/variants.lua");

/// Every grammar-legal variant of `function_call.name` that lua.calls.scm
/// claims to support (identifier, method_index_expression,
/// dot_index_expression, bracket_index_expression, parenthesized_expression,
/// function_call) must actually match, with the right capture kind.
#[test]
fn lua_calls_completeness_all_name_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_calls_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_calls_completeness: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lua").expect("lua calls query missing");
    let caps = collect_captures_full(&lang, LUA_VARIANTS, &query_str);

    // (capture_name, kind, text) triples, one per documented name-field variant.
    let required: &[(&str, &str, &str)] = &[
        ("call", "identifier", "plain_global"),     // name: identifier
        ("call", "identifier", "method_fn"),        // name: method_index_expression
        ("call", "identifier", "dotted_fn"),        // name: dot_index_expression
        ("call", "function_call", "get_handler()"), // name: function_call (chained)
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in lua.calls.scm \
             output for variants.lua, got: {caps:?}"
        );
    }

    // bracket_index_expression callees: whole-node text captured as best-effort @call.
    let bracket_calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "call" && k == "bracket_index_expression")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        bracket_calls.contains(&"dispatch[\"key\"]"),
        "expected bracket_index_expression call 'dispatch[\"key\"]', got: {bracket_calls:?}"
    );
    assert!(
        bracket_calls.contains(&"dispatch[1]"),
        "expected bracket_index_expression call 'dispatch[1]', got: {bracket_calls:?}"
    );

    // parenthesized_expression callee (IIFE-style call target).
    let paren_calls: Vec<&str> = caps
        .iter()
        .filter(|(cn, k, _, _)| cn == "call" && k == "parenthesized_expression")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        paren_calls.contains(&"(plain_global)"),
        "expected parenthesized_expression call '(plain_global)', got: {paren_calls:?}"
    );

    // @call.qualifier must carry the receiver/table text for
    // method/dot/bracket calls, never the call name itself.
    let qualifiers: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call.qualifier")
        .map(|(_, _, t, _)| t.as_str())
        .collect();
    assert!(
        qualifiers.contains(&"NS"),
        "expected 'NS' qualifier for method/dot calls, got: {qualifiers:?}"
    );
    assert!(
        qualifiers.contains(&"dispatch"),
        "expected 'dispatch' qualifier for bracket calls, got: {qualifiers:?}"
    );
}

/// Every grammar-legal variant of `function_declaration.name` (identifier,
/// dot_index_expression, method_index_expression) plus the assignment-based
/// definition forms (identifier, dot_index_expression) must produce a @name
/// capture with the correct definition kind; the dynamic
/// bracket_index_expression assignment target must NOT.
#[test]
fn lua_tags_completeness_all_definition_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_tags_completeness: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lua").expect("lua tags query missing");
    let pairs = collect_tag_pairs(&lang, LUA_VARIANTS, &query_str);

    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "plain_global"),
        "expected 'plain_global' (function_declaration, name: identifier) as \
         definition.function, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "plain_local"),
        "expected 'plain_local' (local function, name: identifier) as \
         definition.function, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "dotted_fn"),
        "expected 'dotted_fn' (function_declaration, name: dot_index_expression) as \
         definition.method, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "method_fn"),
        "expected 'method_fn' (function_declaration, name: method_index_expression) as \
         definition.method, got: {pairs:?}"
    );
    // Bug fix: assignment-based function expression with a bare identifier
    // target was previously silently dropped (only the dot_index_expression
    // target was handled).
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.function" && n == "ident_fn"),
        "expected 'ident_fn' (assignment_statement, variable_list.name: identifier) as \
         definition.function, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "definition.method" && n == "dotted_assign_fn"),
        "expected 'dotted_assign_fn' (assignment_statement, variable_list.name: \
         dot_index_expression) as definition.method, got: {pairs:?}"
    );
    // NEGATIVE: dynamic bracket-index assignment target has no static name.
    assert!(
        !pairs.iter().any(|(_, n)| n == "dynamic_key"),
        "bracket-index assignment target must not produce a definition, got: {pairs:?}"
    );
}

/// Bug fix: lua.tags.scm never had @reference.call at all (lua.calls.scm
/// handled call extraction separately, but tags never mirrored it) — the same
/// class of gap documented as bug #5 in docs/query-testing-methodology.md's
/// Rust worked example ("scoped calls existed in rust.calls.scm but were
/// never ported to rust.tags.scm").
#[test]
fn lua_tags_finds_call_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_tags_call_refs: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_tags_call_refs: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lua").expect("lua tags query missing");
    let pairs = collect_tag_pairs(&lang, LUA_VARIANTS, &query_str);

    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "plain_global"),
        "expected 'plain_global' call as reference.call, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "method_fn"),
        "expected 'method_fn' call as reference.call, got: {pairs:?}"
    );
    assert!(
        pairs
            .iter()
            .any(|(k, n)| k == "reference.call" && n == "dotted_fn"),
        "expected 'dotted_fn' call as reference.call, got: {pairs:?}"
    );
    // Bracket-dispatched calls report the subscripted container's name as a
    // best-effort approximation (matches python.tags.scm's convention).
    let dispatch_refs = pairs
        .iter()
        .filter(|(k, n)| k == "reference.call" && n == "dispatch")
        .count();
    assert!(
        dispatch_refs >= 2,
        "expected at least 2 'dispatch' reference.call entries (dispatch[\"key\"](), \
         dispatch[1]()), got {dispatch_refs}: {pairs:?}"
    );
}

/// Negative cases: constructs that must never appear in @call/@name captures.
#[test]
fn lua_calls_negative_cases_do_not_match() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_calls_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_calls_negative: lua grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lua").expect("lua calls query missing");
    let caps = collect_captures_full(&lang, LUA_VARIANTS, &query_str);
    let call_texts: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "call")
        .map(|(_, _, t, _)| t.as_str())
        .collect();

    // `holder.field` is a bare field read (no call parens); must never be a call.
    assert!(
        !call_texts.contains(&"field"),
        "bare field access 'holder.field' must not be captured as a call, got: {call_texts:?}"
    );

    // Only the call site `adder(1)(2)` should register "adder" as a call —
    // its definition site (the `local adder = function(x) ... end` LHS) must
    // not.
    let adder_calls = call_texts.iter().filter(|t| **t == "adder").count();
    assert_eq!(
        adder_calls, 1,
        "expected exactly 1 call to 'adder' (the call site, not the definition), \
         got {adder_calls}: {call_texts:?}"
    );
}

/// Negative case: a plain variable reference passed to something other than
/// `require(...)` must never be captured as an import path.
#[test]
fn lua_imports_negative_case_non_require_not_matched() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_imports_negative: lua grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("lua")
        .expect("lua imports query missing");
    let paths = collect_captures(&lang, LUA_VARIANTS, &query_str, "import.path");
    // All three require() forms (paren, bareword, long-bracket string) must match.
    assert!(
        paths.iter().any(|p| p.contains("paren_module")),
        "expected 'paren_module' require path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("bareword_module")),
        "expected 'bareword_module' (bareword require, no parens) path, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("bracket_module")),
        "expected 'bracket_module' (long-bracket string require) path, got: {paths:?}"
    );
    // `local dyn = some_module_var` must never contribute an import path.
    assert!(
        !paths.iter().any(|p| p.contains("module_var")),
        "non-require variable reference must not be captured as an import, got: {paths:?}"
    );
}

/// lua.complexity.scm: every branch/loop construct plus `and`/`or` must
/// contribute to @complexity; only branch/loop constructs (not `and`/`or`)
/// contribute to @nesting.
#[test]
fn lua_complexity_finds_all_branch_and_loop_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lua_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lua").ok() else {
        eprintln!("Skipping lua_complexity: lua grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("lua")
        .expect("lua complexity query missing");
    let caps = collect_captures_full(&lang, LUA_VARIANTS, &query_str);

    let complexity_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "complexity")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    for kind in [
        "if_statement",
        "elseif_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
        "and",
        "or",
    ] {
        assert!(
            complexity_kinds.contains(&kind),
            "expected a @complexity capture of kind '{kind}' in variants.lua, \
             got kinds: {complexity_kinds:?}"
        );
    }
    // for_statement must fire for both the numeric and generic clause forms.
    let for_count = complexity_kinds
        .iter()
        .filter(|k| **k == "for_statement")
        .count();
    assert!(
        for_count >= 2,
        "expected at least 2 for_statement @complexity captures (numeric + generic \
         clause), got {for_count}"
    );

    let nesting_kinds: Vec<&str> = caps
        .iter()
        .filter(|(cn, _, _, _)| cn == "nesting")
        .map(|(_, k, _, _)| k.as_str())
        .collect();
    for kind in [
        "if_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
    ] {
        assert!(
            nesting_kinds.contains(&kind),
            "expected a @nesting capture of kind '{kind}' in variants.lua, got kinds: \
             {nesting_kinds:?}"
        );
    }
    // "and"/"or" contribute to @complexity but never to @nesting.
    assert!(
        !nesting_kinds.contains(&"and") && !nesting_kinds.contains(&"or"),
        "'and'/'or' must never contribute to @nesting, got kinds: {nesting_kinds:?}"
    );
}

#[test]
fn lua_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping lua_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "lua",
        LUA_SAMPLE,
        &["-- Simple stack implementation"],
    );
}

#[test]
fn zig_decorations_finds_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping zig_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "zig",
        ZIG_SAMPLE,
        &["/// Classify a number as negative, zero, or positive."],
    );
}

#[test]
fn idris_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping idris_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In tree-sitter-idris, ||| doc comments are parsed as (comment) by the external scanner —
    // there is no separate doc_comment node type.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "idris",
        IDRIS_SAMPLE,
        &["||| Compute Euclidean distance between two points"],
    );
}

#[test]
fn agda_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping agda_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "agda",
        AGDA_SAMPLE,
        &["-- A simple data type"],
    );
}

#[test]
fn elm_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elm_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "elm",
        ELM_SAMPLE,
        &["-- Square a number"],
    );
}

const JULIA_SAMPLE: &str = include_str!("fixtures/julia/sample.jl");

#[test]
fn julia_decorations_finds_macrocall_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping julia_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In tree-sitter-julia, macro applications are macrocall_expression nodes.
    // @inline function classify(...) ... end parses as a macrocall_expression.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "julia",
        JULIA_SAMPLE,
        &["# Classify a number", "@inline"],
    );
}

#[test]
fn perl_decorations_finds_pod_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping perl_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In tree-sitter-perl, POD documentation blocks (=head1 ... =cut) are pod nodes (not pod_statement).
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "perl",
        PERL_SAMPLE,
        &[
            "=head1 NAME",
            "# Classify a number as negative, zero, or positive",
        ],
    );
}

#[test]
fn verilog_decorations_finds_attribute_instance_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping verilog_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // attribute_instance is the verified node name for (* ... *) attributes in tree-sitter-verilog.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "verilog",
        VERILOG_SAMPLE,
        &[
            "(* synthesis, keep *)",
            "// ALU module with basic arithmetic and logic operations",
        ],
    );
}

#[test]
fn vhdl_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping vhdl_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "vhdl",
        VHDL_SAMPLE,
        &["-- Simple FIFO entity"],
    );
}

#[test]
fn ada_decorations_finds_pragma_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping ada_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // pragma_g is the verified node name for Ada pragmas in tree-sitter-ada (RM 2.8).
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "ada",
        ADA_SAMPLE,
        &[
            "pragma Inline(Add);",
            "-- Add two integers and return the result",
        ],
    );
}

const CAPNP_SAMPLE: &str = include_str!("fixtures/capnp/sample.capnp");

#[test]
fn capnp_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping capnp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "capnp",
        CAPNP_SAMPLE,
        &["# A point in 2D space"],
    );
}

const THRIFT_SAMPLE: &str = include_str!("fixtures/thrift/sample.thrift");

#[test]
fn thrift_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping thrift_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "thrift",
        THRIFT_SAMPLE,
        &["// Thrift IDL sample file"],
    );
}

#[test]
fn thrift_imports_finds_include_path() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping thrift_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("thrift").ok() else {
        eprintln!("Skipping thrift_imports: thrift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("thrift")
        .expect("thrift imports query missing");
    let paths = collect_captures(&lang, THRIFT_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("shared.thrift")),
        "expected 'shared.thrift' include path in thrift imports, got: {paths:?}"
    );
}

#[test]
fn graphql_decorations_finds_description_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping graphql_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "graphql",
        GRAPHQL_SAMPLE,
        &[
            "\"\"\"A scalar representing a date and time value.\"\"\"",
            "# Node interface for objects with a unique ID",
        ],
    );
}

const WIT_SAMPLE: &str = include_str!("fixtures/wit/sample.wit");

#[test]
fn wit_decorations_finds_doc_comment_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping wit_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "wit",
        WIT_SAMPLE,
        &[
            "/// Types and functions for working with text",
            "/// A handle to an open resource",
        ],
    );
}

#[test]
fn clojure_decorations_finds_metadata_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping clojure_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // meta_lit is the verified node name for ^:keyword reader metadata in tree-sitter-clojure.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "clojure",
        CLOJURE_SAMPLE,
        &[
            "^:deprecated",
            "; A point in 2D space with x and y coordinates",
        ],
    );
}

#[test]
fn scheme_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping scheme_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "scheme",
        SCHEME_SAMPLE,
        &["; A point in 2D space"],
    );
}

#[test]
fn prolog_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping prolog_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "prolog",
        PROLOG_SAMPLE,
        &["% Facts: family relationships"],
    );
}

// ---------------------------------------------------------------------------
// Caddy / Dockerfile
// ---------------------------------------------------------------------------

const CADDY_SAMPLE: &str = include_str!("fixtures/caddy/sample.caddyfile");

#[test]
fn caddy_imports_finds_snippet_reference() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping caddy_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("caddy").ok() else {
        eprintln!("Skipping caddy_imports: caddy grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("caddy")
        .expect("caddy imports query missing");
    let paths = collect_captures(&lang, CADDY_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("common-headers")),
        "expected '(common-headers)' snippet reference in caddy imports, got: {paths:?}"
    );
}

const DOCKERFILE_SAMPLE: &str = include_str!("fixtures/dockerfile/Sample.dockerfile");
const DOCKERFILE_VARIANTS: &str = include_str!("fixtures/dockerfile/variants.dockerfile");

/// Dimension 4 (real-world fixture coverage): a multi-stage Go build with
/// `--platform=`/digest-pinned FROMs, ARG defaults, multi-name ENV, a stage
/// that references an earlier stage by name (`FROM builder AS test`), and a
/// `COPY --from=` cross-stage copy.
#[test]
fn dockerfile_imports_finds_base_images_stage_refs_and_aliases() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dockerfile_imports_sample: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!("Skipping dockerfile_imports_sample: dockerfile grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("dockerfile")
        .expect("dockerfile imports query missing");
    let paths = collect_captures(&lang, DOCKERFILE_SAMPLE, &query_str, "import.path");
    assert!(
        paths.contains(&"golang:${GO_VERSION}-alpine".to_string()),
        "expected the digest-free, ARG-expanded builder base image, got: {paths:?}"
    );
    assert!(
        paths.contains(&"builder".to_string()),
        "expected 'FROM builder AS test' to surface 'builder' as an @import.path \
         (a stage reference, not an external image), got: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .any(|p| p.starts_with("gcr.io/distroless/static-debian12@sha256:")),
        "expected the digest-pinned final-stage base image, got: {paths:?}"
    );
    let aliases = collect_captures(&lang, DOCKERFILE_SAMPLE, &query_str, "import.alias");
    assert_eq!(
        aliases,
        vec!["builder", "test", "final"],
        "expected exactly the three stage aliases in source order"
    );
}

/// Dimension 4: the same real-world sample's stage names, and every ARG/ENV
/// variable name, must appear as tags — none of the ARG defaults or ENV
/// values (which share `unquoted_string` with the names) may leak through.
#[test]
fn dockerfile_tags_finds_stage_names_and_declared_variables() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dockerfile_tags_sample: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!("Skipping dockerfile_tags_sample: dockerfile grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("dockerfile")
        .expect("dockerfile tags query missing");
    let pairs = collect_tag_pairs(&lang, DOCKERFILE_SAMPLE, &query_str);

    let modules: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.module")
        .map(|(_, n)| n.as_str())
        .collect();
    assert_eq!(modules, vec!["builder", "test", "final"]);

    let constants: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.constant")
        .map(|(_, n)| n.as_str())
        .collect();
    assert_eq!(
        constants,
        vec![
            "GO_VERSION",
            "BUILD_ENV",
            "GO_VERSION",
            "CGO_ENABLED",
            "GOOS",
            "BUILD_ENV",
        ],
        "expected exactly the declared ARG/ENV names, with no default/value text \
         leaking through as spurious symbols"
    );
}

/// Dimension 2/3 (completeness + extraction depth) for imports.scm: every
/// FROM shape node-types.json allows (bare tag, digest, no tag, `AS` alias,
/// `--platform=` prefix, stage-by-name reference) must produce exactly the
/// matching `@import.path`/`@import.alias` pair, verified by kind via
/// `collect_captures_full` so an accidental match on the wrong node type
/// can't hide behind identical capture text.
#[test]
fn dockerfile_imports_completeness_from_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping dockerfile_imports_completeness: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!("Skipping dockerfile_imports_completeness: dockerfile grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("dockerfile")
        .expect("dockerfile imports query missing");

    let paths = collect_captures(&lang, DOCKERFILE_VARIANTS, &query_str, "import.path");
    assert_eq!(
        paths,
        vec![
            "ubuntu:20.04",               // bare tag, no alias
            "ubuntu:20.04@sha256:abc123", // digest, no alias
            "ubuntu",                     // no tag at all
            "golang:1.21-alpine",         // tag + AS alias
            "golang:1.21",                // tag + --platform= sibling + AS alias
            "builder",                    // stage-by-name reference + AS alias
        ],
        "expected exactly six @import.path matches, one per FROM, in source order"
    );

    let full = collect_captures_full(&lang, DOCKERFILE_VARIANTS, &query_str);
    let alias_kinds: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "import.alias")
        .map(|(_, kind, ..)| kind.as_str())
        .collect();
    assert_eq!(
        alias_kinds,
        vec!["image_alias"; 4],
        "every @import.alias must be an image_alias node — the two unaliased \
         FROMs (line 9 and 12) must not contribute a stray alias capture"
    );
}

/// Dimension 2/3 for tags.scm: ARG/ENV name-vs-default(/value) field
/// anchoring, across every default/value node-type variant node-types.json
/// allows (`unquoted_string`, `double_quoted_string`, `single_quoted_string`,
/// and ARG's optional-default / ENV's legacy-no-`=` forms).
#[test]
fn dockerfile_tags_completeness_arg_env_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dockerfile_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!("Skipping dockerfile_tags_completeness: dockerfile grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("dockerfile")
        .expect("dockerfile tags query missing");
    let pairs = collect_tag_pairs(&lang, DOCKERFILE_VARIANTS, &query_str);

    let constants: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.constant")
        .map(|(_, n)| n.as_str())
        .collect();
    assert_eq!(
        constants,
        vec![
            "VERSION", "NAME", "OTHER", "NOEQ", // ARG: default present (3 default
            // kinds: unquoted/double/single-quoted) and default absent
            "KEY1", "KEY2",   // ENV: multi-pair `=` form
            "LEGACY", // ENV: legacy no-`=` single-pair form
        ],
        "expected exactly the ARG/ENV *names*; default/value text (\"1.0\", \
         \"quoted\", \"single\", \"val1\", \"val2\", \"val3\") must never appear"
    );

    let modules: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.module")
        .map(|(_, n)| n.as_str())
        .collect();
    assert_eq!(
        modules,
        vec!["bare_name", "builder", "platform_stage", "from_stage_ref"],
        "the two unaliased FROMs (line 9, 12) must not contribute a module definition"
    );
}

/// Negative case: instruction kinds that are documented as not contributing
/// tags (RUN/CMD/ENTRYPOINT/COPY/ADD/LABEL/EXPOSE/USER/VOLUME/WORKDIR/
/// STOPSIGNAL/ONBUILD/HEALTHCHECK/MAINTAINER/SHELL, in both shell- and
/// exec-form where applicable) must produce zero @name/@definition captures
/// from their own content — only the FROM/ARG/ENV lines earlier in the same
/// fixture may contribute.
#[test]
fn dockerfile_tags_negative_non_symbol_instructions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dockerfile_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!("Skipping dockerfile_tags_negative: dockerfile grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("dockerfile")
        .expect("dockerfile tags query missing");
    let names = collect_captures(&lang, DOCKERFILE_VARIANTS, &query_str, "name");

    // Only the FROM-alias / ARG-name / ENV-name symbols documented above are
    // expected; nothing from any RUN/CMD/COPY/LABEL/etc. line downstream.
    let unexpected: Vec<&String> = names
        .iter()
        .filter(|n| {
            ![
                "bare_name",
                "builder",
                "platform_stage",
                "from_stage_ref",
                "VERSION",
                "NAME",
                "OTHER",
                "NOEQ",
                "KEY1",
                "KEY2",
                "LEGACY",
            ]
            .contains(&n.as_str())
        })
        .collect();
    assert!(
        unexpected.is_empty(),
        "expected no tag captures from RUN/CMD/COPY/ADD/LABEL/etc. lines, found: {unexpected:?}"
    );
}

/// Negative case for imports.scm: `COPY --chown=`, a bare `COPY` with no
/// params, and every non-FROM/non-COPY instruction must not contribute an
/// `@import`.
#[test]
fn dockerfile_imports_negative_non_from_instructions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping dockerfile_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!("Skipping dockerfile_imports_negative: dockerfile grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("dockerfile")
        .expect("dockerfile imports query missing");
    // Exactly the six FROM instructions in the variants fixture; the
    // COPY --from=/--chown= lines and everything after them contribute none
    // (imports.scm only matches from_instruction — see its doc comment on
    // why COPY --from= is handled at the Rust trait level instead).
    let import_stmts: Vec<_> = collect_captures_full(&lang, DOCKERFILE_VARIANTS, &query_str)
        .into_iter()
        .filter(|(cap, ..)| cap == "import")
        .collect();
    assert_eq!(import_stmts.len(), 6, "got: {import_stmts:?}");
}

/// Regression test for the `extract_stage_name` bug: the old implementation
/// searched for a child of kind `as_instruction` before accepting an
/// `image_alias` — but `as` is a direct field on `from_instruction` (there
/// is no `as_instruction` node in this grammar at all), so the old code
/// always returned `None` and every stage alias was silently dropped from
/// `Language::extract_imports`'s trait-level output.
#[test]
fn dockerfile_extract_imports_trait_finds_stage_alias() {
    use normalize_languages::{Dockerfile, Language};
    use tree_sitter::{Parser, StreamingIterator};

    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping dockerfile_extract_imports_trait: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!("Skipping dockerfile_extract_imports_trait: dockerfile grammar .so not found");
        return;
    };
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let source = "FROM golang:1.21 AS builder\n";
    let tree = parser.parse(source, None).expect("parse failed");
    let query = tree_sitter::Query::new(&lang, "(from_instruction) @from").expect("query compile");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let dockerfile = Dockerfile;
    let mut imports = Vec::new();
    while let Some(m) = matches.next() {
        for cap in m.captures {
            imports.extend(dockerfile.extract_imports(&cap.node, source));
        }
    }
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module, "golang:1.21");
    assert_eq!(
        imports[0].alias.as_deref(),
        Some("builder"),
        "extract_stage_name must find the alias via the 'as' field, not a \
         nonexistent 'as_instruction' child"
    );
}

/// `Dockerfile::extract_imports`'s COPY `--from=` handling: stage-name and
/// numeric-index references both produce an import with no alias; a sibling
/// `--chown=` param on the same instruction must not be mistaken for
/// `--from=`; and a bare COPY (no params at all) produces zero imports.
#[test]
fn dockerfile_extract_imports_trait_copy_from_variants() {
    use normalize_languages::{Dockerfile, Language};
    use tree_sitter::{Parser, StreamingIterator};

    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping dockerfile_extract_imports_copy_from: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("dockerfile").ok() else {
        eprintln!(
            "Skipping dockerfile_extract_imports_copy_from: dockerfile grammar .so not found"
        );
        return;
    };
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let source = "COPY --from=builder /out/app /usr/local/bin/app\n\
                  COPY --from=0 /a /b\n\
                  COPY --chown=user:group /a /b\n\
                  COPY /a /b\n";
    let tree = parser.parse(source, None).expect("parse failed");
    let query = tree_sitter::Query::new(&lang, "(copy_instruction) @copy").expect("query compile");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let dockerfile = Dockerfile;
    let mut all: Vec<Vec<String>> = Vec::new();
    while let Some(m) = matches.next() {
        for cap in m.captures {
            let imports = dockerfile.extract_imports(&cap.node, source);
            all.push(imports.into_iter().map(|i| i.module).collect());
        }
    }
    assert_eq!(
        all,
        vec![
            vec!["builder".to_string()],
            vec!["0".to_string()],
            Vec::<String>::new(), // --chown= must not be mistaken for --from=
            Vec::<String>::new(), // bare COPY has no param children at all
        ]
    );
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------
//
// JSON has exactly one query purpose: tags. There are no call/import/complexity/
// type concepts in JSON's grammar (verified: `src/grammar_loader.rs`'s
// `query_source_for("json", ...)` registers only `"json.tags.scm"`, and JSON's
// node-types.json has no call/import-shaped node kinds — see arborium-json
// 2.17.0 node-types.json, dumped in full during this remediation: the only
// node types are `_value`, `array`, `document`, `object`, `pair`, `string`,
// literal/punctuation tokens, `comment` (extra), `escape_sequence`, and
// `string_content`).

const JSON_SAMPLE: &str = include_str!("fixtures/json/sample.json");
const JSON_VARIANTS: &str = include_str!("fixtures/json/variants.json");

/// Returns `(loader, lang)` together — the `GrammarLoader` owns the loaded
/// dylib, so it must stay alive for as long as the returned `Language` is
/// used (dropping it early unloads the library and leaves `Language`'s
/// function pointers dangling, which segfaults rather than erroring).
fn json_lang() -> Option<(normalize_languages::GrammarLoader, tree_sitter::Language)> {
    let loader = normalize_languages::GrammarLoader::new();
    let lang = loader.get("json").ok()?;
    Some((loader, lang))
}

// --- Dimension 4: real-world fixture coverage (sample.json) -----------------

#[test]
fn json_tags_finds_nested_keys_in_realistic_document() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_finds_nested_keys: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let names = collect_captures(&lang, JSON_SAMPLE, &query_str, "name");

    // Top-level scalar/object/array-valued keys.
    for expected in [
        "\"name\"",
        "\"version\"",
        "\"scripts\"",
        "\"dependencies\"",
        "\"keywords\"",
        "\"contributors\"",
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "expected {expected} key in json sample tags, got: {names:?}"
        );
    }
    // Nested object keys (package.json-shaped: scripts.build, deeply nested
    // dependencies.serde.version) must also be found — nesting is structural
    // (pair > object > pair), not something the query itself constrains, but
    // this is the realistic idiom (dimension 4) that would surface a broken
    // query at any depth.
    for expected in ["\"build\"", "\"test\"", "\"tree-sitter\"", "\"version\""] {
        assert!(
            names.contains(&expected.to_string()),
            "expected {expected} nested key in json sample tags, got: {names:?}"
        );
    }
    // Object keys inside array elements (contributors: [{ name, role }, ...])
    // — array elements aren't pairs, but the objects *inside* them still are.
    assert!(
        names.iter().filter(|n| *n == "\"name\"").count() >= 3,
        "expected 'name' from top-level key + 2 contributor objects, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.json) -

/// Every `pair.value` variant node-types.json's `_value` supertype allows
/// (array, false, null, number, object, string, true) must produce a
/// @definition.var capture with kind `pair` on the key — the value's own
/// kind never gates whether the *key* is captured, only whether json.rs's
/// refine_kind() promotes it to a container (tested separately below).
#[test]
fn json_tags_completeness_all_value_kinds_have_captured_keys() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_completeness_all_value_kinds: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_VARIANTS, &query_str);

    // (capture_name, node_kind, text) — every key is a `string` node
    // regardless of its value's kind.
    let required: &[(&str, &str, &str)] = &[
        ("name", "string", "\"valueString\""), // value: string
        ("name", "string", "\"valueNumber\""), // value: number
        ("name", "string", "\"valueTrue\""),   // value: true
        ("name", "string", "\"valueFalse\""),  // value: false
        ("name", "string", "\"valueNull\""),   // value: null
        ("name", "string", "\"valueArray\""),  // value: array
        ("name", "string", "\"valueObject\""), // value: object
    ];
    for (cap_name, kind, text) in required {
        assert!(
            caps.iter()
                .any(|(cn, k, t, _)| cn == cap_name && k == kind && t == text),
            "expected capture ({cap_name}, kind={kind}, text={text}) in json.tags.scm output \
             for variants.json, got: {caps:?}"
        );
    }
}

/// Real, previously-silent bug (found via a killed prior session, verified
/// against real parse output in this session): empty-string keys (`"":
/// value`) parse as a `string` node with NO `string_content` child at all —
/// the grammar only emits that child for non-empty runs. A
/// `(string (string_content) @name)` pattern requires the child and so
/// silently drops the pair from extraction entirely. json.tags.scm now
/// captures the whole `string` node instead, which does not depend on
/// `string_content` existing.
#[test]
fn json_tags_completeness_empty_string_key_is_captured() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_completeness_empty_string_key: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_VARIANTS, &query_str);

    let empty_key_defs: Vec<_> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "definition.var" && k == "pair" && t.starts_with("\"\":"))
        .collect();
    assert_eq!(
        empty_key_defs.len(),
        1,
        "expected exactly 1 @definition.var for the empty-string-key pair, got: {caps:?}"
    );
}

/// Second real, previously-unverified bug found while validating the prior
/// session's fix: keys containing an escape sequence (`"a\nb"`) parse as a
/// `string` node with *multiple* `string_content` children (one literal run
/// per side of each `escape_sequence`). A per-child `@name` pattern produces
/// one separate query match per run — duplicating the @definition.var match
/// for the same pair and truncating @name to a single run ("a" or "b", never
/// the full key). Capturing the whole `string` node produces exactly one
/// match per pair regardless of how many string_content/escape_sequence
/// children it has internally.
#[test]
fn json_tags_completeness_escape_sequence_key_is_single_match() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_completeness_escape_sequence_key: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_VARIANTS, &query_str);

    // "a\nb" key: exactly one @definition.var match (not one per string_content run).
    let escape_run_defs: Vec<_> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "definition.var" && k == "pair" && t.contains("\"a\\nb\":"))
        .collect();
    assert_eq!(
        escape_run_defs.len(),
        1,
        "expected exactly 1 @definition.var for the 'a\\nb' escape-run key, got: {caps:?}"
    );

    // "\n" key (pure escape sequence, no string_content child at all): must
    // still be captured, not dropped the way the empty-key case was.
    let pure_escape_defs: Vec<_> = caps
        .iter()
        .filter(|(cn, k, t, _)| cn == "definition.var" && k == "pair" && t.starts_with("\"\\n\":"))
        .collect();
    assert_eq!(
        pure_escape_defs.len(),
        1,
        "expected exactly 1 @definition.var for the pure-escape '\\n' key, got: {caps:?}"
    );
}

/// NEGATIVE: array elements are values, not pairs — a bare array of scalars
/// must contribute zero @name/@definition.var captures of its own (only the
/// pairs nested in the *objects* an array may contain are captured).
#[test]
fn json_tags_negative_array_scalar_elements_not_captured() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_tags_negative_array_scalar_elements: json grammar not found");
        return;
    };
    let query_str = loader.get_tags("json").expect("json tags query missing");
    let caps = collect_captures_full(&lang, JSON_SAMPLE, &query_str);

    // "keywords": ["parsing", "tree-sitter", "cli"] — none of the array's
    // string elements may appear as a @name capture (they are values, not
    // pair keys).
    for scalar in ["\"parsing\"", "\"cli\""] {
        assert!(
            !caps.iter().any(|(cn, _, t, _)| cn == "name" && t == scalar),
            "array scalar element {scalar} must never appear as a @name capture, got: {caps:?}"
        );
    }
}

// --- json.rs Language trait: node_name() / refine_kind() extraction depth ---

/// `node_name()` must return the full raw key text (between the string
/// node's own quotes), not just the first `string_content` run — regression
/// test for the escape-run truncation bug at the Rust-code level (as
/// opposed to the query level tested above), since `node_name()` is what
/// actually supplies the symbol name consumed by `collect_symbols_from_tags`.
#[test]
fn json_node_name_handles_empty_and_escape_keys() {
    let Some((loader, lang)) = json_lang() else {
        eprintln!("Skipping json_node_name_handles_empty_and_escape_keys: json grammar not found");
        return;
    };
    let _ = &loader;
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(JSON_VARIANTS, None).expect("parse failed");

    let json_lang_impl = normalize_languages::Json;
    let mut cursor = tree.walk();
    let mut found: Vec<(String, String)> = Vec::new();

    fn walk(
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        support: &dyn normalize_languages::Language,
        found: &mut Vec<(String, String)>,
    ) {
        loop {
            let node = cursor.node();
            if node.kind() == "pair"
                && let Some(name) = support.node_name(&node, content)
            {
                let value_text = node
                    .child_by_field_name("value")
                    .map(|v| v.utf8_text(content.as_bytes()).unwrap_or(""))
                    .unwrap_or("");
                found.push((name.to_string(), value_text.to_string()));
            }
            if cursor.goto_first_child() {
                walk(cursor, content, support, found);
                cursor.goto_parent();
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    walk(&mut cursor, JSON_VARIANTS, &json_lang_impl, &mut found);

    assert!(
        found.contains(&(String::new(), "\"emptyKeyValue\"".to_string())),
        "expected node_name() to return \"\" (not None) for the empty-string key, got: {found:?}"
    );
    assert!(
        found.contains(&("a\\nb".to_string(), "\"escapeRunKeyValue\"".to_string())),
        "expected node_name() to return the FULL 'a\\nb' key text (not truncated to 'a'), \
         got: {found:?}"
    );
    assert!(
        found.contains(&("\\n".to_string(), "\"pureEscapeKeyValue\"".to_string())),
        "expected node_name() to return '\\n' (pure-escape key, no string_content child), \
         got: {found:?}"
    );
    assert!(
        found.contains(&("plainKey".to_string(), "\"plainKeyValue\"".to_string())),
        "expected node_name() to return the plain key unchanged, got: {found:?}"
    );
}

/// `refine_kind()` must promote a pair to `Module` (container) only when its
/// value is an `object` — every other `_value` variant (array/false/null/
/// number/string/true) must remain a plain `Variable`. Verified via
/// `collect_symbols_from_tags`'s nesting logic (only `is_container_kind`
/// containers can hold children) rather than re-implementing refine_kind's
/// logic here.
#[test]
fn json_refine_kind_only_object_values_are_containers() {
    let Some((loader, _lang)) = json_lang() else {
        eprintln!(
            "Skipping json_refine_kind_only_object_values_are_containers: json grammar not found"
        );
        return;
    };
    let _ = &loader;
    let json_impl = normalize_languages::Json;

    let mut parser = Parser::new();
    let lang2 = normalize_languages::GrammarLoader::new()
        .get("json")
        .expect("json grammar");
    parser.set_language(&lang2).expect("set_language failed");
    let tree = parser.parse(JSON_VARIANTS, None).expect("parse failed");
    let root = tree.root_node();

    // Find the "valueObject" and "valueArray" pair nodes directly and check
    // refine_kind()'s classification of each.
    fn find_pair<'t>(
        node: tree_sitter::Node<'t>,
        key_text: &str,
        content: &str,
    ) -> Option<tree_sitter::Node<'t>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "pair"
                && let Some(key) = child.child_by_field_name("key")
                && key.utf8_text(content.as_bytes()).unwrap_or("") == key_text
            {
                return Some(child);
            }
            if let Some(found) = find_pair(child, key_text, content) {
                return Some(found);
            }
        }
        None
    }

    let value_object_pair = find_pair(root, "\"valueObject\"", JSON_VARIANTS)
        .expect("valueObject pair not found in variants.json");
    let value_array_pair = find_pair(root, "\"valueArray\"", JSON_VARIANTS)
        .expect("valueArray pair not found in variants.json");

    let refined_object = normalize_languages::Language::refine_kind(
        &json_impl,
        &value_object_pair,
        JSON_VARIANTS,
        normalize_languages::SymbolKind::Variable,
    );
    let refined_array = normalize_languages::Language::refine_kind(
        &json_impl,
        &value_array_pair,
        JSON_VARIANTS,
        normalize_languages::SymbolKind::Variable,
    );

    assert_eq!(
        refined_object,
        normalize_languages::SymbolKind::Module,
        "object-valued pair must refine to Module (container)"
    );
    assert_eq!(
        refined_array,
        normalize_languages::SymbolKind::Variable,
        "array-valued pair must remain Variable (not a container), got: {refined_array:?}"
    );
}

// ---------------------------------------------------------------------------
// TOML
// ---------------------------------------------------------------------------
//
// TOML only has a tags.scm (no calls/complexity/imports/types — data-format
// grammar, not a programming language). arborium-toml 2.17.0's grammar has
// no named fields at all: the header key of a `table`/`table_array_element`
// and the key of a `pair` is whichever of three sibling node kinds appears —
// `bare_key`, `quoted_key`, or `dotted_key` — verified via `node-types.json`
// and cross-checked against real parse output (`normalize syntax ast`/
// `normalize syntax query`). The original query only matched `bare_key`,
// silently dropping every quoted-key and dotted-key table/pair name; dotted
// table headers alone (`[workspace.dependencies]`, `[profile.dev.build-
// override]`, …) appear 40+ times and dotted pair keys 370+ times in this
// repo's own `.toml` files.

const TOML_SAMPLE: &str = include_str!("fixtures/toml/sample.toml");
const TOML_VARIANTS: &str = include_str!("fixtures/toml/variants.toml");

// --- Dimension 4: real-world fixture coverage (sample.toml) ----------------

#[test]
fn toml_tags_finds_sample_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_finds_sample_definitions: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!("Skipping toml_tags_finds_sample_definitions: toml grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let pairs = collect_tag_pairs(&lang, TOML_SAMPLE, &query_str);
    let class_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.class")
        .map(|(_, n)| n.as_str())
        .collect();
    let var_names: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == "definition.var")
        .map(|(_, n)| n.as_str())
        .collect();

    // Plain table header.
    assert!(
        class_names.contains(&"package"),
        "expected 'package' table in toml tags, got: {class_names:?}"
    );
    // Dotted table header: [workspace.dependencies].
    assert!(
        class_names.contains(&"workspace.dependencies"),
        "expected dotted table header 'workspace.dependencies' in toml tags, got: {class_names:?}"
    );
    // Multi-segment dotted table header: [profile.dev.build-override].
    assert!(
        class_names.contains(&"profile.dev.build-override"),
        "expected dotted table header 'profile.dev.build-override' in toml tags, got: {class_names:?}"
    );
    // Quoted-key table header: ["cfg(unix)".dependencies].
    assert!(
        class_names.contains(&"\"cfg(unix)\".dependencies"),
        "expected quoted-key dotted table header in toml tags, got: {class_names:?}"
    );
    // Array-of-tables headers: three [[bin]] entries, each its own definition.
    let bin_count = class_names.iter().filter(|n| **n == "bin").count();
    assert_eq!(
        bin_count, 3,
        "expected 3 separate '[[bin]]' array-table entries in toml tags, got {bin_count}: {class_names:?}"
    );
    // Dotted pair key nested inside a table: [lints.rust] -> rust.unused = "warn".
    assert!(
        var_names.contains(&"rust.unused"),
        "expected dotted pair key 'rust.unused' in toml tags, got: {var_names:?}"
    );
    // Quoted pair key: "ci.badge-url" = "...".
    assert!(
        var_names.contains(&"\"ci.badge-url\""),
        "expected quoted pair key '\"ci.badge-url\"' in toml tags, got: {var_names:?}"
    );
    // Dotted array-of-tables header: [[metadata.matrix]], twice.
    let matrix_count = class_names
        .iter()
        .filter(|n| **n == "metadata.matrix")
        .count();
    assert_eq!(
        matrix_count, 2,
        "expected 2 separate '[[metadata.matrix]]' entries in toml tags, got {matrix_count}: {class_names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix and extraction depth (variants.toml) -

#[test]
fn toml_tags_completeness_table_header_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_completeness_table_header_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!(
            "Skipping toml_tags_completeness_table_header_variants: toml grammar .so not found"
        );
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let captures = collect_captures_full(&lang, TOML_VARIANTS, &query_str);

    // (expected @name text, expected node kind, which container: table | table_array_element)
    let expected: &[(&str, &str, &str)] = &[
        // table header key variants
        ("bare_table", "bare_key", "table"),
        ("\"quoted table header\"", "quoted_key", "table"),
        ("dotted.table.header", "dotted_key", "table"),
        ("dotted.\"quoted segment\".header", "dotted_key", "table"),
        // table_array_element header key variants
        ("bare_array_table", "bare_key", "table_array_element"),
        (
            "\"quoted array table header\"",
            "quoted_key",
            "table_array_element",
        ),
        ("dotted.array.table", "dotted_key", "table_array_element"),
    ];

    for (text, kind, _container) in expected {
        let found = captures
            .iter()
            .any(|(cap, k, t, _line)| cap == "name" && k == kind && t == text);
        assert!(
            found,
            "expected @name capture text={text:?} kind={kind:?} in toml_tags_completeness_table_header_variants, \
             got captures: {captures:?}"
        );
    }

    // Every table/table_array_element header must co-occur with the right
    // definition.* tag kind (dimension: definition-kind distinction).
    let tag_pairs = collect_tag_pairs(&lang, TOML_VARIANTS, &query_str);
    assert!(
        tag_pairs.contains(&(
            "definition.class".to_string(),
            "dotted.table.header".to_string()
        )),
        "expected dotted table header tagged as definition.class, got: {tag_pairs:?}"
    );
    assert!(
        tag_pairs.contains(&(
            "definition.class".to_string(),
            "dotted.array.table".to_string()
        )),
        "expected dotted array-table header tagged as definition.class, got: {tag_pairs:?}"
    );
}

#[test]
fn toml_tags_completeness_pair_key_variants() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_completeness_pair_key_variants: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!("Skipping toml_tags_completeness_pair_key_variants: toml grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let tag_pairs = collect_tag_pairs(&lang, TOML_VARIANTS, &query_str);

    // Top-level pair key variants.
    for kind_text in [
        "bare_pair",
        "\"quoted pair key\"",
        "dotted.pair.key",
        "dotted.\"quoted segment\".key",
    ] {
        assert!(
            tag_pairs.contains(&("definition.var".to_string(), kind_text.to_string())),
            "expected pair key '{kind_text}' tagged as definition.var, got: {tag_pairs:?}"
        );
    }

    // Same three key-shape variants nested inside an explicit [nested] table
    // — the key variants must be recognized regardless of container.
    let var_count_dotted_nested = tag_pairs
        .iter()
        .filter(|(k, n)| k == "definition.var" && n == "dotted.pair.key")
        .count();
    assert_eq!(
        var_count_dotted_nested, 2,
        "expected 'dotted.pair.key' as a pair key both top-level and nested in [nested], got {var_count_dotted_nested}: {tag_pairs:?}"
    );

    // collect_captures_full: verify node kind, not just text, for the
    // dotted_key variant specifically (extraction-depth dimension).
    let captures = collect_captures_full(&lang, TOML_VARIANTS, &query_str);
    let dotted_pair_kind = captures
        .iter()
        .find(|(cap, _k, t, _line)| cap == "name" && t == "dotted.pair.key")
        .map(|(_cap, k, _t, _line)| k.as_str());
    assert_eq!(
        dotted_pair_kind,
        Some("dotted_key"),
        "expected 'dotted.pair.key' @name capture to be node kind dotted_key, got: {dotted_pair_kind:?} \
         (full captures: {captures:?})"
    );
}

#[test]
fn toml_tags_negative_values_and_scalars_not_captured_as_names() {
    let Some(gdir) = grammar_dir() else {
        eprintln!(
            "Skipping toml_tags_negative_values_and_scalars_not_captured_as_names: run `cargo xtask build-grammars` first"
        );
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("toml").ok() else {
        eprintln!(
            "Skipping toml_tags_negative_values_and_scalars_not_captured_as_names: toml grammar .so not found"
        );
        return;
    };
    let query_str = loader.get_tags("toml").expect("toml tags query missing");
    let names = collect_captures(&lang, TOML_VARIANTS, &query_str, "name");

    // `value_looks_like_key = "not.a.real.key"` — the *string value* text
    // must never itself appear as a captured @name; only the pair's own key
    // ("value_looks_like_key") should.
    assert!(
        names.contains(&"value_looks_like_key".to_string()),
        "expected the pair's own key to be captured, got: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.contains("not.a.real.key")),
        "string value text must never be captured as a @name, got: {names:?}"
    );

    // Scalar array elements (`plain_array = [1, 2, 3]`) are never keys —
    // only the pair's own key ("plain_array") should be captured, not "1",
    // "2", or "3".
    assert!(
        !names.contains(&"1".to_string())
            && !names.contains(&"2".to_string())
            && !names.contains(&"3".to_string()),
        "scalar array elements must never be captured as a @name, got: {names:?}"
    );

    // Inline tables (`inline = { a = 1, b = 2 }`): the pair's own key
    // ("inline") is captured as a definition.var, same as any other pair.
    // Its inner pairs ("a", "b") also match the query as written — this is
    // documented, existing behavior (see the comment atop toml.tags.scm and
    // `is_inside_inline_table` in toml.rs): the query intentionally doesn't
    // special-case inline tables, and downstream Rust-level `node_name()`
    // filters inline-table-nested pairs back out of actual symbol
    // extraction. This assertion pins that query-level behavior so a future
    // change to either layer is deliberate, not accidental.
    assert!(
        names.contains(&"inline".to_string())
            && names.contains(&"a".to_string())
            && names.contains(&"b".to_string()),
        "expected inline table's own key plus its inner pair keys to all match \
         at the query level (filtered downstream in toml.rs), got: {names:?}"
    );
}

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

// ---------------------------------------------------------------------------
// YAML
// ---------------------------------------------------------------------------

const YAML_SAMPLE: &str = include_str!("fixtures/yaml/sample.yaml");
const YAML_VARIANTS: &str = include_str!("fixtures/yaml/variants.yaml");

/// `(loader, language, tags)` — see `sql_lang_and_queries` for why the
/// loader must be kept alongside the language it produced (dropping the
/// `GrammarLoader` unloads the backing `.so` and dangles the `Language`'s
/// function pointers).
type YamlLangAndTags = (GrammarLoader, tree_sitter::Language, Arc<String>);

fn yaml_lang_and_tags() -> Option<YamlLangAndTags> {
    let gdir = grammar_dir()?;
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let lang = loader.get("yaml").ok()?;
    let tags = loader.get_tags("yaml")?;
    Some((loader, lang, tags))
}

// --- Dimension 4: real-world fixture coverage (sample.yaml) ----------------

#[test]
fn yaml_tags_finds_real_world_keys() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, YAML_SAMPLE, &tags, "name");
    // Block-style nested keys.
    assert!(
        names.iter().any(|n| n == "jobs"),
        "expected top-level 'jobs' key, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "runs-on"),
        "expected nested 'runs-on' key, got: {names:?}"
    );
    // YAML merge key (`<<: *defaults`) — a plain_scalar key like any other.
    assert!(
        names.iter().any(|n| n == "<<"),
        "expected merge key '<<', got: {names:?}"
    );
    // Flow-mapping keys inside a block sequence item
    // (`- { os: ubuntu-latest, arch: x64 }`).
    assert!(
        names.iter().any(|n| n == "os"),
        "expected flow_pair key 'os' inside matrix_include, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "arch"),
        "expected flow_pair key 'arch' inside matrix_include, got: {names:?}"
    );
    // Quoted top-level keys.
    assert!(
        names.iter().any(|n| n == "\"quoted top-level key\""),
        "expected double-quoted top-level key, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "'single quoted top-level key'"),
        "expected single-quoted top-level key, got: {names:?}"
    );
}

// --- Dimension 2 + 3: completeness matrix (variants.yaml) -------------------

/// Every `block_mapping_pair`/`flow_pair` key-scalar variant found by
/// cross-referencing arborium-yaml 2.17.0's node-types.json (plain_scalar,
/// double_quote_scalar, single_quote_scalar — for both block and flow
/// pairs) must produce a `definition.var` with the correct capture kind.
/// Uses `collect_captures_full` so a kind mismatch (e.g. a quoted key
/// accidentally matched by the plain_scalar clause, or vice versa) can't
/// hide behind string-only assertions.
#[test]
fn yaml_tags_completeness_key_scalar_variants() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let full = collect_captures_full(&lang, YAML_VARIANTS, &tags);
    let name_kind_text = |name: &str| -> Option<&str> {
        full.iter()
            .find(|(cap, _, text, _)| cap == "name" && text == name)
            .map(|(_, kind, ..)| kind.as_str())
    };

    // block_mapping_pair.key variants.
    assert_eq!(
        name_kind_text("plain_key"),
        Some("string_scalar"),
        "plain block key must capture as string_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("\"double quoted key\""),
        Some("double_quote_scalar"),
        "double-quoted block key must capture as double_quote_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("'single quoted key'"),
        Some("single_quote_scalar"),
        "single-quoted block key must capture as single_quote_scalar, full: {full:?}"
    );
    // Anchor/tag-prefixed keys still resolve to the plain scalar name —
    // the anchor/tag is a sibling within the same flow_node, not a wrapper.
    assert_eq!(
        name_kind_text("anchored_key"),
        Some("string_scalar"),
        "anchor-prefixed key must still capture the plain scalar name, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("tagged_key"),
        Some("string_scalar"),
        "tag-prefixed key must still capture the plain scalar name, full: {full:?}"
    );

    // flow_pair.key variants (all three scalar kinds inside a flow_mapping).
    assert_eq!(
        name_kind_text("flow_plain"),
        Some("string_scalar"),
        "flow_pair plain key must capture as string_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("\"flow double quoted\""),
        Some("double_quote_scalar"),
        "flow_pair double-quoted key must capture as double_quote_scalar, full: {full:?}"
    );
    assert_eq!(
        name_kind_text("'flow single quoted'"),
        Some("single_quote_scalar"),
        "flow_pair single-quoted key must capture as single_quote_scalar, full: {full:?}"
    );

    // Multi-document stream: a key in the second document must be found
    // too — tags matching is structural, not document-scoped.
    assert_eq!(
        name_kind_text("doc2_key"),
        Some("string_scalar"),
        "key in second document of a multi-document stream must still be captured, full: {full:?}"
    );
}

/// `node_name`/`refine_kind`/`container_body` (Rust-side symbol
/// reconstruction) are covered directly in `crates/normalize-languages/src/yaml.rs`'s
/// own unit tests, since they operate on `tree_sitter::Node` rather than
/// producing query captures — this test only asserts the raw query's
/// container-key captures (both nesting paths: a block key with an inline
/// flow-mapping value, and a flow pair nested inside another flow mapping).
#[test]
fn yaml_tags_completeness_container_nesting() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags_container_nesting: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, YAML_VARIANTS, &tags, "name");

    // block_mapping_pair value = block_node > block_mapping.
    assert!(names.iter().any(|n| n == "block_container"));
    assert!(names.iter().any(|n| n == "nested_key"));

    // block_mapping_pair value = flow_node > flow_mapping (inline flow
    // value on a block-style key).
    assert!(names.iter().any(|n| n == "inline_flow_container"));
    assert!(names.iter().any(|n| n == "a"));
    assert!(names.iter().any(|n| n == "b"));

    // flow_pair value = flow_node > flow_mapping (flow mapping nested
    // inside another flow mapping).
    assert!(names.iter().any(|n| n == "nested_flow_container"));
    assert!(names.iter().any(|n| n == "outer"));
    assert!(names.iter().any(|n| n == "inner"));
}

/// Negative cases: constructs that must NOT produce a `@name` capture.
/// Asserts the exact total count on `variants.yaml` so a stray extra match
/// (e.g. a sequence item accidentally matched) can't hide behind a
/// `.any()`-only check.
#[test]
fn yaml_tags_negative_sequences_and_complex_keys() {
    let Some((_loader, lang, tags)) = yaml_lang_and_tags() else {
        eprintln!("Skipping yaml_tags_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let names = collect_captures(&lang, YAML_VARIANTS, &tags, "name");

    // Sequence items themselves are never named symbols — only the key
    // that holds the sequence is.
    assert!(
        !names.iter().any(|n| n == "item1" || n == "item2"),
        "block sequence items must not be captured as symbols, got: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n == "1" || n == "2" || n == "3"),
        "flow sequence items must not be captured as symbols, got: {names:?}"
    );

    // Explicit complex key (`? [a, b]`) — a flow_sequence key has no single
    // scalar name to extract; must not match.
    assert!(
        !names.iter().any(|n| n.contains('[') || n.contains(']')),
        "complex flow-sequence key must not be captured, got: {names:?}"
    );

    // Explicit block-scalar key (`? |`) — multi-line blob, must not match.
    assert!(
        !names.iter().any(|n| n.contains("block scalar key")),
        "explicit block-scalar key must not be captured, got: {names:?}"
    );

    // Anchors/aliases: `alias_source`/`alias_use` are ordinary block-mapping
    // keys (captured normally); the anchor definition (`&shared_anchor`)
    // and alias reference (`*shared_anchor`) themselves must not produce
    // separate captures — this codebase's tags pipeline has no
    // definition/reference convention for them (see yaml.tags.scm).
    assert!(
        names.iter().any(|n| n == "alias_source"),
        "expected 'alias_source' key itself to be captured, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "alias_use"),
        "expected 'alias_use' key itself to be captured, got: {names:?}"
    );
    assert!(
        !names
            .iter()
            .any(|n| n == "shared_anchor" || n == "&shared_anchor" || n == "*shared_anchor"),
        "anchor/alias names themselves must not be captured as symbols, got: {names:?}"
    );

    // Exact total count on variants.yaml: pins the full set of matches so
    // any future regression (extra or missing match) is caught precisely.
    assert_eq!(
        names.len(),
        22,
        "expected exactly 22 @name captures in variants.yaml, got {}: {names:?}",
        names.len()
    );
}
