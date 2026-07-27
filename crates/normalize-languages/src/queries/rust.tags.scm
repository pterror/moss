; Vendored from https://github.com/tree-sitter/tree-sitter-rust
; License: MIT

; ADT definitions

(struct_item
    name: (type_identifier) @name) @definition.class

(enum_item
    name: (type_identifier) @name) @definition.class

(union_item
    name: (type_identifier) @name) @definition.class

; type aliases

(type_item
    name: (type_identifier) @name) @definition.class

; method definitions

(declaration_list
    (function_item
        name: (identifier) @name) @definition.method)

; function definitions

(function_item
    name: (identifier) @name) @definition.function

; trait definitions
(trait_item
    name: (type_identifier) @name) @definition.interface

; module definitions
(mod_item
    name: (identifier) @name) @definition.module

; macro definitions

(macro_definition
    name: (identifier) @name) @definition.macro

; references

(call_expression
    function: (identifier) @name) @reference.call

(call_expression
    function: (field_expression
        field: (field_identifier) @name)) @reference.call

; Scoped calls: module::func(), Type::method()

(call_expression
    function: (scoped_identifier
        name: (identifier) @name)) @reference.call

; Turbofish calls: func::<T>(), obj.method::<T>()

(call_expression
    function: (generic_function
        function: (identifier) @name)) @reference.call

(call_expression
    function: (generic_function
        function: (scoped_identifier
            name: (identifier) @name))) @reference.call

(call_expression
    function: (generic_function
        function: (field_expression
            field: (field_identifier) @name))) @reference.call

(macro_invocation
    macro: (identifier) @name) @reference.call

; implementations (as containers so methods inside can be nested correctly)
;
; `impl_item.type` (the Self type) and `impl_item.trait` are both `_type`/type-ish
; fields whose grammar allows more than the plain `type_identifier` form: a generic
; impl target (`impl<T> Container<T>`) parses as `generic_type`, and a
; path-qualified trait (`impl std::fmt::Display for X`) parses as
; `scoped_type_identifier`. Both are common Rust idioms and must be covered
; alongside the plain form, or generic/qualified impls silently lose their
; container (methods inside stop being nested under the impl).

; Plain: impl Foo { ... }
(impl_item
    type: (type_identifier) @name) @definition.module

; Generic: impl<T> Foo<T> { ... }
(impl_item
    type: (generic_type
        type: (type_identifier) @name)) @definition.module

; Path-qualified: impl foo::Bar { ... } (rare but grammar-legal)
(impl_item
    type: (scoped_type_identifier
        name: (type_identifier) @name)) @definition.module

; Plain trait impl: impl Trait for Foo
(impl_item
    trait: (type_identifier) @name) @reference.implementation

; Generic trait impl: impl From<u32> for Foo, impl PartialEq<Foo> for Foo
(impl_item
    trait: (generic_type
        type: (type_identifier) @name)) @reference.implementation

(impl_item
    trait: (generic_type
        type: (scoped_type_identifier
            name: (type_identifier) @name))) @reference.implementation

; Path-qualified trait impl: impl std::fmt::Display for Foo
(impl_item
    trait: (scoped_type_identifier
        name: (type_identifier) @name)) @reference.implementation

; Inherent impl (no trait): impl Foo { ... }
(impl_item
    type: (type_identifier) @name
    !trait) @reference.implementation

; Inherent generic impl (no trait): impl<T> Foo<T> { ... }
(impl_item
    type: (generic_type
        type: (type_identifier) @name)
    !trait) @reference.implementation
