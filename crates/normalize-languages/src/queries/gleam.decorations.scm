(module_comment) @decoration

(statement_comment) @decoration

(comment) @decoration

; Attributes: @deprecated("..."), @external(erlang, "mod", "fn"),
; @target(erlang)/@target(javascript) conditional-compilation markers.
; Gleam's equivalent of Rust's #[attr]/Python's @decorator — verified via
; `normalize syntax query` against `@deprecated(...)`/`@external(...)`
; probes; both parse cleanly as `attribute` nodes.
(attribute) @decoration
