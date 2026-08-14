; Complexity query for HCL (Terraform/HashiCorp Configuration Language)
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; HCL complexity comes from conditional expressions, for expressions,
; and dynamic blocks.

; NOTE on reachability: every consumer of this query (`normalize rank complexity`,
; normalize-ratchet's `complexity` metric, normalize-budget's complexity-delta
; metric) only computes complexity for tag-query matches captured
; `@definition.function` / `@definition.method` (see
; crates/normalize/src/analyze/complexity.rs, crates/normalize-ratchet/src/
; metrics/complexity.rs). `hcl.tags.scm` never emits either capture — HCL has no
; function/method construct, only `@definition.var` for blocks and attributes —
; so this query, while internally correct, is currently unreachable by any
; built-in per-symbol complexity report. It IS exercised directly by
; `normalize_facts::extract::compute_complexity` when called on an arbitrary node
; (verified via `normalize syntax query`), so it is not dead in the sense of
; "never runs" — just not wired to a symbol-level complexity metric for this
; language. Promoting HCL blocks to a Function-like `SymbolKind` to make them
; complexity-eligible would ripple into `is_container`/view-rendering/filtering
; behavior for every HCL symbol and needs its own review, not a query-file-only
; fix — tracked as a follow-up in TODO.md rather than done speculatively here.

; Complexity nodes
(conditional) @complexity
(for_tuple_expr) @complexity
(for_object_expr) @complexity

; Nesting nodes
(block) @nesting
(conditional) @nesting
(for_tuple_expr) @nesting
(for_object_expr) @nesting
