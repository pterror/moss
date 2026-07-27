; CMake CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium CMake grammar node types.
;
; CMake's `if_condition` is *flat*, not nested: `if_command`, `body`,
; `elseif_command`, `body`, ..., `else_command`, `body`, `endif_command` are
; all direct, unfielded siblings of `if_condition` — there's no nesting of
; elseif/else content inside the if. Likewise `if_command`/`elseif_command`
; have no `condition:` field; their condition arguments are direct
; `(argument)` children of an `argument_list`. We match condition + the
; immediately-following `body` via anchors.

; ---------------------------------------------------------------------------
; if / elseif / else (branch)
; ---------------------------------------------------------------------------

(if_condition
  (if_command
    (argument_list (argument) @cfg.branch.condition))
  .
  (body) @cfg.branch.then
  .
  (else_command)
  .
  (body) @cfg.branch.else
) @cfg.branch

(if_condition
  (if_command
    (argument_list (argument) @cfg.branch.condition))
  .
  (body) @cfg.branch.then
) @cfg.branch

(if_condition
  (elseif_command
    (argument_list (argument) @cfg.branch.condition))
  .
  (body) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; foreach (loop)
; ---------------------------------------------------------------------------

(foreach_loop
  (foreach_command
    (argument_list (argument) @cfg.loop.condition))
  .
  (body) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_loop
  (while_command
    (argument_list (argument) @cfg.loop.condition))
  .
  (body) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; CMake has no dedicated break/continue/return statement node types — they're
; ordinary commands (`normal_command` with `identifier` "break"/"continue"/
; "return"). Command names are case-insensitive in CMake.

(normal_command
  (identifier) @_name
  (#match? @_name "(?i)^break$")
) @cfg.exit.break

(normal_command
  (identifier) @_name
  (#match? @_name "(?i)^continue$")
) @cfg.exit.continue

(normal_command
  (identifier) @_name
  (#match? @_name "(?i)^return$")
) @cfg.exit.return
