; Elixir tags query
; Covers: def/defp/defmacro functions and defmodule modules
;
; In Elixir's tree-sitter grammar, all definitions are represented as `call`
; nodes. The target (def, defp, defmodule, etc.) is an `identifier` child.
; Function names appear as the first argument (a `call` or `identifier` node
; inside `arguments`).

; Public functions: def <name>(...)
(call
  target: (identifier) @_kw
  (arguments
    (call
      target: (identifier) @name))
  (#eq? @_kw "def")) @definition.function

; Public functions (no-args): def name do ... end
(call
  target: (identifier) @_kw
  (arguments
    (identifier) @name)
  (#eq? @_kw "def")) @definition.function

; Private functions: defp <name>(...)
(call
  target: (identifier) @_kw
  (arguments
    (call
      target: (identifier) @name))
  (#eq? @_kw "defp")) @definition.function

; Private functions (no-args): defp name do ... end
(call
  target: (identifier) @_kw
  (arguments
    (identifier) @name)
  (#eq? @_kw "defp")) @definition.function

; Public macros: defmacro <name>(...)
(call
  target: (identifier) @_kw
  (arguments
    (call
      target: (identifier) @name))
  (#eq? @_kw "defmacro")) @definition.macro

; Public macros (no-args): defmacro name do ... end
(call
  target: (identifier) @_kw
  (arguments
    (identifier) @name)
  (#eq? @_kw "defmacro")) @definition.macro

; Private macros: defmacrop <name>(...)
(call
  target: (identifier) @_kw
  (arguments
    (call
      target: (identifier) @name))
  (#eq? @_kw "defmacrop")) @definition.macro

; Private macros (no-args): defmacrop name do ... end
(call
  target: (identifier) @_kw
  (arguments
    (identifier) @name)
  (#eq? @_kw "defmacrop")) @definition.macro

; Modules: defmodule <Alias>
(call
  target: (identifier) @_kw
  (arguments
    (alias) @name)
  (#eq? @_kw "defmodule")) @definition.module

; Protocols: defprotocol <Alias>
(call
  target: (identifier) @_kw
  (arguments
    (alias) @name)
  (#eq? @_kw "defprotocol")) @definition.interface

; Struct: defstruct (inside a module, struct is the module name — skip name capture)
; Implementation: defimpl <Protocol> for <Type>
(call
  target: (identifier) @_kw
  (arguments
    (alias) @name)
  (#eq? @_kw "defimpl")) @reference.implementation

; Guard clauses: `def name(args) when guard do ... end`. A guarded function
; head's `arguments` field does NOT contain a `call`/`identifier` directly —
; it contains a `binary_operator` (operator "when") whose `left` is the
; call/identifier that the un-guarded forms above match directly. Verified
; via `normalize syntax query`: none of the four un-guarded patterns above
; match a guarded def/defp/defmacro/defmacrop head at all, silently dropping
; every guarded function — one of the most common idioms in real Elixir code.
(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (call
        target: (identifier) @name)))
  (#eq? @_kw "def")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (identifier) @name))
  (#eq? @_kw "def")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (call
        target: (identifier) @name)))
  (#eq? @_kw "defp")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (identifier) @name))
  (#eq? @_kw "defp")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (call
        target: (identifier) @name)))
  (#eq? @_kw "defmacro")) @definition.macro

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (identifier) @name))
  (#eq? @_kw "defmacro")) @definition.macro

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (call
        target: (identifier) @name)))
  (#eq? @_kw "defmacrop")) @definition.macro

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (identifier) @name))
  (#eq? @_kw "defmacrop")) @definition.macro

; defguard/defguardp: `defguard name(args) when guard-expr`. Structurally a
; guarded call just like the forms above (arguments -> binary_operator ->
; left: call), but defguard's guard clause is not optional (there is no
; unguarded defguard form in real Elixir), so only this shape is needed.
(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (call
        target: (identifier) @name)))
  (#eq? @_kw "defguard")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (identifier) @name))
  (#eq? @_kw "defguard")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (call
        target: (identifier) @name)))
  (#eq? @_kw "defguardp")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (binary_operator
      left: (identifier) @name))
  (#eq? @_kw "defguardp")) @definition.function

; defdelegate: `defdelegate name(args), to: Module`. Same call/identifier
; argument shape as def/defp (no guard involved — the "to:" target is a
; keyword pair, not part of the name-bearing argument).
(call
  target: (identifier) @_kw
  (arguments
    (call
      target: (identifier) @name))
  (#eq? @_kw "defdelegate")) @definition.function

(call
  target: (identifier) @_kw
  (arguments
    (identifier) @name)
  (#eq? @_kw "defdelegate")) @definition.function

; Dynamic/macro-generated def names (`def unquote(name)(x) do ... end`) are
; grammar-legal — `call.target` allows `call` in addition to `identifier`/
; `dot` (verified via `normalize syntax query`: `unquote(name)(1, 2)` parses
; as `call target: (call)`) — but the name is only known at macro-expansion
; time, not statically from the source text. There is no honest static name
; to capture here, so this form is intentionally left unmatched rather than
; fabricating a name from the `unquote(...)` call text.
