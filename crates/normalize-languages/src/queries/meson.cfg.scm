; Meson CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Meson grammar node types and against
; crates/normalize-languages/tests/fixtures/meson/meson.build via
; the GrammarLoader directly (Meson's build files have no distinct
; file extension `normalize syntax query -p <file>` recognizes by
; default; verified with a scratch probe test instead).
;
; if_command / elseif_command have NO fields — the condition is the
; first flat unnamed child, the rest are body statements; matched
; positionally with a leading "." anchor. else_command has no
; condition, only flat body statements. foreach_command HAS fields:
; "item" (loop variable) and "array" (iterable); the loop body is
; still flat unnamed children, so "array:" must be consumed
; (uncaptured) before the body pattern or it gets swept into the body
; capture. There is no dedicated break_statement/continue_statement —
; the grammar uses (keyword_break)/(keyword_continue) leaf nodes.

; ---------------------------------------------------------------------------
; if / elif / else (branch)
; ---------------------------------------------------------------------------

(if_command
  .
  (_) @cfg.branch.condition
  (_)+ @cfg.branch.then
) @cfg.branch

(elseif_command
  .
  (_) @cfg.branch.condition
  (_)+ @cfg.branch.then
) @cfg.branch

(else_command) @cfg.branch.else

; ---------------------------------------------------------------------------
; foreach (loop)
; ---------------------------------------------------------------------------

(foreach_command
  item: (identifier) @cfg.loop.condition
  array: (_)
  (_)+ @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(keyword_break) @cfg.exit.break

(keyword_continue) @cfg.exit.continue
