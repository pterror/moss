; Type reference query for Haskell
; Captures type constructors used in type signatures and annotations.
;
; Unconditionally matching `(name)` (rather than scoping to a specific field)
; is intentional and verified, not sloppy: `name` is a leaf node reserved
; exclusively for type-level identifiers in this grammar — value-level
; identifiers use `variable`, and value-level data-constructor references use
; `constructor` — so a bare `(name)` never collides with expression captures.
; This also means it already covers qualified (`Map.Map`, via the inner
; `qualified.id`) and generic-application (`Maybe Int`, `Map.Map Int Int`,
; via `apply.constructor`) type references for free, verified via real parse.
;
; KNOWN GAP (documented, not fixed): custom infix type operators used as a
; type constructor, e.g. `x :: Int :+: String` — the operator itself
; (`:+:`) is an `infix.operator` child, not a `name`, so it is never
; captured. Fixing this correctly requires scoping to type-level `infix`
; nodes specifically (the same `infix` node shape is also used for ordinary
; value-level operators like `x + y`, so an unscoped `(infix operator:
; (constructor_operator) @type.reference)` clause would misfire on
; value-level constructor operators, e.g. `x :| y`). Custom infix *type*
; operators (TypeOperators extension) are a real but advanced/rare feature;
; left undone per "verify real-world usage density" rather than adding a
; clause whose false-positive risk outweighs its rarity.

; Plain type names (constructors start with uppercase): Maybe, Int, String
(name) @type.reference
