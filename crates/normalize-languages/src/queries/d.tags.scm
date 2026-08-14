; D tags query
; D grammar nodes have no fields — names are positional identifier children

(class_declaration
  (identifier) @name) @definition.class

; Generic class: class Foo(T) { ... } — a distinct node type from
; class_declaration, not a class_declaration wrapped in a template.
(class_template_declaration
  (identifier) @name) @definition.class

(struct_declaration
  (identifier) @name) @definition.class

; Generic struct: struct Foo(T) { ... } — distinct node type, see above.
(struct_template_declaration
  (identifier) @name) @definition.class

(union_declaration
  (identifier) @name) @definition.class

; Generic union: union Foo(T) { ... } — distinct node type, see above.
(union_template_declaration
  (identifier) @name) @definition.class

(interface_declaration
  (identifier) @name) @definition.interface

; Generic interface: interface Foo(T) { ... } — distinct node type, see above.
(interface_template_declaration
  (identifier) @name) @definition.interface

(enum_declaration
  (identifier) @name) @definition.type

(func_declaration
  (func_declarator
    (identifier) @name)) @definition.function

; auto-return-type functions: auto foo() { ... } — auto_func_declaration is a
; distinct node type from func_declaration (no func_declarator wrapper; the
; identifier is a direct child).
(auto_func_declaration
  (identifier) @name) @definition.function
