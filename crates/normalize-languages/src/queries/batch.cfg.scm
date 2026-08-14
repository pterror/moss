; Batch (Windows CMD) CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Batch grammar node types.
;
; The tree-sitter-batch grammar is minimal: it does not model if_statement,
; for_statement, while_statement, or goto as distinct AST nodes. All control
; flow keywords (IF, FOR, GOTO, etc.) are collapsed into the generic `keyword`
; node alongside non-branching commands. No @cfg.branch / @cfg.loop captures
; are possible without false positives.
;
; Labels (function definitions) are structurally modeled by the grammar but
; are NOT captured here — the CFG capture vocabulary (@cfg.branch/@cfg.loop/
; @cfg.match/@cfg.exit.*, see normalize-cfg) has no slot for a bare label,
; and labels are also unreliable for that purpose: `goto :label` and
; `call :label` each emit a spurious extra `function_definition` sibling for
; the target, indistinguishable at the query level from a genuine label
; definition (verified via `normalize syntax ast`; see the longer note in
; batch.complexity.scm, which does capture labels as @nesting and documents
; this false positive in full since @nesting has no such natural exclusion).
; This query intentionally produces no captures for any construct.
