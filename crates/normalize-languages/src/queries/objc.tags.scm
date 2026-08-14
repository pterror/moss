; Objective-C tags query
; class_interface, class_implementation, and protocol_declaration have no name field —
; the name is a positional identifier child.

; class_interface/class_implementation ALSO carry `category` and `superclass`
; fields typed (identifier) — the exact same node type as the class name
; itself. An unanchored `(identifier) @name` (the original pattern here)
; therefore matched the superclass identifier (`@interface Point : NSObject`
; -> both "Point" AND "NSObject" captured as @name) and the category
; identifier (`@interface Point (Utilities)` -> both "Point" AND "Utilities")
; for every class with a superclass or category, corrupting every symbol
; index built from this query. Verified via `normalize syntax query` against
; a probe file. The `.` anchor pins the match to the first child, which is
; always the class name itself (verified: name always precedes `:
; superclass` and `(category)` positionally).
(class_interface
  . (identifier) @name) @definition.class

(class_implementation
  . (identifier) @name) @definition.class

; protocol_declaration has no superclass/category fields, so its
; (identifier) child is unambiguous, AND the `<Protocol, ...>` conformance
; list identifiers live one level deeper inside `protocol_reference_list`
; (not a direct child), so this pattern was already correct as written.
(protocol_declaration
  (identifier) @name) @definition.interface

; C-style function declarator: matches both `function_definition` (has a
; body) and bare prototypes (`double distance(Point *a, Point *b);`, no
; body — very common in ObjC headers). The old pattern here only matched
; through `function_definition`, silently dropping every bare prototype
; (verified 0 matches via `normalize syntax query` before this fix, vs.
; correct match after). Mirrors c.tags.scm's `function_declarator` pattern.
(function_declarator
  declarator: (identifier) @name) @definition.function

; #define NAME value / #define NAME(args) body — macro definitions, ported
; from c.tags.scm (objc shares the C preprocessor grammar). Verified via
; probe: previously 0 macro coverage at all in objc.tags.scm.
(preproc_def name: (identifier) @name) @definition.macro
(preproc_function_def name: (identifier) @name) @definition.macro

; Header guards (`#ifndef FOO_H` / `#define FOO_H` with no value / `#endif`)
; are structural noise, not meaningful symbols. Ported verbatim from
; c.tags.scm — see that file's comment for the full rationale. `@_suppress`
; drops the guard's `@definition.macro` capture so it never becomes a
; symbol, without affecting macros that happen to share the guard's name
; but carry a real value.
(preproc_ifdef
  name: (identifier) @_guard.cond
  (preproc_def
    name: (identifier) @_guard.name
    !value) @_suppress
  (#eq? @_guard.cond @_guard.name))

(type_definition
  declarator: (type_identifier) @name) @definition.type

; Typedef'd function pointer: `typedef int (*Comparator)(int, int);` — ported
; from c.tags.scm. The alias name nests three levels deep (function_declarator
; > parenthesized_declarator > pointer_declarator > type_identifier), not as
; a direct `type_definition.declarator` child, so the plain pattern above
; silently drops this callback-type idiom. Verified via probe.
(type_definition
  declarator: (function_declarator
    declarator: (parenthesized_declarator
      (pointer_declarator
        declarator: (type_identifier) @name)))) @definition.type

(struct_specifier
  name: (type_identifier) @name
  body: (_)) @definition.class

; Union definitions (`union Value { ... };`) — ported from c.tags.scm; had
; zero coverage in objc.tags.scm despite being valid, if rare, ObjC/C syntax.
(union_specifier
  name: (type_identifier) @name
  body: (_)) @definition.class

(enum_specifier
  name: (type_identifier) @name) @definition.type

; ---------------------------------------------------------------------------
; Objective-C methods (method_declaration in @interface/@protocol prototypes,
; method_definition in @implementation bodies) — previously had ZERO tags
; coverage despite being the primary unit of ObjC code. Neither node has
; field names (verified via node-types.json: only a flat `children` list),
; so the selector head can't be reached with a `field:` constraint. Every
; method_declaration/method_definition begins with a literal `-` or `+`
; token, then a `(ReturnType)` method_type, then the first identifier of the
; selector (`area` for a no-arg method, `initWithX` for the first segment of
; a multi-keyword selector `initWithX:y:`). Anchoring on that fixed prefix
; with `.` selects exactly the selector head and nothing else — an
; unanchored `(identifier) @name` would also match every later selector
; segment and parameter name (verified via probe, same class of bug as the
; class_interface fix above). Multi-keyword selectors are NOT reconstructed
; into `initWithX:y:` — the grammar gives no single node spanning the full
; selector, only the first segment, which is the best available without
; fabricating structure (mirrors objc.calls.scm's message_expression note).
(method_declaration
  . ["-" "+"] . (method_type) . (identifier) @name) @definition.method

(method_definition
  . ["-" "+"] . (method_type) . (identifier) @name) @definition.method
