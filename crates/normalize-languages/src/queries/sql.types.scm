; SQL types query
; @type — type references in column definitions, CREATE TYPE statements,
; function parameters, DECLAREd procedural variables, CAST expressions, ALTER
; COLUMN ... TYPE, and CREATE SEQUENCE ... AS.
;
; SQL has static type information in DDL statements. Column definitions
; carry an explicit data type (INTEGER, VARCHAR, TEXT, etc.), and CREATE TYPE
; defines named composite/enum/domain types.

; Column type in CREATE TABLE: col_name INTEGER
(column_definition
  type: (_) @type)

; Column type via custom_type (user-defined type reference): col_name my_type
(column_definition
  custom_type: (object_reference) @type)

; Column type in ALTER TABLE ... ALTER COLUMN ... TYPE: same field shape as
; column_definition's `type` field, so a single wildcard clause covers every
; builtin variant.
(alter_column
  type: (_) @type)

; NOTE: unlike column_definition, `function_argument` and `function_declaration`
; (the node used for DECLAREd procedural variables, e.g. `current_stock
; INTEGER;` inside a function body) have no `type` field for builtin types —
; the type is a bare positional child, and so is the parameter name/mode
; keywords (IN/OUT/VARIADIC) and any DEFAULT value. Positional adjacency
; (`(identifier) . (_) @type`) was tried and rejected: it misses unnamed
; parameters (`CREATE FUNCTION f(INTEGER, TEXT)`, valid SQL — confirmed via
; real parse) and misidentifies mode keywords/default values as the type when
; a name or mode keyword precedes the actual type. Enumerating every concrete
; builtin type node kind (mirrors column_definition's own `type` field variant
; list) is the only construct verified correct for named, unnamed, and
; mode-qualified parameters alike.

; Function/procedure parameter type (builtin primitives)
(function_argument
  [
    (array_size_definition) (bigint) (binary) (bit) (char) (datetimeoffset)
    (decimal) (double) (enum) (float) (int) (keyword_bigserial) (keyword_boolean)
    (keyword_box2d) (keyword_box3d) (keyword_bytea) (keyword_date) (keyword_datetime)
    (keyword_datetime2) (keyword_geography) (keyword_geometry) (keyword_image)
    (keyword_inet) (keyword_interval) (keyword_json) (keyword_jsonb) (keyword_money)
    (keyword_name) (keyword_oid) (keyword_regclass) (keyword_regnamespace)
    (keyword_regproc) (keyword_regtype) (keyword_serial) (keyword_smalldatetime)
    (keyword_smallmoney) (keyword_smallserial) (keyword_string) (keyword_text)
    (keyword_timestamptz) (keyword_uuid) (keyword_xml) (mediumint) (nchar) (numeric)
    (nvarchar) (smallint) (time) (timestamp) (tinyint) (varbinary) (varchar)
  ] @type)

; Function/procedure parameter type (user-defined type reference)
(function_argument
  custom_type: (object_reference) @type)

; DECLAREd procedural variable type (builtin primitives) — e.g.
; `DECLARE current_stock INTEGER;` inside a function body.
(function_declaration
  [
    (array_size_definition) (bigint) (binary) (bit) (char) (datetimeoffset)
    (decimal) (double) (enum) (float) (int) (keyword_bigserial) (keyword_boolean)
    (keyword_box2d) (keyword_box3d) (keyword_bytea) (keyword_date) (keyword_datetime)
    (keyword_datetime2) (keyword_geography) (keyword_geometry) (keyword_image)
    (keyword_inet) (keyword_interval) (keyword_json) (keyword_jsonb) (keyword_money)
    (keyword_name) (keyword_oid) (keyword_regclass) (keyword_regnamespace)
    (keyword_regproc) (keyword_regtype) (keyword_serial) (keyword_smalldatetime)
    (keyword_smallmoney) (keyword_smallserial) (keyword_string) (keyword_text)
    (keyword_timestamptz) (keyword_uuid) (keyword_xml) (mediumint) (nchar) (numeric)
    (nvarchar) (smallint) (time) (timestamp) (tinyint) (varbinary) (varchar)
  ] @type)

; DECLAREd procedural variable type (user-defined type reference)
(function_declaration
  custom_type: (object_reference) @type)

; CAST(expr AS type) — the target type follows `keyword_as` positionally
; (verified: `cast` has no field for it, but `keyword_as` is always the
; immediately preceding sibling, so anchoring there is unambiguous and works
; uniformly for builtin and user-defined target types alike).
(cast
  (keyword_as) . (_) @type)

; CREATE SEQUENCE ... AS <type> — mirrors CAST's anchoring: `create_sequence`
; has no field for its own builtin base type, but it always immediately
; follows `keyword_as`.
(create_sequence
  (keyword_as) . (_) @type)
