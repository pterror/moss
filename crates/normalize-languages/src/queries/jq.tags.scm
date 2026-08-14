; jq tags query
;
; jq function definitions: def name(params): body;
; The first identifier child of funcdef is the function name (funcdefargs'
; own identifier/variable parameter names are nested one level deeper, in
; funcdefargs, so this pattern's lack of a field constraint doesn't
; over-match them — verified via `normalize syntax ast` against a
; multi-parameter funcdef).

(funcdef
  (identifier) @name) @definition.function

; ---------------------------------------------------------------------------
; Call references
; ---------------------------------------------------------------------------
; Mirrors jq.calls.scm's `(funcname) @call` coverage — this was ported late
; (jq.calls.scm had it, jq.tags.scm didn't), the same class of gap
; documented as bug #5 in docs/query-testing-methodology.md's Rust example.
;
; jq's grammar has no wrapping "call expression" node (a call and its
; arglist are flattened directly into the enclosing `query` node, same
; flattening as if/reduce/try — see jq.cfg.scm's header comment), so
; `@reference.call` is the enclosing `query` node itself, anchored to
; require `funcname` as its first child so it only matches queries that
; *are* a call (not some unrelated query that merely contains a funcname
; deeper inside, e.g. as a call argument).
(query
  . (funcname) @name) @reference.call
