; Idris tags query

; Function signatures: distance : Point -> Point -> Double
(signature
  name: (loname) @name) @definition.function

; Data types: data Shape = ...
(data
  name: (data_name) @name) @definition.class

; Records: record Point where ...
(record
  name: (record_name) @name) @definition.class

; Interfaces
(interface
  (interface_head
    name: (interface_name) @name)) @definition.interface

; Interface method signatures: `show : a -> String` inside `interface Show a where ...`.
; Verified via `normalize syntax ast`/`normalize syntax query` on a probe file
; that `interface_body` wraps member `signature` nodes directly (same
; `(signature name: (loname))` shape as top-level signatures) — previously
; completely untagged, so every interface's method list was invisible.
(interface_body
  (signature
    name: (loname) @name)) @definition.method

; Record constructors: `record Point where constructor MkPoint ...`.
; `record_body` wraps a `constructor` node whose `name` field is the
; constructor's caname (confirmed via `normalize syntax query` against the
; existing `sample.idr` fixture's `constructor MkPoint` line, which
; previously produced zero tag matches despite being the single most
; fundamental symbol a record type introduces — the value constructor used
; to build every instance of the record).
(record_body
  (constructor
    name: (caname) @name)) @definition.function

; GADT-style data constructors: `data Expr : Type where Lit : Int -> Expr`.
; `data_body` wraps member `signature` nodes for the `where`-block form
; (confirmed via probe file + `normalize syntax query`); the flat
; `data Shape = Circle Double | ...` form does NOT produce this shape — its
; constructors are bare `exp_name`/`caname` siblings indistinguishable from
; any other type-level application in this grammar's CST (same structural
; ambiguity documented in `haskell.tags.scm` for `data (:+:) a b = L a | R b`),
; so that form is intentionally left untagged rather than fabricating a
; disambiguation the grammar doesn't provide.
(data_body
  (signature
    name: (caname) @name)) @definition.function
