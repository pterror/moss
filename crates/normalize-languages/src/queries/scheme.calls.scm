; Scheme calls query
; @call — function application nodes
; @call.qualifier — not applicable (Scheme has no method dispatch at this
; level; qualified library-prefixed names, e.g. via `import (prefix ...)`,
; are just plain symbols with a longer name, not a distinct node shape)
;
; Scheme (and Lisps generally) use `(proc arg ...)` list syntax for calls.
; The grammar represents everything as `list` nodes. The first element of
; a list is the operator/function being called. A `symbol` as the first
; child is a named function call.
;
; The syntax for a function call and a special-form/macro invocation is
; IDENTICAL in a Lisp — both are `(leading-symbol args...)`. Without
; excluding known special forms, every `(define ...)`, `(let ...)`,
; `(cond ...)`, `(if ...)`, `(import ...)` etc. was counted as a "call" —
; confirmed via `normalize syntax query` against `sample.scm`, where
; `define`/`let`/`cond`/`if`/`import`/`define-record-type`/`else` were all
; reported as calls, polluting any call-graph consumer with keyword noise.
; List drawn from R7RS's standard syntactic keywords (R7RS §4, §5) plus the
; forms this codebase's own `scheme.tags.scm`/`scheme.cfg.scm`/
; `scheme.complexity.scm`/`scheme.imports.scm` already treat as structural,
; not call, constructs.
;
; KNOWN LIMITATION, not fixed here: a parameter list like `(n)` in
; `(define (square n) ...)`, a `define-record-type` field spec like
; `(x point-x)`, and a library name list like `(scheme base)` inside
; `import` are all structurally IDENTICAL to a zero-arg call `(n)`/
; `(point-x)`/`(base)` — every one is a `list` whose first child is a
; `symbol`. Scheme's grammar gives no field distinguishing "this list is a
; binding/spec form" from "this list is a call", and disambiguating would
; require tracking which parent form (define/define-record-type/import/...)
; the list sits inside, which a single flat tree-sitter pattern cannot
; express for the general case (the parent-form-aware patterns in
; `scheme.tags.scm`/`scheme.imports.scm` only work because they anchor on
; the *specific* known parent forms, not the general "any call" case).
; Confirmed via `normalize syntax query` that names like `square`/`x`/`y`/
; `scheme` (from `(scheme base)`) still appear as bogus @call captures
; after the special-form exclusion below. This mirrors the exact, documented
; limitation in `commonlisp.calls.scm` — not a gap to special-case away.
(list
  .
  (symbol) @call
  (#not-match?
    @call
    "^(define|define-syntax|define-record-type|define-values|lambda|named-lambda|let|let\\*|letrec|letrec\\*|let-values|let\\*-values|let-syntax|letrec-syntax|if|cond|case|case-lambda|when|unless|and|or|do|begin|set!|quote|quasiquote|unquote|unquote-splicing|delay|delay-force|make-promise|guard|with-exception-handler|parameterize|else|import|require|syntax-rules|syntax-error|cond-expand|include|include-ci|export|define-library|library|only|except|prefix|rename)$"))
