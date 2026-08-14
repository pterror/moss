; Agda type reference query
; Captures the type expression of a type signature.
;
; The original `(type_signature (expr) @type.reference)` pattern was dead
; code: `type_signature` is declared in node-types.json but never actually
; produced by the grammar for real source (verified: `(type_signature) @x`
; matches 0 times across sample.agda and several probe files). In practice
; Agda has no dedicated "type signature" node at all:
;
;   - A top-level/local signature (`classify : Int -> String`, and a data
;     constructor's type like `Circle : Shape`) parses as an ordinary
;     `function` node — the SAME node type used for a defining equation
;     (`classify n = ...`) — distinguished only by the `rhs`'s leading
;     anonymous token being `:` instead of `=`.
;   - A record field signature (`field x : Int`) parses as `signature`
;     containing a `field_name` leaf followed by `:` and an `expr`, not a
;     nested `signature` wrapping a `function_name` (the shape the old
;     agda.tags.scm pattern for this case incorrectly assumed too).
;
; `qid` (the leaf identifier node) is used for both type-level and
; value-level names in this grammar — unlike Haskell's grammar, which
; reserves a dedicated `name` node for type-level identifiers — so an
; unscoped `(qid) @type.reference` would capture every value-level
; identifier too. Capturing the whole `expr` (as the original pattern's
; intent already was) keeps the query scoped to real type-signature
; positions.

; Top-level or local function/data-constructor signature.
(function
  (lhs
    (function_name))
  (rhs
    ":"
    .
    (expr) @type.reference))

; Record field signature: `field x : Int` inside a `record ... where` block.
(signature
  (field_name)
  .
  ":"
  .
  (expr) @type.reference)
