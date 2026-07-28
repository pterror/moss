; C# type references
; Captures identifiers used in type positions.
;
; Unlike languages whose grammar gives type identifiers a distinct node kind
; (e.g. Java/Rust's `type_identifier`), arborium-c-sharp 2.17.0 reuses the
; plain `identifier` node for BOTH type and value positions — there is no
; `type_identifier` kind in this grammar's node-types.json at all. The
; previous `(identifier) @type.reference` pattern below was therefore
; completely unconstrained and matched every identifier anywhere in the
; file — method names, parameter names, local variable names, everything —
; not just type positions. Verified on the pre-existing sample.cs fixture:
; it produced 103 captures including "Push", "Add", "item", "items", none of
; which are type references. This is a severe, previously-silent correctness
; bug (dimension 1), not merely a completeness gap.
;
; The fix: constrain to the actual `type:`/`returns:` field positions
; documented in node-types.json (variable_declaration, parameter,
; method_declaration.returns, property_declaration, local_function_
; statement, foreach_statement, catch_declaration, cast_expression,
; is_expression/as_expression, object_creation_expression), covering the
; three practically-occurring leaf shapes of the `type` supertype: plain
; `identifier`, `generic_name` (generics), and `qualified_name` (dotted
; paths) — plus one level of `nullable_type` unwrapping for C# 8+ nullable
; reference types (`Foo? x`), a heavily-used modern idiom. Rarer `type`
; supertype members (`pointer_type`, `function_pointer_type`, `ref_type`,
; `tuple_type`, `array_type` element types) are unsafe-code/advanced-generics
; constructs with negligible real-world density; left undocumented rather
; than fabricating exhaustive handling for them.

(variable_declaration
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)
         (nullable_type type: [(identifier) @type.reference
                                (generic_name (identifier) @type.reference)
                                (qualified_name name: (identifier) @type.reference)])])

(parameter
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)
         (nullable_type type: [(identifier) @type.reference
                                (generic_name (identifier) @type.reference)
                                (qualified_name name: (identifier) @type.reference)])])

(method_declaration
  returns: [(identifier) @type.reference
            (generic_name (identifier) @type.reference)
            (qualified_name name: (identifier) @type.reference)
            (nullable_type type: [(identifier) @type.reference
                                   (generic_name (identifier) @type.reference)
                                   (qualified_name name: (identifier) @type.reference)])])

(local_function_statement
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)
         (nullable_type type: [(identifier) @type.reference
                                (generic_name (identifier) @type.reference)
                                (qualified_name name: (identifier) @type.reference)])])

(property_declaration
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)
         (nullable_type type: [(identifier) @type.reference
                                (generic_name (identifier) @type.reference)
                                (qualified_name name: (identifier) @type.reference)])])

(foreach_statement
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)])

(catch_declaration
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)])

(cast_expression
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)])

(is_expression
  right: [(identifier) @type.reference
          (generic_name (identifier) @type.reference)
          (qualified_name name: (identifier) @type.reference)])

(as_expression
  right: [(identifier) @type.reference
          (generic_name (identifier) @type.reference)
          (qualified_name name: (identifier) @type.reference)])

(object_creation_expression
  type: [(identifier) @type.reference
         (generic_name (identifier) @type.reference)
         (qualified_name name: (identifier) @type.reference)])

; Type-defining declarations: classes, structs, interfaces, enums, and
; records are all definitions of a named type. Previously entirely absent
; from c-sharp.types.scm (unlike java.types.scm's equivalent set, added in
; the batch-1 sweep) — no @definition.type at all.
(class_declaration name: (identifier) @name) @definition.type

(struct_declaration name: (identifier) @name) @definition.type

(interface_declaration name: (identifier) @name) @definition.type

(enum_declaration name: (identifier) @name) @definition.type

(record_declaration name: (identifier) @name) @definition.type
