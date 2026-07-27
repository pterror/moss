; Svelte CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Verified against arborium Svelte grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
;
; Svelte has template-level control flow via {#if}, {#each}, {#await}.
; JavaScript logic inside <script> blocks is handled separately by the
; embedded JS/TS grammar (not this query) — this file targets only the
; template-directive layer, not the landmine case of needing to reach
; into script blocks.
;
; This grammar does NOT have "condition"/"consequence" fields on
; if_statement — the condition is nested one level inside
; (if_start condition: (svelte_raw_text)), and it is genuinely raw
; unparsed text (Svelte's template grammar doesn't parse the
; expression inside {#if ...} as JS/TS), not a real expression node.
; The then/else content likewise has no wrapper "body" node — it is
; whatever flat sibling elements follow if_start/else_start/each_start
; directly, so only the FIRST such sibling is captured (anchored with
; "." to avoid also matching the closing {/if}/{/each} tag), which is
; a faithful-but-partial signal when a branch has multiple top-level
; template elements.

; ---------------------------------------------------------------------------
; {#if} / {:else if} / {:else} (branch)
; ---------------------------------------------------------------------------

(if_statement
  (if_start condition: (_) @cfg.branch.condition)
  . (_) @cfg.branch.then
  (else_if_block) @cfg.branch.else
) @cfg.branch

(if_statement
  (if_start condition: (_) @cfg.branch.condition)
  . (_) @cfg.branch.then
  (else_block) @cfg.branch.else
) @cfg.branch

(if_statement
  (if_start condition: (_) @cfg.branch.condition)
  . (_) @cfg.branch.then
  .
) @cfg.branch

(else_if_block
  (else_if_start condition: (_) @cfg.branch.condition)
  . (_) @cfg.branch.then
) @cfg.branch

(else_block
  (else_start)
  . (_) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; {#each} (loop over collection)
; ---------------------------------------------------------------------------

(each_statement
  (each_start identifier: (_) @cfg.loop.condition)
  . (_) @cfg.loop.body
) @cfg.loop
