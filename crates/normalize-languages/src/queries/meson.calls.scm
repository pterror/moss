; Meson calls query
; @call — function/method call nodes
; @call.qualifier — object receiver for method calls
;
; Meson represents standalone function calls as `normal_command` nodes with a
; `command` field. Method calls on objects appear as `expression_statement`
; nodes with `object` and `function` fields.
;
; Per node-types.json, `expression_statement.object` allows THREE types --
; `identifier` (`dep.method()`), `string` (`'template'.format(...)`, a very
; common Meson idiom), and `listitem` (`files[0].strip()`, method calls on
; an indexed list/dict element) -- verified all three actually occur via
; `normalize syntax query` against probe source. The previous version of
; this query only handled the `identifier` case, silently dropping the
; qualifier for string-literal and indexed-element receivers.
;
; `expression_statement.function` also allows `expression_statement` in
; addition to `normal_command`: chained method calls (`dep.version().strip()`)
; parse as a FLAT chain -- the outer `expression_statement`'s `function`
; field is itself an `expression_statement` wrapping the first call in the
; chain, not a nested `object.method()` shape. The trailing pattern below
; reaches through that one level to tag the chain's leading call with the
; original receiver as its qualifier; later calls in the chain are still
; captured by the unconditional `normal_command` pattern below (their
; receiver is a call result, not a named entity, so no qualifier applies).

; Standalone function call: func_name(args)
; Also matches every call regardless of nesting depth (chained method
; calls included), since normal_command's shape is the same everywhere.
(normal_command
  command: (identifier) @call)

; Method call on object: obj.method(args)
(expression_statement
  object: (identifier) @call.qualifier
  function: (normal_command
    command: (identifier) @call))

; Method call on a string literal: 'template'.format(args)
(expression_statement
  object: (string) @call.qualifier
  function: (normal_command
    command: (identifier) @call))

; Method call on an indexed list/dict element: files[0].strip()
(expression_statement
  object: (listitem) @call.qualifier
  function: (normal_command
    command: (identifier) @call))

; Chained method call: obj.method1().method2()... — tag the qualifier of
; the chain's leading call.
(expression_statement
  object: (_) @call.qualifier
  function: (expression_statement
    .
    function: (normal_command
      command: (identifier) @call)))
