; jq CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium jq grammar node types.
;
; In jq, if/then/else/elif/end, reduce/foreach, and try/catch are anonymous
; keyword tokens, not named statement nodes — the whole construct is
; flattened directly into the enclosing `query` node rather than wrapped in
; a dedicated node type (verified via `normalize syntax ast`: a top-level
; `if X then Y else Z end` produces one `query` node whose children are the
; flat sequence `if (query) then (query) (elif)* (else)? end`, not a
; wrapping `if_statement` node). `elif`/`else`/`catch` *do* get their own
; named node type (one per clause), so those are matched directly; the
; top-level `if`/`reduce`/`foreach`/`try` keywords are matched as anonymous
; token literals (`"if"` etc — confirmed queryable, not a limitation) with
; a `.` anchor requiring they be the query node's first child, so only the
; query that *is* that construct matches (not, e.g., an unrelated query
; that happens to contain the word "if" as a string literal).

; ---------------------------------------------------------------------------
; if / elif / else (branch)
; ---------------------------------------------------------------------------

; Bare `if ... then ... end` (previously entirely uncaptured — only `elif`
; below produced a @cfg.branch, so the most common jq conditional form, an
; if/else with no elif, registered zero branch nodes in the CFG).
(query
  . "if"
  (query) @cfg.branch.condition
  "then"
  (query) @cfg.branch.then
  (else (query) @cfg.branch.else)?
) @cfg.branch

; elif clause — has its own named node type wrapping `elif COND then BODY`.
; `else`, if present, is a sibling of the elif node(s) under the same `if`,
; not a child of the elif itself, so it's captured separately below (its
; own @cfg.branch.else would double-count against the top-level `if`'s
; branch above; only the top-level if's else is meaningful as a branch
; target here — an elif's own trailing arm is always `else`, not its own
; branch-else).
(elif
  . "elif"
  (query) @cfg.branch.condition
  "then"
  (query) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; reduce / foreach (jq's only loop/iteration constructs)
; ---------------------------------------------------------------------------
; No sub-captures for condition/body: `reduce SOURCE as $x (INIT; UPDATE)`
; has no boolean condition, and `foreach`'s 3-part form (INIT; UPDATE;
; EXTRACT) has two candidate "body" expressions with different roles
; (accumulator update vs. emitted value) that a single @cfg.loop.body
; capture can't distinguish without misrepresenting one of them — so only
; the whole-construct container is captured, not further decomposed.
(query . "reduce") @cfg.loop
(query . "foreach") @cfg.loop

; ---------------------------------------------------------------------------
; try / catch
; ---------------------------------------------------------------------------
; Previously only `(catch) @cfg.try.catch` existed, with no `@cfg.try`
; container at all — try/catch never registered as a CFG node in jq. The
; `try EXPR catch EXPR2` / `try EXPR` (catch optional) forms are both
; flattened into the same `query` node shape as if/reduce/foreach above.
(query
  . "try"
  (query) @cfg.try.body
) @cfg.try

(catch) @cfg.try.catch
