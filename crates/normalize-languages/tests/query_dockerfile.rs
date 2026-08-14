//! Query fixture tests for dockerfile.
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
