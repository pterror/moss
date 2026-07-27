; Lean 4 CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Lean 4 grammar node types and against
; crates/normalize-languages/tests/fixtures/lean/sample.lean via
;   normalize syntax query <query> -p <file> --show-source
;
; Lean 4 is a dependently-typed proof assistant / functional language.
; Control flow: if-then-else, match, do-notation `for`.
;
; if_then_else has NO named fields — condition/then/else are flat,
; unnamed children interleaved with the literal "if"/"then"/"else"
; keyword tokens, matched positionally here. match has "value"
; (scrutinee) and "patterns" (match_alt arms) fields. for_in has
; "iterable" and "body" fields (body is always a (do) block).
;
; Grammar has no while-loop node type at all (no "while" anywhere in
; node-types.json — Lean 4 do-notation has no native while construct
; in this grammar) and no break/continue/throw node types either;
; do-notation only models `do_return`. These are correctly left
; unimplemented rather than fabricated.

; ---------------------------------------------------------------------------
; if / else (branch expression — if_then_else)
; ---------------------------------------------------------------------------

(if_then_else
  "if"
  (_) @cfg.branch.condition
  "then"
  (_) @cfg.branch.then
  "else"
  (_) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; match (pattern matching)
; ---------------------------------------------------------------------------

(match
  value: (_) @cfg.match.scrutinee
  patterns: (match_alt) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (do-notation loop — Lean 4)
; ---------------------------------------------------------------------------

(for_in
  iterable: (_) @cfg.loop.condition
  body: (do) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(do_return) @cfg.exit.return
