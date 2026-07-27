; Zsh CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Modeled on bash.cfg.scm (Zsh shares Bash's grammar base) and checked
; field-by-field against arborium-zsh 2.17.0's node-types.json.
;
; IMPORTANT — UNVERIFIED AT RUNTIME (documented gap, not a guess):
; Every other language in this remediation batch was verified by parsing a
; real fixture with `normalize syntax ast` and checking actual capture
; output with `normalize syntax query`. That was NOT possible for zsh: the
; compiled arborium-zsh 2.17.0 grammar (target/grammars/zsh.so, rebuilt
; fresh via `cargo xtask build-grammars --force` while investigating this)
; fails to parse even the most trivial real-world zsh source into anything
; but `(ERROR)` nodes. Confirmed empirically across a dozen minimal
; snippets: a bare two-word command (`echo hi`) does not parse as a single
; `command` node with an argument — it splits into an ERROR-wrapped first
; word and an unrelated second `command`. `if`/`for`/`while`/`case`
; keyword-led statements never produce `if_statement`/`for_statement`/
; `while_statement`/`case_statement` nodes from realistic source at all
; (tried with/without brackets, single- and multi-line, `[ ]` and `[[ ]]`,
; nested and top-level). This reproduces from a clean rebuild, so it is not
; a stale-.so artifact; grammar.json declares no `externals` (no scanner),
; which is consistent with the argument/reserved-word disambiguation that
; upstream tree-sitter-bash normally handles via an external scanner simply
; not working in this grammar.
;
; This is a grammar-level defect (affects ALL zsh query types — tags,
; imports, calls — not just this CFG query), not something a `.cfg.scm`
; rewrite can fix. The patterns below are schema-correct (every node type
; and field name exists in node-types.json and mirrors the equivalent,
; empirically-verified bash.cfg.scm construct), so this file compiles
; cleanly against the grammar. Whether it ever CAPTURES anything from real
; zsh source is unverified and, per current evidence, unlikely until the
; upstream grammar/scanner defect is fixed. Tracked in TODO.md rather than
; silently declared "done".
;
; Zsh's grammar has no C-style `for ((i=0; i<n; i++))` node type at all
; (unlike Bash's `c_style_for_statement`) — genuine grammar gap, not
; fabricated.

; ---------------------------------------------------------------------------
; if / elif / else (branch)
; ---------------------------------------------------------------------------

; if_statement has `condition:`/`consequence:` fields, no `body:` field.
; Anchor immediately after condition to bind the then-branch positionally,
; mirroring bash.cfg.scm's proven approach.
(if_statement
  condition: (_) @cfg.branch.condition
  .
  (_) @cfg.branch.then
) @cfg.branch

(if_statement
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  (elif_clause) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; case (match)
; ---------------------------------------------------------------------------

(case_statement
  value: (_) @cfg.match.scrutinee
  (case_item) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_statement
  variable: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while / until (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; Zsh has no dedicated return/break/continue statement node types — they're
; ordinary builtin commands (`command` with `command_name: "return"`, etc),
; same as Bash.

(command
  name: (command_name) @_cmd
  (#eq? @_cmd "return")
) @cfg.exit.return

(command
  name: (command_name) @_cmd
  (#eq? @_cmd "break")
) @cfg.exit.break

(command
  name: (command_name) @_cmd
  (#eq? @_cmd "continue")
) @cfg.exit.continue
