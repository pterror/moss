; Lua CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against installed Lua grammar (arborium-lua) by running:
;   normalize syntax ast <file>
;   normalize syntax query <query> -p <file> --show-source
;
; The Lua grammar uses positional children, not named fields. Node
; types confirmed: if_statement, elseif_statement, else_statement,
; for_statement, for_numeric_clause, while_statement, repeat_statement,
; return_statement, block. The then/else bodies of if_statement and
; elseif_statement are wrapped in a (block) node (not flat statements),
; so anchoring on the literal "if"/"then"/"elseif" keyword tokens plus
; (block) disambiguates condition vs. body reliably where a bare (_)
; wildcard would over-match across sibling elseif/else clauses.
;
; There is no "function_call_statement" node type — a bare call used as
; a statement is just a (function_call) node directly; its callee name
; field wraps a plain (identifier), not "function_call_statement".

; ---------------------------------------------------------------------------
; if / elseif / else (branch)
; ---------------------------------------------------------------------------

; if with else (else_statement child present)
(if_statement
  "if"
  (_) @cfg.branch.condition
  "then"
  (block) @cfg.branch.then
  (else_statement) @cfg.branch.else
) @cfg.branch

; if without else (no else_statement child)
(if_statement
  "if"
  (_) @cfg.branch.condition
  "then"
  (block) @cfg.branch.then
  .
) @cfg.branch

; elseif clause (branch within if chain)
(elseif_statement
  "elseif"
  (_) @cfg.branch.condition
  "then"
  (block) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; for (numeric and generic)
; ---------------------------------------------------------------------------

(for_statement
  (for_numeric_clause) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

; generic for (for k, v in ...)
(for_statement
  (for_generic_clause) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

; for_statement with just a block (fallback)
(for_statement
  (block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  (_) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; repeat / until (do-while equivalent)
; ---------------------------------------------------------------------------

(repeat_statement
  (block) @cfg.loop.body
  (_) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

; Lua 5.2+ has goto but no continue; error() is the throw equivalent
(function_call
  name: (identifier) @_fn
  (#eq? @_fn "error")
) @cfg.exit.throw
