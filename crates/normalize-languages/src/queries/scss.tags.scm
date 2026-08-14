; SCSS tags — mixins, functions, rule sets, at-rules, declarations

(mixin_statement
  name: (identifier) @name) @definition.function

(function_statement
  name: (identifier) @name) @definition.function

(rule_set
  (selectors) @name) @definition.class

(media_statement) @definition.module

(supports_statement) @definition.module

(keyframes_statement
  (keyframes_name) @name) @definition.module

; Generic at-rule fallback: @font-face, @layer, @property, @page, and any
; other at-rule this grammar has no dedicated node type for all parse
; uniformly as `at_rule`, exactly as in css.tags.scm (confirmed via
; `normalize syntax ast`: @font-face/@page/@property/@layer all produce this
; node). NOT extended to @container here — unlike arborium-css,
; arborium-scss 2.17.0 produces an ERROR node for `@container (condition)
; { ... }` (confirmed via `normalize syntax ast`); the parenthesized
; condition is unparseable in this grammar's `at_rule` rule even though the
; bare block form and @font-face/@page/@layer/@property all parse cleanly.
(at_rule) @definition.module

(declaration
  (property_name) @name) @definition.variable

; @include mixin-name; / @include mixin-name(args);
; See scss.calls.scm's header comment: `include_statement`'s mixin name is a
; direct `identifier` child, not wrapped in a call_expression.
(include_statement
  (identifier) @name) @reference.call

; NOT HANDLED — grammar limitation, not a query gap: `@extend %placeholder;`
; (the primary real-world use of placeholder selectors) produces an ERROR
; node in arborium-scss 2.17.0 — `extend_statement`'s children field list
; allows `class_selector`/`string_value`/etc. but not `placeholder`
; (confirmed via `normalize syntax ast` and node-types.json). `@extend
; .some-class;` (extending a real class selector) parses cleanly as
; `extend_statement > class_selector` but is not captured as a
; @reference.* here — deliberately out of scope for this pass, since it
; would be the query's first selector-as-reference capture and needs a
; dedicated design decision (which capture kind, dedup against the
; definition-side rule_set selector) rather than a drive-by addition.
