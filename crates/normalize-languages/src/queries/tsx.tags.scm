; Vendored from https://github.com/tree-sitter/tree-sitter-typescript
;
; Extended past the upstream baseline to close completeness gaps found via
; docs/query-testing-methodology.md — ported from typescript.tags.scm, whose
; grammar TSX otherwise matches exactly for non-JSX constructs (verified via
; `normalize syntax query` against arborium-tsx's node-types.json):
; method_signature/abstract_method_signature/method_definition.name allows
; computed_property_name/private_property_identifier in addition to plain
; property_identifier; new_expression.constructor allows member_expression
; (namespaced constructors, e.g. `new vscode.Position(...)`); `namespace Foo
; {}`/`namespace Foo.Bar {}` parse as `internal_module`, a node kind distinct
; from (and far more common than) the legacy `module Foo {}` keyword form,
; which was the only one previously handled; and class_heritage's
; extends/implements were entirely unhandled (no @reference.class/
; @reference.implementation for TSX classes at all). `number`/`string`-keyed
; method names (`123() {}`, `"x"() {}`) are grammar-legal per
; node-types.json but vanishingly rare in real TSX; not added.
; License: MIT

(function_declaration
  name: (identifier) @name) @definition.function

(function_signature
  name: (identifier) @name) @definition.function

; Arrow function components: const Counter: FC<...> = (...) => ...
(variable_declarator
  name: (identifier) @name
  value: (arrow_function)) @definition.function

(method_signature
  name: [(property_identifier) (private_property_identifier) (computed_property_name)] @name) @definition.method

(abstract_method_signature
  name: [(property_identifier) (private_property_identifier) (computed_property_name)] @name) @definition.method

(method_definition
  name: [(property_identifier) (private_property_identifier) (computed_property_name)] @name) @definition.method

(class_declaration
  name: (type_identifier) @name) @definition.class

(abstract_class_declaration
  name: (type_identifier) @name) @definition.class

; `module Foo {}` / `module Foo.Bar {}` / `declare module "foo" {}` (legacy
; `module` keyword; name may be a dotted path or an ambient string literal).
(module
  name: [(identifier) (nested_identifier) (string)] @name) @definition.module

; `namespace Foo {}` / `namespace Foo.Bar {}` — the modern, far more common
; keyword; parses as a distinct `internal_module` node, not `module`.
(internal_module
  name: [(identifier) (nested_identifier)] @name) @definition.module

(interface_declaration
  name: (type_identifier) @name) @definition.interface

(enum_declaration
  name: (identifier) @name) @definition.enum

(type_alias_declaration
  name: (type_identifier) @name) @definition.type

(type_annotation
  (type_identifier) @name) @reference.type

; new Foo() / new ns.Foo() (namespaced/qualified constructor)
(new_expression
  constructor: [(identifier) (member_expression)] @name) @reference.class

; class Derived extends Base {} / extends ns.Base {} / extends Mixin(Base) {}
(extends_clause
  value: (_) @name) @reference.class

; class Derived implements Logger {} (plain interface)
(implements_clause
  (type_identifier) @name) @reference.implementation

; class Derived implements Comparable<T> {} (generic interface)
(implements_clause
  (generic_type
    name: (type_identifier) @name)) @reference.implementation
