; Zig complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Zig's tree-sitter grammar uses PascalCase node names (e.g. IfStatement,
; ForStatement) inherited from its grammar source.
;
; Verified against arborium-zig 2.17.0's node-types.json and real parse
; output.
;
; `(ErrorUnionExpr) @complexity` (the prior version of this file) is a
; real bug, not just an incomplete-coverage gap: `ErrorUnionExpr` is Zig's
; near-universal generic-expression wrapper node — every single expression
; in the grammar passes through one, error union or not (confirmed:
; `dx * dx`, a plain variable read, a string literal, and a `!void` return
; type all wrap in `ErrorUnionExpr` even with no `!`/error handling
; involved anywhere). It matched 108 of 125 total complexity captures on
; the ~80-line zig sample fixture alone — effectively "count every
; expression" rather than "count decision points", making complexity
; scores meaningless. Removed.
;
; The real error-handling complexity sources are `try` (implicit
; early-return-on-error, `UnaryExpr operator: (PrefixOp) "try"`) and
; `catch` (explicit error handling). `catch` needs no separate pattern:
; it's implemented as an ordinary `BinaryExpr` (operator `BitwiseOp`
; "catch" — the same shape `zig.cfg.scm`'s try/catch pattern uses), so the
; existing `(BinaryExpr) @complexity` below already counts it. `try` does
; need its own pattern — it's a `UnaryExpr`, not a `BinaryExpr` — and the
; prior version counted neither.

; Complexity nodes
(IfStatement) @complexity
(ForStatement) @complexity
(WhileStatement) @complexity
(SwitchExpr) @complexity
(BinaryExpr) @complexity
(UnaryExpr
  operator: (PrefixOp) @_op
  (#eq? @_op "try")) @complexity

; Nesting nodes
(IfStatement) @nesting
(ForStatement) @nesting
(WhileStatement) @nesting
(SwitchExpr) @nesting
(FnProto) @nesting
(ContainerDecl) @nesting
