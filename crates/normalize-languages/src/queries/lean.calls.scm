; Lean 4 calls query
; @call — function application nodes
; @call.qualifier — not applicable (Lean uses dot notation differently)
;
; Lean 4 uses juxtaposition for function application. The `apply` node
; represents function application; its `name` field holds the callee.
;
; Verified against arborium-lean 2.17.0's node-types.json and real parse
; output (`normalize syntax ast` / `normalize syntax query`).
;
; `apply`'s `name` field allows ~40 node-type variants (any term can be
; applied — `(fun x => x+1) 5`, `(have h := p; f) x`, etc.), since Lean has
; no syntactic distinction between "callable" and other expressions. Most
; of those variants are complex sub-expressions with no meaningful "name"
; to report to a call-graph consumer and are left unhandled by design, not
; by oversight. Two variants beyond the plain `identifier` the prior
; version handled are common enough, and specific enough to have a useful
; captured name, to add:
;
; - `parenthesized`: applying a grouped expression, most commonly an
;   immediately-invoked lambda — `(fun x => x + 1) 5` — a common local-
;   scoping idiom in functional Lean code. Confirmed via
;   `normalize syntax ast`.
; - `proj`: applying an anonymous-constructor/tuple field projection —
;   `pair.1 pair.2` (calling the function stored in the first component of
;   a pair). Confirmed via `normalize syntax ast`; NOT the same shape as
;   ordinary dot-notation method calls like `xs.foldl (...)`, which parse
;   as a single dotted `identifier` token already covered below.
;
; Note: `apply` is also how *type-level* application parses (`List Nat` in
; a type position) — Lean's dependent-type grammar has no distinct node
; for "term application" vs "type application" (types are ordinary terms).
; This query therefore also matches type-application sites; there is no
; structural signal available to exclude them, mirroring the same
; types-are-values ambiguity documented in `zig.types.scm`/`d.types.scm`.

; Function application: f x y, Float.sqrt x, xs.foldl (...)
(apply
  name: (identifier) @call)

; Immediately-invoked parenthesized expression: (fun x => x + 1) 5
(apply
  name: (parenthesized) @call)

; Applying a tuple/anonymous-constructor projection: pair.1 pair.2
(apply
  name: (proj) @call)
