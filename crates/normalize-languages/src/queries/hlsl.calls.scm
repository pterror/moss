; HLSL calls query
; HLSL is C-like; node types mirror tree-sitter-c.
; @call — function being called
; @call.qualifier — receiver for member/pointer calls

; Simple call: func()
(call_expression
  function: (identifier) @call)

; Member/method call: obj.Method()
(call_expression
  function: (field_expression
    argument: (_) @call.qualifier
    field: (field_identifier) @call))

; Templated member call: buf.Load<float4>(addr), tex.Gather<uint>(...)
; `field_expression.field` allows `template_method` (verified in
; node-types.json), whose own `.name` field strips the `<...>` template
; argument list — mirrors the analogous Rust turbofish-call fix. Common for
; typed resource loads on ByteAddressBuffer/StructuredBuffer/Texture*.
(call_expression
  function: (field_expression
    argument: (_) @call.qualifier
    field: (template_method
      name: (field_identifier) @call)))

; Templated free function call: Identity<float4>(v)
; `call_expression.function` allows `template_function` (SM6.x-style
; generic/template functions); `.name` strips the template argument list.
(call_expression
  function: (template_function
    name: (identifier) @call))
