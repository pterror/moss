; Perl CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Verified against arborium Perl grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
; Note: the CLI's language registry resolves ".pl" to Perl, but Prolog
; also claims ".pl" — probing was done through ".pm" fixture copies to
; force Perl resolution unambiguously (documented as a pre-existing
; collision in TODO.md, not fixed here).
;
; conditional_statement has "condition"/"block" fields (NOT
; "consequence"/"alternative" — those never existed on this node).
; elsif/else are not siblings of conditional_statement — each is
; nested one inside the previous: conditional_statement -> (elsif) ->
; (elsif) -> ... -> (else), so the elsif/else chain is walked by
; matching (elsif)/(else) as their own @cfg.branch nodes nested inside
; the outer's branch_else range.
;
; Perl has two structurally distinct for-loops: for_statement (foreach
; form, fields: variable/list/block — list is the iterable, mapped to
; loop.condition since CaptureKind has no separate "iterable" slot)
; and cstyle_for_statement (C-style, field: condition/block).
;
; last/next/redo are not their own node types — they are
; (loopex_expression loopex: "last"/"next"/"redo"), an anonymous
; keyword token under a named field, matched here via field + literal
; token (not #eq?, since the token itself is the field value).
;
; die is not a dedicated node type either — it is an ordinary function
; call (ambiguous_function_call_expression without parens,
; function_call_expression with parens), matched via #eq? on the
; callee text, same technique as Lua's error()-as-throw.
;
; given/when/default (Perl's experimental switch, use feature
; 'switch') is NOT supported by this grammar — it parses to ERROR
; nodes (verified with a given/when/default probe file). No match
; capture is emitted for it; there is nothing valid to capture.

; ---------------------------------------------------------------------------
; if / elsif / else (branch)
; ---------------------------------------------------------------------------

(conditional_statement
  condition: (_) @cfg.branch.condition
  block: (_) @cfg.branch.then
  (elsif) @cfg.branch.else
) @cfg.branch

(conditional_statement
  condition: (_) @cfg.branch.condition
  block: (_) @cfg.branch.then
  .
) @cfg.branch

(elsif
  condition: (_) @cfg.branch.condition
  block: (_) @cfg.branch.then
  (elsif) @cfg.branch.else
) @cfg.branch

(elsif
  condition: (_) @cfg.branch.condition
  block: (_) @cfg.branch.then
  (else) @cfg.branch.else
) @cfg.branch

(elsif
  condition: (_) @cfg.branch.condition
  block: (_) @cfg.branch.then
  .
) @cfg.branch

(else
  block: (_) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; while / until / foreach / C-style for (loop)
; ---------------------------------------------------------------------------

(loop_statement
  condition: (_) @cfg.loop.condition
  block: (_) @cfg.loop.body
) @cfg.loop

(for_statement
  list: (_) @cfg.loop.condition
  block: (_) @cfg.loop.body
) @cfg.loop

(cstyle_for_statement
  condition: (_) @cfg.loop.condition
  block: (_) @cfg.loop.body
) @cfg.loop

; do { ... } while (...) — condition has no field name reaching the
; query engine (display tooling shows "condition:" but the query
; engine does not match it as a field); matched positionally instead.
(postfix_loop_expression
  (do_expression (block) @cfg.loop.body)
  (_) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch — eval {} is Perl's try equivalent ($@ holds the error)
; ---------------------------------------------------------------------------

(eval_expression
  (block) @cfg.try.body
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_expression) @cfg.exit.return

(loopex_expression loopex: "last") @cfg.exit.break

(loopex_expression loopex: "next") @cfg.exit.continue

(loopex_expression loopex: "redo") @cfg.exit.continue

(ambiguous_function_call_expression
  function: (function) @_fn
  (#eq? @_fn "die")
) @cfg.exit.throw

(function_call_expression
  function: (function) @_fn
  (#eq? @_fn "die")
) @cfg.exit.throw
