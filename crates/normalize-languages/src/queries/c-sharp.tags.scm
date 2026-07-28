; C# tags query

; Class declarations
(class_declaration
  name: (identifier) @name) @definition.class

; Struct declarations
(struct_declaration
  name: (identifier) @name) @definition.class

; Interface declarations
(interface_declaration
  name: (identifier) @name) @definition.interface

; Enum declarations
(enum_declaration
  name: (identifier) @name) @definition.enum

; Record declarations
(record_declaration
  name: (identifier) @name) @definition.class

; Namespace declarations
(namespace_declaration
  name: (_) @name) @definition.module

; File-scoped namespace declarations
(file_scoped_namespace_declaration
  name: (_) @name) @definition.module

; Method declarations
(method_declaration
  name: (identifier) @name) @definition.method

; Constructor declarations
(constructor_declaration
  name: (identifier) @name) @definition.method

; Property declarations
(property_declaration
  name: (identifier) @name) @definition.method

; Local function statements
(local_function_statement
  name: (identifier) @name) @definition.function

; Method invocations (as call references, mirroring c-sharp.calls.scm). See
; that file for the field-completeness cross-reference of
; invocation_expression.function and member_access_expression.name against
; arborium-c-sharp 2.17.0's node-types.json (identifier vs. generic_name).
(invocation_expression
  function: (identifier) @name) @reference.call

(invocation_expression
  function: (generic_name (identifier) @name)) @reference.call

(invocation_expression
  function: (member_access_expression
    name: [(identifier) @name (generic_name (identifier) @name)])) @reference.call

(invocation_expression
  function: (conditional_access_expression
    (member_binding_expression
      name: [(identifier) @name (generic_name (identifier) @name)]))) @reference.call

; Explicit constructor invocation: base(...)/this(...) constructor delegation
; from a subclass/overload constructor. `base`/`this` are anonymous string
; tokens inside `constructor_initializer` (no field, no distinct named node
; kind) — entirely unmatched before this fix, so every constructor that
; delegates to a base or sibling overload constructor silently disappeared
; from call extraction, exactly like Java's explicit_constructor_invocation gap.
(constructor_initializer "base" @name) @reference.call

(constructor_initializer "this" @name) @reference.call

; Object creation: `new Foo()`, `new List<int>()`, `new System.Text.StringBuilder()`.
; `object_creation_expression.type` is the `type` supertype, whose
; grammar-legal variants include plain identifier, generic_name (near-
; ubiquitous — `new List<T>()`), and qualified_name (`new System.Random()`).
(object_creation_expression
  type: (identifier) @name) @reference.class

(object_creation_expression
  type: (generic_name (identifier) @name)) @reference.class

(object_creation_expression
  type: (qualified_name name: (identifier) @name)) @reference.class

; Base list: `class Foo : Base, IBar, IBaz<T> { }`. Unlike Java/TypeScript,
; the C# grammar has NO distinct `extends`/`implements` clause — a single
; `base_list` node holds the (optional) base class AND every implemented
; interface as an undifferentiated sequence of `type` children, with no
; syntactic marker distinguishing which entry is the superclass. Per
; CLAUDE.md's "be honest about capabilities": since the CST genuinely cannot
; tell them apart (semantic knowledge — e.g. "does this simple name resolve
; to a class or an interface" — would be required, which is out of scope for
; a syntactic query), every base_list entry is captured uniformly as
; @reference.class rather than fabricating an extends/implements split.
; This entire base_list handling was previously completely absent — every
; superclass and every implemented interface silently disappeared from tags.
(base_list
  (identifier) @name) @reference.class

(base_list
  (generic_name (identifier) @name)) @reference.class

(base_list
  (qualified_name name: (identifier) @name)) @reference.class

; Primary-constructor base type: `record Person(...) : PersonBase(...)`,
; `class Foo(int x) : Base(x)`. A distinct node kind (`primary_constructor_
; base_type`, holding the base type PLUS its constructor argument_list) used
; specifically for primary-constructor inheritance — grammar-legal for both
; classes and records with primary constructors, a common modern C# idiom
; (records especially). Entirely unhandled before this fix.
(base_list
  (primary_constructor_base_type
    type: (identifier) @name)) @reference.class

(base_list
  (primary_constructor_base_type
    type: (generic_name (identifier) @name))) @reference.class

(base_list
  (primary_constructor_base_type
    type: (qualified_name name: (identifier) @name))) @reference.class
