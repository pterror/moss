; Scheme tags query
;
; Scheme uses list nodes for all forms.
; The first child symbol names the form.

; (define (name args) body)
(list
  (symbol) @_kw (#eq? @_kw "define")
  .
  (list
    .
    (symbol) @name)) @definition.function

; (define name (lambda ...)) / (define name (case-lambda ...))
(list
  (symbol) @_kw (#eq? @_kw "define")
  .
  (symbol) @name
  .
  (list
    (symbol) @_lambda (#match? @_lambda "^(lambda|case-lambda)$"))) @definition.function

; (define name value) — constant/variable.
;
; Anchored so `value` must be the LAST child (the trailing `.`), and
; excludes the lambda/case-lambda-value case via `#not-match?` on the
; captured value text. Without both of these, this pattern — which
; previously had no anchor on `value` at all and matched on `define` +
; `name` alone — also matched `(define name (lambda ...))`, producing BOTH
; `@definition.constant` and `@definition.function` on the same node.
; Confirmed via `normalize syntax query`: `(define add (lambda (a b) (+ a
; b)))` fired both captures before this fix.
(list
  (symbol) @_kw (#eq? @_kw "define")
  .
  (symbol) @name
  .
  (_) @_value
  .
  (#not-match? @_value "^\\((lambda|case-lambda)\\b")) @definition.constant

; (define-record-type name ...)
(list
  (symbol) @_kw (#eq? @_kw "define-record-type")
  .
  (symbol) @name) @definition.class

; (define-syntax name ...)
(list
  (symbol) @_kw (#eq? @_kw "define-syntax")
  .
  (symbol) @name) @definition.macro
