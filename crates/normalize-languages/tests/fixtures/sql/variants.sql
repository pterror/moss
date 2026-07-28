-- Completeness matrix for SQL query files (tags/calls/complexity/types).
-- One small, commented construct per node-type variant exercised by the
-- .scm queries, plus a NEGATIVE section of near-miss constructs that must
-- NOT match. See docs/query-testing-methodology.md.

-- === tags.scm: @definition.* variants ===

-- @definition.function (create_function, anchored to keyword_function)
CREATE FUNCTION variants.plain_fn(x INTEGER) RETURNS INTEGER AS $$
BEGIN
    RETURN x;
END;
$$ LANGUAGE plpgsql;

-- @definition.function via CREATE OR REPLACE FUNCTION with a custom
-- (schema-qualified) RETURNS type — regression fixture for the anchoring
-- bug: an unanchored query also captured the return type as a second,
-- spurious @definition.function.
CREATE OR REPLACE FUNCTION variants.custom_return_fn(x INTEGER)
RETURNS variants.status AS $$
BEGIN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- @definition.class (create_table)
CREATE TABLE variants.widgets (
    id INTEGER,
    label VARCHAR(50)
);

-- @definition.class (create_view)
CREATE VIEW variants.widget_labels AS
    SELECT label FROM variants.widgets;

-- @definition.class (create_materialized_view)
CREATE MATERIALIZED VIEW variants.widget_count AS
    SELECT COUNT(*) AS n FROM variants.widgets;

-- @definition.module (create_schema, anchored to keyword_schema)
CREATE SCHEMA variants;

-- @definition.module via CREATE SCHEMA ... AUTHORIZATION — regression
-- fixture for the anchoring bug: an unanchored query also captured the
-- role name as a second, spurious @definition.module.
CREATE SCHEMA variants_auth AUTHORIZATION some_role;

-- @definition.type (create_type)
CREATE TYPE variants.status AS ENUM ('active', 'inactive');

-- @definition.var (create_index)
CREATE INDEX idx_widgets_label ON variants.widgets (label);

-- @definition.function (create_trigger, anchored to keyword_trigger)
CREATE TRIGGER widgets_touch
AFTER INSERT ON variants.widgets
FOR EACH ROW
EXECUTE FUNCTION variants.notify_widget();

-- @definition.var (create_sequence)
CREATE SEQUENCE variants.widget_seq AS BIGINT INCREMENT 1 START 1;

-- @reference.call (invocation, anchored to first object_reference child)
SELECT COUNT(*) FROM variants.widgets;

-- === calls.scm / tags.scm @reference.call: invocation variants ===

-- Plain function call
SELECT NOW();

-- Schema-qualified function call
SELECT variants.notify_widget();

-- Aggregate with DISTINCT
SELECT COUNT(DISTINCT label) FROM variants.widgets;

-- Window function (invocation nested inside window_function)
SELECT ROW_NUMBER() OVER (ORDER BY id) FROM variants.widgets;

-- === types.scm: @type variants ===

-- column_definition type: (_) — builtin primitive
CREATE TABLE variants.typed_columns (
    a INTEGER,
    b VARCHAR(10),
    c NUMERIC(5, 2)
);

-- column_definition custom_type: (object_reference)
CREATE TABLE variants.custom_typed_columns (
    status variants.status
);

-- alter_column type: (_)
ALTER TABLE variants.typed_columns ALTER COLUMN c TYPE NUMERIC(10, 4);

-- function_argument builtin type, named parameter
CREATE FUNCTION variants.named_param_fn(qty INTEGER) RETURNS INTEGER AS $$
BEGIN
    RETURN qty;
END;
$$ LANGUAGE plpgsql;

-- function_argument builtin type, unnamed parameter — regression fixture:
-- the type-adjacency approach (identifier . type) misses this shape entirely
-- since there is no identifier at all.
CREATE FUNCTION variants.unnamed_param_fn(INTEGER, TEXT) RETURNS INTEGER AS $$
BEGIN
    RETURN 1;
END;
$$ LANGUAGE plpgsql;

-- function_argument custom_type: (object_reference)
CREATE FUNCTION variants.custom_param_fn(s variants.status) RETURNS INTEGER AS $$
BEGIN
    RETURN 1;
END;
$$ LANGUAGE plpgsql;

-- function_declaration builtin type (DECLAREd procedural variable)
CREATE FUNCTION variants.declares_var() RETURNS INTEGER AS $$
DECLARE
    counter INTEGER;
BEGIN
    RETURN counter;
END;
$$ LANGUAGE plpgsql;

-- function_declaration custom_type: (object_reference)
CREATE FUNCTION variants.declares_custom_var() RETURNS INTEGER AS $$
DECLARE
    s variants.status;
BEGIN
    RETURN 1;
END;
$$ LANGUAGE plpgsql;

-- cast (keyword_as) . type
SELECT CAST(id AS TEXT) FROM variants.widgets;

-- create_sequence (keyword_as) . type
CREATE SEQUENCE variants.typed_seq AS INTEGER;

-- === complexity.scm variants ===

-- when_clause (CASE WHEN branch) @complexity
SELECT CASE WHEN label = 'a' THEN 1 ELSE 0 END FROM variants.widgets;

-- join @complexity
SELECT * FROM variants.widgets w JOIN variants.typed_columns t ON w.id = t.a;

-- where @complexity
SELECT * FROM variants.widgets WHERE label = 'a';

-- having @complexity
SELECT label, COUNT(*) FROM variants.widgets GROUP BY label HAVING COUNT(*) > 1;

-- set_operation @complexity (UNION)
SELECT id FROM variants.widgets
UNION
SELECT a FROM variants.typed_columns;

-- exists @complexity
SELECT * FROM variants.widgets WHERE EXISTS (SELECT 1 FROM variants.typed_columns);

-- select @nesting
SELECT id FROM variants.widgets;

-- subquery @nesting
SELECT * FROM (SELECT id FROM variants.widgets) AS sub;

-- cte @nesting
WITH labeled AS (SELECT label FROM variants.widgets)
SELECT * FROM labeled;

-- === NEGATIVE: constructs that must NOT match ===

-- EXTRACT's date-part keyword (YEAR) is invocation's `unit` field, not a
-- call — must NOT appear as a @call/@reference.call capture. (The function
-- name EXTRACT itself IS a legitimate call and DOES match.)
SELECT EXTRACT(YEAR FROM ordered_at) FROM variants.typed_columns;

-- A bare table reference in FROM is a `relation`/`object_reference`, never
-- wrapped in `invocation` — must NOT match calls/@reference.call.
SELECT * FROM variants.widgets;

-- CREATE TABLE ... AS SELECT ... FROM other_table: the FROM target is nested
-- inside `create_query`/`select`/`from`, not a direct child of
-- `create_table` — must produce exactly one @definition.class (the new
-- table), not a second one for the source table.
CREATE TABLE variants.widgets_copy AS
    SELECT * FROM variants.widgets;

-- Table-level FOREIGN KEY constraint: the REFERENCES target is nested
-- inside `column_definitions`/`constraint`, not a direct child of
-- `create_table` — must not produce a second @definition.class.
--
-- NOTE (grammar limitation): a *named* `CONSTRAINT fk_name FOREIGN KEY ...`
-- form produces an ERROR node in arborium-sql 2.17.0 (confirmed via real
-- parse) — the unnamed form below is the one the grammar actually supports
-- cleanly.
CREATE TABLE variants.widget_refs (
    id INTEGER,
    widget_id INTEGER,
    FOREIGN KEY (widget_id) REFERENCES variants.widgets(id)
);
