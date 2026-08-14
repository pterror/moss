; Agda calls query
; @call — call/application expression nodes
; @call.qualifier — module qualifier for qualified calls
;
; Agda uses juxtaposition for function application (like Haskell): `f x y`
; is an `expr` node whose children are a flat sequence of `atom`s, the first
; being the applied function. Module application uses `module_application`.
;
; KNOWN GRAMMAR LIMITATION (documented, not silently dropped): Agda's CST
; does not structurally distinguish function application from infix-operator
; use or bare type/arrow expressions — `f x`, `n + n`, and `Int -> String`
; are ALL represented identically as a flat `expr` of `atom` children;
; resolving which identifiers are "operators" requires the file's
; `infix`/`infixl`/`infixr` fixity declarations, which a single positional
; tree-sitter pattern cannot consult. Verified via `normalize syntax ast`/
; `normalize syntax query --show-source` against sample.agda and variants.agda:
;   - `n + n` produces a false-positive @call = "n" (the left operand of a
;     user-defined infix operator) — unavoidable without fixity resolution.
;   - `if c then t else f` produces a (structurally accurate) @call = "if":
;     `if` is an ordinary identifier applied like any function in this
;     grammar (there is no dedicated if/then/else syntax node — see
;     agda.cfg.scm), so this is not a bug.
;
; Two earlier, more permissive designs were tried and rejected because they
; produced *systematic*, not just occasional, false positives:
;   1. An unscoped `(expr . (atom . (qid) @call))` matched the type
;      expression of every `:` signature too (a type name like `Int` is
;      itself a single-atom `expr`, structurally identical to a bare-name
;      call target) — every typed function's signature falsely produced a
;      @call capture for each of its type names.
;   2. Without requiring a second `atom` sibling, a bare single-atom `expr`
;      (e.g. a literal function body like `area = 314`, where `314` itself
;      parses as a `qid` atom in this grammar) also falsely matched.
; The patterns below fix both: anchoring on `rhs "="` (never `":"`, i.e.
; never a signature) and requiring a second sibling atom (i.e. an actual
; argument, not a bare reference/literal).

; Module application: module M = SomeModule arg
(module_application
  (module_name) @call)

; Function application at the head of a defining equation's body
; (`f x y = ...`'s rhs, or a nested body expression). Requires >=2 atom
; children so a bare single-name/literal rhs is not mistaken for a call.
(rhs
  "="
  .
  (expr
    .
    (atom
      .
      (qid) @call)
    .
    (atom)))
