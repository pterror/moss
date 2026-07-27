; Elixir CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Elixir grammar node types.
;
; In Elixir's tree-sitter grammar, if/case/cond/with/for/unless/try are
; represented as call nodes — they are macros, not special forms. We match
; on the specific call names for precision. `try`'s rescue/catch/after
; clauses are `rescue_block`/`catch_block`/`after_block` node types (not
; `rescue_clause`/`catch_clause`), and they sit as siblings *inside* the
; same `do_block` as the try body, not as separate call arguments.

; ---------------------------------------------------------------------------
; if / unless (branch)
; ---------------------------------------------------------------------------

(call
  target: (identifier) @_fn
  (arguments . (_) @cfg.branch.condition)
  (do_block
    .
    (_) @cfg.branch.then
    (else_block . (_) @cfg.branch.else)?
  )
  (#eq? @_fn "if")
) @cfg.branch

(call
  target: (identifier) @_fn
  (arguments . (_) @cfg.branch.condition)
  (do_block
    .
    (_) @cfg.branch.then
    (else_block . (_) @cfg.branch.else)?
  )
  (#eq? @_fn "unless")
) @cfg.branch

; ---------------------------------------------------------------------------
; case (match)
; ---------------------------------------------------------------------------

(call
  target: (identifier) @_fn
  (arguments . (_) @cfg.match.scrutinee)
  (do_block
    (stab_clause) @cfg.match.arm
  )
  (#eq? @_fn "case")
) @cfg.match

; ---------------------------------------------------------------------------
; cond (multi-branch conditional)
; ---------------------------------------------------------------------------

(call
  target: (identifier) @_fn
  (do_block
    (stab_clause) @cfg.branch.then
  )
  (#eq? @_fn "cond")
) @cfg.branch

; ---------------------------------------------------------------------------
; for (comprehension / loop-like construct)
; ---------------------------------------------------------------------------

(call
  target: (identifier) @_fn
  (arguments . (_) @cfg.loop.condition)
  (do_block . (_) @cfg.loop.body)
  (#eq? @_fn "for")
) @cfg.loop

; ---------------------------------------------------------------------------
; try / rescue / catch / after (exception handling)
; ---------------------------------------------------------------------------

(call
  target: (identifier) @_fn
  (do_block . (_) @cfg.try.body)
  (#eq? @_fn "try")
) @cfg.try

(rescue_block) @cfg.try.catch

(catch_block) @cfg.try.catch

(after_block) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; raise/throw are calls in Elixir
(call
  target: (identifier) @_fn
  (#match? @_fn "^(raise|throw|exit)$")
) @cfg.exit.throw
