; Objective-C calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; Objective-C has two call forms:
;   - C-style: func(args) — call_expression
;   - ObjC message send: [receiver selector args] — message_expression

; C-style function call: func(args)
(call_expression
  function: (identifier) @call)

; C-style with qualifier: obj->method(args)
(call_expression
  function: (field_expression
    argument: (_) @call.qualifier
    field: (field_identifier) @call))

; ObjC message send: [receiver message:arg]
;
; The grammar labels EVERY identifier segment of a multi-keyword selector
; with the SAME `method` field — not just the first (verified via probe:
; `[[Point alloc] initWithX:3.0 y:4.0]` binds `method` to "initWithX" AND
; "y", even though "y" is the second keyword segment of one selector, not a
; second call). An unanchored `method: (identifier) @call` therefore emitted
; one spurious extra @call per additional keyword segment for every
; multi-keyword message send — verified against sample.m, where it inflated
; `[[Point alloc] initWithX:3.0 y:4.0]` from 2 real calls (alloc,
; initWithX) to 3 captures (alloc, initWithX, y). The `.` anchor between
; `receiver:` and `method:` restricts the match to the method-field
; identifier immediately following the receiver — true only for the FIRST
; keyword segment, since later segments are preceded by `:` and an argument
; node, never immediately adjacent to the receiver.
(message_expression
  receiver: (_) @call.qualifier
  . method: (identifier) @call)
