; PHP imports query
; @import       — the entire use/include/require statement (for line number)
; @import.path  — the namespace path or file path string
; @import.name  — a single name from a grouped use declaration
; @import.alias — alias after 'as'

; use Namespace\Class;
; `!alias` excludes clauses that also carry an `as` alias — without it, this
; pattern and the alias-form pattern below both matched every aliased
; import, double-counting @import.path for every `use X as Y;` (confirmed
; via `normalize syntax query`: `use App\Models\User as U;` produced two
; identical @import.path captures before this fix).
(namespace_use_declaration
  (namespace_use_clause
    (qualified_name) @import.path
    !alias)) @import

; use Namespace\Class as Alias;
(namespace_use_declaration
  (namespace_use_clause
    (qualified_name) @import.path
    alias: (name) @import.alias)) @import

; use Exception; use Throwable as T; — a single-segment (no namespace
; separator) `use` clause parses as bare `name`, not `qualified_name`. This
; is extremely common for global/built-in classes (Exception, Throwable,
; ArrayAccess, Countable, Iterator, Closure, stdClass, DateTime, …) and was
; previously dropped entirely — the query only ever matched `qualified_name`.
(namespace_use_declaration
  (namespace_use_clause
    (name) @import.path
    !alias)) @import

(namespace_use_declaration
  (namespace_use_clause
    (name) @import.path
    alias: (name) @import.alias)) @import

; use function Namespace\func; (also handled by namespace_use_declaration above)

; use const Namespace\CONST; (handled by namespace_use_declaration above)

; Grouped use: use App\{Foo, Bar as B}; — each grouped member is its own
; namespace_use_clause, but nested one level deeper than the plain form: it
; sits inside a namespace_use_group child of namespace_use_declaration, not
; as a direct child of namespace_use_declaration itself. Tree-sitter query
; nesting requires a DIRECT child match (verified: the four patterns above
; produced zero matches for any grouped member), so grouped `use` was
; entirely unmatched until these four group-nested duplicates were added.
; The group's own `App\` prefix is still not concatenated onto each
; member's path (path concatenation is a resolver-level concern, matching
; this file's existing behavior for the non-grouped qualified_name case).
(namespace_use_declaration
  (namespace_use_group
    (namespace_use_clause
      (qualified_name) @import.path
      !alias))) @import

(namespace_use_declaration
  (namespace_use_group
    (namespace_use_clause
      (qualified_name) @import.path
      alias: (name) @import.alias))) @import

(namespace_use_declaration
  (namespace_use_group
    (namespace_use_clause
      (name) @import.path
      !alias))) @import

(namespace_use_declaration
  (namespace_use_group
    (namespace_use_clause
      (name) @import.path
      alias: (name) @import.alias))) @import

; Trait composition: use TraitA, TraitB; inside a class/trait body. This is
; a distinct node kind (`use_declaration`, not `namespace_use_declaration`)
; from the top-level namespace imports above — it composes methods into the
; current class/trait rather than importing a namespace path, the same role
; Ruby's `include`/`extend`/`prepend` play (see ruby.imports.scm), so it's
; captured here rather than in tags.scm. Was entirely unmatched before this
; fix. `use_declaration`'s children allow name/qualified_name/relative_name
; directly (multiple traits are sibling children, not wrapped in a list) —
; `use_list` only appears for the separate conflict-resolution clause
; (`use A, B { A::foo insteadof B; }`), not covered here.
(use_declaration
  [
    (name)
    (qualified_name)
    (relative_name)
  ] @import.path) @import

; include 'file.php'
(include_expression
  (string
    (string_content) @import.path)) @import

; include_once 'file.php'
(include_once_expression
  (string
    (string_content) @import.path)) @import

; require 'file.php' / require_once 'file.php' — distinct node kinds from
; include_expression/include_once_expression (not variants of the same
; node), so the include-only patterns above never matched them. `require`
; is the far more common file-inclusion form in real PHP (it fatals on
; failure instead of warning), so this was a significant, previously-silent
; gap: no `require`/`require_once` ever showed up as an import.
(require_expression
  (string
    (string_content) @import.path)) @import

(require_once_expression
  (string
    (string_content) @import.path)) @import

; require_once __DIR__ . '/bootstrap.php'; — the extremely common
; "concatenate a directory constant with a relative path" idiom. The
; argument is a `binary_expression` (concatenation), not a bare string, so
; none of the four direct-string patterns above match it (verified via
; `normalize syntax query`: this exact construct produced zero matches
; before this fix). Captures the string-literal suffix as a best-effort
; partial signal — the same "partial signal over silence" tradeoff
; ruby.types.scm makes for `class Foo < Struct.new(...)`. Genuinely
; non-literal, fully-computed paths (e.g. built from a variable) are left
; uncaptured rather than fabricated.
(include_expression
  (binary_expression
    right: (string
      (string_content) @import.path))) @import

(include_once_expression
  (binary_expression
    right: (string
      (string_content) @import.path))) @import

(require_expression
  (binary_expression
    right: (string
      (string_content) @import.path))) @import

(require_once_expression
  (binary_expression
    right: (string
      (string_content) @import.path))) @import
