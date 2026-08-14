; Refactor node classification for Zig.
; (The Zig grammar uses CamelCase node kinds.)

; Function definitions and their parameter list. A function is `Decl(FnProto,
; Block)` — the prototype and body are siblings — so the function-def capture is
; the `FnProto` (which directly contains `ParamDeclList`).
(FnProto (ParamDeclList) @refactor.param_list) @refactor.function_def

; Call expressions and their argument list.
;
; Out of the methodology's core four dimensions (tags/calls/imports/
; complexity/types), but the plain-call pattern below shares the exact
; node-type gap `zig.calls.scm` had (see that file's comment): a dotted
; call (`obj.method()`) is NOT `SuffixExpr(IDENTIFIER, FnCallArguments)` —
; `FnCallArguments` lives inside a `FieldOrFnCall` sibling, not as a direct
; `SuffixExpr` child, for any dotted call. Verified via `normalize syntax
; query`: the plain-call-only pattern below matched `foo()`/`@import(...)`
; but 0 times against `obj.method()`/`std.debug.print(...)`, silently
; dropping method calls' arg lists from the refactor engine. Fixed the same
; way as `zig.calls.scm`.
(SuffixExpr (FnCallArguments) @refactor.arg_list) @refactor.call

(SuffixExpr
  (FieldOrFnCall
    function_call: (IDENTIFIER)
    (FnCallArguments) @refactor.arg_list)) @refactor.call

; Variable declarations (inline-variable). `var`/`const` bindings are `VarDecl`.
(VarDecl) @refactor.var_decl

; Reassignment. Every expression statement is wrapped in `AssignExpr`; a *real*
; assignment is distinguished by the presence of an `AssignOp` child (a bare
; expression statement has no `AssignOp`).
(AssignExpr (AssignOp)) @refactor.reassign

; Scope / block containers.
(Block) @refactor.scope @refactor.block
(BlockExpr) @refactor.scope @refactor.block
(source_file) @refactor.scope @refactor.block

; Statements. Zig wraps each statement in a `Statement` node; the loop/if/labeled
; forms are the compound control-flow statements.
(Statement) @refactor.statement
(IfStatement) @refactor.statement
(LoopStatement) @refactor.statement
(LabeledStatement) @refactor.statement
