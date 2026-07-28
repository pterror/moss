; SQL tags — definitions for functions, tables, views, schemas, types, and
; other named database objects, plus function-call references.

; NOTE (bug fix): `create_function`'s own name (`object_reference`) must be
; anchored to the position immediately after `keyword_function` — an
; unanchored `(create_function (object_reference) @name)` also matches a
; second, spurious `object_reference` whenever the function's RETURNS clause
; names a custom (schema-qualified) type, e.g. `RETURNS inventory.status`
; (confirmed via real parse: `CREATE FUNCTION f(...) RETURNS inventory.status
; AS ...` produced two @name captures — the function itself, and the return
; type — both tagged @definition.function).
(create_function
  (keyword_function) . (object_reference) @name) @definition.function

(create_table
  (object_reference) @name) @definition.class

(create_view
  (object_reference) @name) @definition.class

; NOTE (bug fix): same anchoring issue as create_function — an unanchored
; `(create_schema (identifier) @name)` also matches the role name in
; `CREATE SCHEMA foo AUTHORIZATION bar;`, tagging `bar` as a second, spurious
; schema definition (confirmed via real parse).
(create_schema
  (keyword_schema) . (identifier) @name) @definition.module

(create_type
  (object_reference) @name) @definition.type

; CREATE INDEX — the index's own name is a bare `identifier` (not
; `object_reference`); the indexed table appears separately as
; `object_reference` and must not be captured here (verified: only one
; `identifier` child exists per create_index, so no anchor needed).
(create_index
  (identifier) @name) @definition.var

; CREATE TRIGGER — anchored to the first `object_reference` after
; `keyword_trigger`: an unanchored pattern also matches the trigger's target
; table and the function it executes, both also represented as
; `object_reference` children (confirmed via real parse).
(create_trigger
  (keyword_trigger) . (object_reference) @name) @definition.function

; CREATE SEQUENCE — anchored to the first `object_reference` after
; `keyword_sequence`, mirroring create_function/create_trigger: a sequence's
; `AS <custom_type>` clause can also introduce an `object_reference`.
(create_sequence
  (keyword_sequence) . (object_reference) @name) @definition.var

(create_materialized_view
  (object_reference) @name) @definition.class

; Function/procedure call references. Mirrors sql.calls.scm's anchoring fix:
; `invocation`'s `unit` field (used by `EXTRACT(field FROM source)`, e.g. the
; `YEAR` in `EXTRACT(YEAR FROM x)`) is also an unconstrained `object_reference`
; child, so the call name must be anchored to the first child.
(invocation
  . (object_reference
      name: (identifier) @name)) @reference.call
