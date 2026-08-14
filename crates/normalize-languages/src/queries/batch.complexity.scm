; Batch (Windows CMD) complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; The tree-sitter-batch grammar is minimal: it does not model if_statement,
; for_statement, while_statement, or goto as distinct AST nodes. All control
; flow keywords (IF, FOR, GOTO, etc.) are collapsed into the generic `keyword`
; node alongside non-branching commands. Because the grammar cannot distinguish
; branching keywords from other keywords at the node-type level, no @complexity
; or @nesting captures are possible without false positives.
;
; Function definitions (labels starting with :) are the only structural
; containers — captured as @nesting for nesting depth.
;
; KNOWN FALSE POSITIVE (see batch.cfg.scm for the full explanation, verified
; via `normalize syntax ast`): `goto :label` and `call :label` each emit a
; spurious extra `function_definition` node for the target, indistinguishable
; at the query level from a genuine label definition. Not fixable via a
; `.scm` predicate — tree-sitter queries have no "not preceded by sibling of
; kind X" negation, and `call` isn't even a recognized keyword in this
; grammar (it parses inside an `ERROR` node), so there's no stable anchor to
; filter on for that case either.

; Nesting nodes
(function_definition) @nesting
