; Vendored from https://github.com/tree-sitter/tree-sitter-java
; License: MIT

(class_declaration
  name: (identifier) @name) @definition.class

(method_declaration
  name: (identifier) @name) @definition.method

(method_invocation
  name: (identifier) @name
  arguments: (argument_list) @reference.call)

; Explicit constructor invocation: super(...) / this(...) constructor
; delegation (distinct node kind from method_invocation; see java.calls.scm).
(explicit_constructor_invocation
  constructor: (super) @name) @reference.call

(explicit_constructor_invocation
  constructor: (this) @name) @reference.call

(interface_declaration
  name: (identifier) @name) @definition.interface

(enum_declaration
  name: (identifier) @name) @definition.enum

; Records (Java 16+): record Point(int x, int y) {}
; Mapped to @definition.class (not a distinct "record" kind — SymbolKind has
; no Record variant): a record compiles to a final class extending
; java.lang.Record, so "class" is the closest accurate existing kind.
(record_declaration
  name: (identifier) @name) @definition.class

; Annotation type declarations: @interface Foo { String value(); }
; Mapped to @definition.interface: the JVM spec compiles annotation types to
; interfaces extending java.lang.annotation.Annotation.
(annotation_type_declaration
  name: (identifier) @name) @definition.interface

; `implements`/`extends-interfaces` lists (type_list). Each element's field
; is `_type`, whose grammar-legal variants include plain, generic, and
; path-qualified forms — all common (`implements Comparable<Foo>`,
; `implements List<T>`, `implements java.io.Serializable`).

(type_list
  (type_identifier) @name) @reference.implementation

(type_list
  (generic_type
    (type_identifier) @name)) @reference.implementation

(type_list
  (generic_type
    (scoped_type_identifier
      (type_identifier) @name .))) @reference.implementation

(type_list
  (scoped_type_identifier
    (type_identifier) @name .)) @reference.implementation

; Object creation: `new Foo()`, `new ArrayList<>()`, `new java.util.HashMap<>()`.
; `object_creation_expression.type` is `_simple_type`, whose grammar-legal
; variants include plain type_identifier, generic_type (near-ubiquitous —
; `new ArrayList<>()`), and scoped_type_identifier (`new java.util.Date()`).

(object_creation_expression
  type: (type_identifier) @name) @reference.class

(object_creation_expression
  type: (generic_type
    (type_identifier) @name)) @reference.class

(object_creation_expression
  type: (generic_type
    (scoped_type_identifier
      (type_identifier) @name .))) @reference.class

(object_creation_expression
  type: (scoped_type_identifier
    (type_identifier) @name .)) @reference.class

; `extends` clause (superclass). Same `_type` variant set as above:
; `extends AbstractList<String>`, `extends java.util.AbstractList`.

(superclass (type_identifier) @name) @reference.class

(superclass
  (generic_type
    (type_identifier) @name)) @reference.class

(superclass
  (generic_type
    (scoped_type_identifier
      (type_identifier) @name .))) @reference.class

(superclass
  (scoped_type_identifier
    (type_identifier) @name .)) @reference.class
