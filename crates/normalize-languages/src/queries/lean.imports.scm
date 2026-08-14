; Lean 4 imports query
; @import       — the entire import statement (for line number)
; @import.path  — the module path

; import Mathlib.Algebra.Group
;
; Verified against arborium-lean 2.17.0's node-types.json and real parse
; output. `import` has a `module:` field pointing to the actual module-path
; identifier; the prior version captured the whole `(import)` node (which
; includes the literal "import " keyword text) as `@import.path`, so every
; extracted path was contaminated with a leading "import " — confirmed via
; `normalize syntax query`: `@import.path` returned "import
; Mathlib.Data.List.Basic" instead of "Mathlib.Data.List.Basic".
(import
  module: (identifier) @import.path) @import
