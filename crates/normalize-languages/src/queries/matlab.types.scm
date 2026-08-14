; MATLAB type references
; Captures type names used in class inheritance (superclass references).
;
; MATLAB class definitions use `<` to list superclasses:
;   classdef Foo < Bar & Baz
; BUG FIX (verified against arborium-matlab 2.17.0 node-types.json and real
; parse output via a GrammarLoader probe — NOT the CLI, since `.m` is
; ambiguous with Objective-C): the classdef superclass list does NOT parse
; as a `superclass` node at all. It is `class_definition`'s `superclasses`
; child, which wraps one `property_name` per parent, each wrapping an
; `identifier`:
;   (class_definition name: (identifier)
;     (superclasses (property_name (identifier)) (property_name (identifier))))
; The original query — `(superclass (identifier) @type.reference)` — matched
; a real grammar node type, but one that never appears in ordinary classdef
; inheritance, so it silently produced ZERO captures for every classdef in
; this crate's own sample fixture (`classdef Shape < handle`) and for any
; realistic multi-superclass form (`classdef Foo < Bar & Baz`).

; Superclass reference(s) in classdef: classdef Foo < Bar & Baz
(superclasses
  (property_name
    (identifier) @type.reference))

; `superclass` IS a real node type — but it appears as an optional child of
; `function_call`, for MATLAB's explicit superclass-qualified method-call
; syntax (calling an overridden superclass method/constructor from a
; subclass method body):
;   obj = obj@Bar();
; which parses as:
;   (function_call name: (identifier) (superclass (identifier)))
; This also names a real superclass, so it belongs in this query's stated
; scope ("superclass references") alongside the classdef form above.
(superclass
  (identifier) @type.reference)
