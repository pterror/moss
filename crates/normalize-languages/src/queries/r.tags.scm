; R tags query
;
; R functions are assigned: name <- function(...) {...}
; In the tree-sitter grammar, assignments are binary_operator nodes.
; We match the left-assignment form with a function_definition RHS.

; name <- function(...)
(binary_operator
  lhs: (identifier) @name
  operator: "<-"
  rhs: (function_definition)) @definition.function

; name = function(...)
(binary_operator
  lhs: (identifier) @name
  operator: "="
  rhs: (function_definition)) @definition.function

; name <<- function(...)  (global assignment)
(binary_operator
  lhs: (identifier) @name
  operator: "<<-"
  rhs: (function_definition)) @definition.function

; self$method <- function(...) / self@method <- function(...)
; `extract_operator` is the `$`/`@` (slot) accessor node — `binary_operator.lhs`
; permits it per node-types.json. This is the standard R6/Reference-Class/
; environment-based OOP method-definition idiom (self$run <- function() {...}).
(binary_operator
  lhs: (extract_operator
    rhs: (identifier) @name)
  operator: "<-"
  rhs: (function_definition)) @definition.function

(binary_operator
  lhs: (extract_operator
    rhs: (identifier) @name)
  operator: "="
  rhs: (function_definition)) @definition.function

; (function(...) {...}) -> name  /  ->>  (right-assignment)
; R's right-assign operators have lower precedence than a bare
; `function(...) body`, so the function must be parenthesized for the
; `->`/`->>` to bind the whole definition rather than just its body — see
; probe verification in the R query-completeness sweep.
(binary_operator
  lhs: (parenthesized_expression
    body: (function_definition))
  operator: "->"
  rhs: (identifier) @name) @definition.function

(binary_operator
  lhs: (parenthesized_expression
    body: (function_definition))
  operator: "->>"
  rhs: (identifier) @name) @definition.function
