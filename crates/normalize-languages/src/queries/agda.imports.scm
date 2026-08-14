; Agda imports query
; @import       — the entire with/import/open statement (for line number)
; @import.path  — the module path capture
; @import.glob  — open import marker (wildcard = opens entire namespace)
;
; `open` has two distinct real shapes (verified via `normalize syntax ast`):
;   - `open import Data.List` — combined open+import, parses as
;     `open > import > module_name`.
;   - `open Data.List` — glob-opening an already-`import`ed module, parses
;     as `open > module_name` directly (no nested `import`).
; An earlier version unconditionally captured `(open) @import.path @import`
; for BOTH shapes, which double-counted every `open import` statement: the
; nested `(import)` pattern below already matches its `import` child
; (queries match by node type regardless of parent), so the combined form
; produced two overlapping @import/@import.path entries for one statement
; ("import Data.List" from the inner match, "open import Data.List" from
; the outer). `open import` is the more common real-world idiom (more so
; than bare `import`), so this was a systematic, not rare, duplicate.

; import Data.List
(import) @import.path @import

; open import Data.List — the @import.path/@import capture for this
; statement already comes from the `(import)` pattern above (it matches the
; nested `import` child too); this pattern adds only the @import.glob
; marker on the outer `open` node, without re-emitting @import.path/@import.
(open
  (import)) @import.glob

; open Data.List — glob-opening an already-imported module; `open` is the
; whole statement here (no nested `import` child), so it needs the full
; @import.path/@import/@import.glob triplet.
(open
  (module_name) @import.path) @import.glob @import
