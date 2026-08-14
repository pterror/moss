; Type reference query for Julia
; Captures type identifiers used in :: annotations and parametric types.
;
; typed_expression has no named fields (node-types.json: empty fields
; object); its second positional child is whichever expression node spells
; the type. Cross-referenced against node-types.json and verified per-variant
; via `normalize syntax ast`/`syntax query`: identifier (already handled),
; field_expression (qualified type: `z::Base.Int`), and parametrized_type_expression
; (`a::Dict{String,Any}`, already separately covered by the standalone
; parametrized_type_expression pattern below since that pattern floats
; anywhere in the tree) were the three variants found. The qualified-type
; case was completely unhandled — `x::Base.Int`, `x::LinearAlgebra.Diagonal`
; produced zero @type.reference captures.

; Type annotations: x::Int, foo(x::Float64)::String
(typed_expression
  . _ @_value
  (identifier) @type.reference)

; Qualified type annotations: x::Base.Int, x::LinearAlgebra.Diagonal —
; capture the leaf type name, matching rust.types.scm's "capture the leaf
; name" convention for scoped_type_identifier.
(typed_expression
  . _ @_value
  (field_expression
    (_) @_qualifier
    (identifier) @type.reference))

; Parametrized types: Vector{Int}, Dict{String, Any} — the generic name
(parametrized_type_expression
  (identifier) @type.reference)

; Parametrized types' type arguments: the Int/Any/... inside {...}. Verified
; via `normalize syntax ast` that Vector{Int}/Dict{String,Any} wrap their
; parameters in a curly_expression child of parametrized_type_expression.
(curly_expression
  (identifier) @type.reference)

; Supertype / type-bound references using `<:`: `struct Circle <: Shape`
; (type_head wraps a binary_expression) and `where T <: Shape` (where_expression
; wraps a binary_expression). Both shapes parse `<:` as a plain binary_expression
; with an `operator` child carrying the token text — verified via `normalize
; syntax ast`/`syntax query` against probe files for both struct-supertype and
; where-clause-bound forms.
(binary_expression
  . (identifier)
  (operator) @_op
  . (identifier) @type.reference
  (#eq? @_op "<:"))
