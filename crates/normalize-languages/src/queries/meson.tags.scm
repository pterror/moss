; Meson tags query
;
; Meson has no first-class function definitions; the only definable symbols
; are variables, bound via simple assignment (`x = ...`), augmented
; assignment (`x += / -= / /= ...`), or as loop variable(s) in a `foreach`.
;
; IMPORTANT: assignment in this grammar is NOT a dedicated node with
; LHS/RHS fields. `operatorunit` is the generic node for every binary
; operator expression -- assignment AND comparison AND boolean ops alike
; -- and it declares no fields at all in node-types.json, only positional
; children; the operator itself (one of `=`, `+=`, `-=`, `/=`, `==`, `!=`,
; `<`, `<=`, `>`, `>=`, `and`, `or`) is the only way to distinguish an
; assignment from e.g. a comparison. The previous version of this query
; matched `var_unit`/`variableunit` nodes on the assumption they wrapped
; the assignment's LHS name; verified against a real meson.build sample
; via `normalize syntax query`, that assumption was wrong. Those nodes
; instead wrap variable REFERENCES appearing as positional call/kwarg
; arguments or inside comparisons and ternary branches -- e.g.
; `if src == 'stop'` produces a `var_unit` wrapping the whole comparison,
; whose `value:` field points at the referenced identifier `src` -- so the
; old query captured references (and misclassified them as definitions),
; while never capturing real assignment targets such as
; `glib_dep = dependency(...)`.

; x = value
(operatorunit
  .
  (identifier) @name
  .
  "="
  .
  (_)) @definition.var

; Augmented assignment: x += / -= / /= value
(operatorunit
  .
  (identifier) @name
  .
  "+="
  .
  (_)) @definition.var

(operatorunit
  .
  (identifier) @name
  .
  "-="
  .
  (_)) @definition.var

(operatorunit
  .
  (identifier) @name
  .
  "/="
  .
  (_)) @definition.var

; foreach loop variable(s): foreach x, y : array ... endforeach
; `item` is a multiple field -- fires once per loop variable in a
; two-variable dict-iteration foreach.
(foreach_command
  item: (identifier) @name) @definition.var
