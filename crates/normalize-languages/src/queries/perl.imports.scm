; Perl imports query
; @import       — the entire import statement (for line number)
; @import.path  — the module path being imported

; use Module::Name;
(use_statement
  module: (package) @import.path) @import

; require Module::Name; or require 'file.pl';
; require_expression has no named field for its argument; the module name
; and the file-path form parse as structurally distinct child node types
; (verified via `normalize syntax query -p <probe> "(require_expression
; (bareword) @c)"` / "(require_expression (string_literal) @c)"` against
; `require Foo::Bar;` and `require 'file.pl';` probes — the module form is
; a `bareword`, not a `package` node, despite the module path looking
; identical to a `use` statement's argument).
(require_expression
  (bareword) @import.path) @import

(require_expression
  (string_literal) @import.path) @import
