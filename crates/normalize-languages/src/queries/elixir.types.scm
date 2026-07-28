; Type reference query for Elixir
;
; Elixir has no separate type/value namespace: module names (aliases) double
; as the closest thing Elixir has to "type" names (a struct's type IS its
; defining module's name, e.g. `%Stack{}`'s type is `Stack`). This query
; captures every `(alias)` node in the file — module references in
; `defmodule`, `@spec`/`@type`/`@behaviour` annotations, `defimpl ... for:`,
; ordinary remote calls (`Enum.map`), etc. — all uniformly, since `alias` is
; a single leaf node type with no field variants to distinguish "this alias
; is inside a typespec" from "this alias is a module reference elsewhere"
; without ancestor-aware predicates this query engine does not support (only
; `match?`/`not-match?`/`eq?`/`not-eq?`, verified against
; `query_predicates.rs`). This is intentionally broad rather than narrowly
; scoped to `@spec`/`@type` bodies — verified this is the existing, tested
; contract (`elixir_types_finds_module_aliases` already asserts a
; `defmodule` name like "MathUtils" is an acceptable @type.reference), and
; there is currently no downstream Rust consumer imposing a stricter
; contract to design against.
(alias) @type.reference
