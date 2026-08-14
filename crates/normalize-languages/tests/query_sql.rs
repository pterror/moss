//! Query fixture tests for sql.
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
