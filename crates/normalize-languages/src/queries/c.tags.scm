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
