; Zig CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Zig grammar node types using real fixtures.
;
; Zig uses PascalCase node names: IfStatement, ForStatement, WhileStatement,
; SwitchExpr. There are NO `condition:`/`body:` fields anywhere in this
; grammar, and no `ElseSuffix`/`ReturnStatement`/`BreakStatement`/
; `ContinueStatement` node types (all invented in the prior broken version).
; `IfStatement`/`WhileStatement` wrap their header in an `IfPrefix`/
; `WhilePrefix` node (condition is a bare child of that prefix); the
; then-body is a bare sibling of the prefix (anchored, since without an
; anchor a wildcard can skip straight to the trailing else-`Statement`).
; `else` is an anonymous token followed by a `Statement` sibling that wraps
; either a nested `IfStatement` (else-if chain) or the terminal body.
; `return`/`break`/`continue` are anonymous tokens inside a shared
; `AssignExpr` node (there's no dedicated statement node per keyword).
; Zig's error-union `catch` handler is not a dedicated node type either —
; it's the `right:` field of a `BinaryExpr` whose `operator:` is a
; `BitwiseOp` wrapping the `catch` token.

; ---------------------------------------------------------------------------
; if / else (branch)
; ---------------------------------------------------------------------------

(IfStatement
  (IfPrefix (_) @cfg.branch.condition) . (_) @cfg.branch.then
  "else"
  (Statement (IfStatement) @cfg.branch.else)
) @cfg.branch

(IfStatement
  (IfPrefix (_) @cfg.branch.condition) . (_) @cfg.branch.then
  "else"
  (Statement) @cfg.branch.else
) @cfg.branch

(IfStatement
  (IfPrefix (_) @cfg.branch.condition) . (_) @cfg.branch.then .
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(SwitchExpr
  . (_) @cfg.match.scrutinee
  (SwitchProng) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(ForStatement
  (ForPrefix) @cfg.loop.condition . (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(WhileStatement
  (WhilePrefix) @cfg.loop.condition . (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch (error union handling in Zig uses catch, not try blocks)
; ---------------------------------------------------------------------------

; Zig error handling: expr catch |err| { ... }
(BinaryExpr
  operator: (BitwiseOp "catch")
  right: (_) @cfg.try.catch
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(AssignExpr "return" @cfg.exit.return)

(AssignExpr "break" @cfg.exit.break)

(AssignExpr "continue" @cfg.exit.continue)
