; Jinja2 CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against installed Jinja2 grammar by running:
;   normalize syntax ast /tmp/sample.jinja2
;
; Node types confirmed: if_statement, elif_clause, else_clause, for_statement,
; endfor, endif.
;
; for_statement also has a `for_else` child ({% for %}...{% else %}...{% endfor %},
; the else-branch that runs when the loop iterates zero times) and a
; `condition` field ({% for x in items if cond %}, a per-item filter). Both
; are confirmed real grammar nodes (`normalize syntax query -p <probe>
; "(for_statement (for_else) @e) @for"` and "...condition: (_) @c..." both
; matched). Neither is captured here: normalize-cfg's CaptureKind vocabulary
; (crates/normalize-cfg/src/builder.rs) has no loop-else or loop-filter kind
; (only Loop/LoopCondition/LoopBody exist), so there is nowhere to route
; these captures without adding a new CaptureKind to normalize-cfg itself —
; out of scope for a query-only fix. Documented here rather than fabricating
; a match against an existing capture name.

; ---------------------------------------------------------------------------
; if / elif / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  (_) @cfg.branch.condition
  (elif_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  (_) @cfg.branch.condition
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  (_) @cfg.branch.condition
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; for (loop over collection)
; ---------------------------------------------------------------------------

; BUG FIXED: the previous pattern was `(identifier) @cfg.loop.condition
; (identifier) @cfg.loop.condition`, matching the loop target and the
; iterable positionally as two identically-named captures — and never
; capturing `@cfg.loop.body` at all. It only worked by accident when the
; iterable was a bare identifier ({% for x in items %}); any non-identifier
; iterable ({% for x in items|sort %}, {% for x in get_items() %},
; {% for k, v in pairs.items() %}) produced zero matches, since `iterable`
; is grammar-typed as the full 23-variant expression union, not `identifier`
; alone (node-types.json). Verified via `normalize syntax query` that
; `iterable: (_)` and `body: (_)` both match correctly on all of the above
; forms, including tuple targets (`target: (identifier_tuple)`).
;
; `body` is grammar-optional (an empty loop, {% for x in items %}{% endfor %},
; has no body children at all) — requiring `body: (_)` unconditionally drops
; the whole @cfg.loop match for an empty loop (verified: 0 matches). Two
; mutually-exclusive patterns, split on field presence via `!body`/`body:`,
; cover both cases without double-counting a non-empty loop.
(for_statement
  iterable: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

(for_statement
  iterable: (_) @cfg.loop.condition
  !body
) @cfg.loop
