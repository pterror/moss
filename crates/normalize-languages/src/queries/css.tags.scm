; CSS symbols: selectors as classes, at-rules as modules, declarations as variables.
;
; This grammar declares no `fields` at all (verified against arborium-css
; 2.17.0's node-types.json — every node type's "fields" object is empty), so
; unlike field-based grammars (Rust, C#, SQL) there is nothing to
; field-constrain here; completeness instead means "every distinct at-rule
; node type the grammar produces is captured".

(rule_set
  (selectors) @name) @definition.class

(media_statement) @definition.module

(supports_statement) @definition.module

(keyframes_statement
  (keyframes_name) @name) @definition.function

; @scope (.card) to (.content) { ... } — CSS Scoping (scoped style boundary).
(scope_statement) @definition.module

; Generic at-rule fallback: @font-face, @layer, @property, @container, @page,
; and any other at-rule this grammar has no dedicated node type for all parse
; uniformly as `at_rule` (an `at_keyword` child plus an optional `block`) —
; confirmed via real parse (`normalize syntax ast`) that @font-face, @layer,
; @property, @container, and @page all produce this same generic node, with
; no syntactic way to distinguish "this is specifically @font-face" from the
; tree shape alone. Matches both the block form (`@font-face { ... }`) and
; the statement form (`@layer a, b, c;`); name extraction in css.rs decides
; whether to render a trailing `{ … }` based on whether a block is present.
(at_rule) @definition.module

(declaration
  (property_name) @name) @definition.var
