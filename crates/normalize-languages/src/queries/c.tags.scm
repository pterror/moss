; Vendored from https://github.com/tree-sitter/tree-sitter-c
; License: MIT

(struct_specifier name: (type_identifier) @name body:(_)) @definition.class

; Union definitions: bare (`union Foo { ... };`) and typedef'd
; (`typedef union Baz { ... } BazT;`) both parse as a top-level
; `union_specifier` with name+body, exactly like `struct_specifier` above —
; NOT as `declaration type: (union_specifier ...)`. That shape only matches a
; *bodyless* union-typed variable declaration (`union Bar instance;`), which
; is a usage of an existing tag, not a definition; the old pattern here
; mislabeled that usage as @definition.class while missing every real union
; definition (verified via `normalize syntax query` against a probe file).
(union_specifier name: (type_identifier) @name body: (_)) @definition.class

(function_declarator declarator: (identifier) @name) @definition.function

; #define NAME value / #define NAME(args) body — macro definitions never
; had any tags coverage; `normalize view --types-only` on any C header full
; of macro constants or function-like macros previously reported nothing.
(preproc_def name: (identifier) @name) @definition.macro
(preproc_function_def name: (identifier) @name) @definition.macro

; Header guards (`#ifndef FOO_H` / `#define FOO_H` with no value / `#endif`)
; are structural noise, not meaningful symbols — a user reading `normalize
; view --types-only` on a header wants its real declarations, not the guard
; token. This is a structural pattern (a valueless `preproc_def` whose name
; matches the enclosing `preproc_ifdef`'s condition), not a naming-convention
; hack: `#eq?` compares the guard condition to the macro name, and `!value`
; requires the define to carry no value, so a *meaningful* macro that
; happens to share its ifdef's name (`#ifndef DEBUG` / `#define DEBUG 1`,
; verified via `normalize syntax query` to still match the plain macro
; pattern above since it has a value) is not affected. `@_suppress` is a
; generic capture convention (see `collect_symbols_from_tags` in
; normalize-facts): any node captured `@_suppress` by any pattern in this
; query has its `@definition.*` capture (from this or any other pattern)
; dropped, so it never becomes a symbol.
(preproc_ifdef
  name: (identifier) @_guard.cond
  (preproc_def
    name: (identifier) @_guard.name
    !value) @_suppress
  (#eq? @_guard.cond @_guard.name))

(type_definition declarator: (type_identifier) @name) @definition.type

; Typedef'd function pointer: `typedef int (*FuncPtr)(int, int);` — the
; grammar nests the alias name three levels deep (function_declarator >
; parenthesized_declarator > pointer_declarator > type_identifier), not as a
; direct `type_definition.declarator` child. A common callback-type idiom
; (POSIX handlers, qsort comparators, etc.) that the plain pattern above
; silently dropped.
(type_definition
  declarator: (function_declarator
    declarator: (parenthesized_declarator
      (pointer_declarator
        declarator: (type_identifier) @name)))) @definition.type

(enum_specifier name: (type_identifier) @name) @definition.type
