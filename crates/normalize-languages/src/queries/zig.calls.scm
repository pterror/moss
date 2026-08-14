; Zig calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; Zig uses PascalCase node names. Regular calls are SuffixExpr nodes with
; a FnCallArguments child. Builtin calls (@import, @This, etc.) are
; builtin_call_expression nodes.
;
; Verified against arborium-zig 2.17.0's node-types.json and real parse
; output.

; Regular call: func()
; SuffixExpr wraps both the callee and the argument list as direct children.
(SuffixExpr
  variable_type_function: (IDENTIFIER) @call
  (FnCallArguments))

; Field (method) call: obj.method(), std.debug.print(...), a.b().c()
; A dotted call is NOT `SuffixExpr(qualifier, SuffixOp, IDENTIFIER,
; FnCallArguments)` — `SuffixOp` is a distinct node used for array
; indexing/dereference (`a[i]`, `a.*`, `a.?`), never for a dotted call.
; The prior version of this pattern required `SuffixOp` between the
; qualifier and the identifier, which real dotted-call syntax never
; produces — confirmed via `normalize syntax query`: it matched 0 times
; against `obj.method()`/`std.debug.print()`/etc, silently dropping every
; method call in this codebase's own Zig fixtures from extraction.
;
; The actual structure chains `FieldOrFnCall` siblings inside `SuffixExpr`;
; each link that carries both a `function_call:` field and `FnCallArguments`
; is itself a call. `@call.qualifier` is the SuffixExpr's base receiver
; identifier — for a multi-call chain (`a.b().c()`) this fires once per
; call in the chain, each with the same base qualifier `a` (the grammar
; exposes no intermediate node to anchor a more precise per-link receiver
; against, since the true receiver of the second call is the *result* of
; the first, not an identifier).
(SuffixExpr
  variable_type_function: (IDENTIFIER) @call.qualifier
  (FieldOrFnCall
    function_call: (IDENTIFIER) @call
    (FnCallArguments)))

; Builtin call: @import("file"), @This(), etc.
; Builtins use BUILTINIDENTIFIER inside a SuffixExpr
(SuffixExpr
  (BUILTINIDENTIFIER) @call
  (FnCallArguments))
