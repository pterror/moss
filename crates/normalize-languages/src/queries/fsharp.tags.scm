; F# tags query
; Covers: functions/values, members, types, modules

; Function and value definitions
(function_or_value_defn
  (function_declaration_left
    . (identifier) @name)) @definition.function

; KNOWN LIMITATION, not fixed here: active-pattern definitions
; (`let (|Even|Odd|) n = ...`) have `active_pattern` — not `identifier` —
; as the first child of `function_declaration_left` (confirmed via
; `normalize syntax ast`/node-types.json: `function_declaration_left`'s
; declared child types are `access_modifier|active_pattern|
; argument_patterns|identifier|op_identifier|type_arguments`). An active
; pattern can name up to 7 cases in one form (`(|Case1|Case2|...|)`), each
; nested as an `active_pattern_op_name` token inside `active_pattern` —
; there is no single "the name" the way every other definition form here
; has, so this is left uncaptured rather than fabricating a single-name
; extraction that doesn't match the construct's actual multi-name shape.

; Member definitions (methods and properties)
;
; `method_or_prop_defn`'s `name` field is a `property_or_ident` node, NOT
; a direct `identifier` child — confirmed via node-types.json and
; `normalize syntax ast`. For an instance member (`member this.Add(...)`),
; `property_or_ident` has TWO `identifier` children (`this`, then the real
; member name); for a static member (`static member Create()`) it has
; ONE. The previous pattern searched for `(identifier)` as a direct child
; of `method_or_prop_defn` (skipping the intermediate `property_or_ident`
; entirely), which never matches at that depth — confirmed via `normalize
; syntax query`: zero @definition.method captures on any member in a
; sample class with instance methods, an instance property, and a static
; factory method. Anchoring on the LAST `identifier` child of
; `property_or_ident` (via the trailing `.`) correctly picks the member
; name in both the `this.Name` and bare-name cases.
(member_defn
  (method_or_prop_defn
    (property_or_ident
      (identifier) @name .))) @definition.method

; Module definitions (named_module wraps the entire file-level module)
;
; `long_identifier` captures the FULL dotted path for a nested module
; declaration (`module Outer.Inner.Test`) as one node/one @name — not one
; capture per path component. The previous pattern captured the inner
; `(identifier)` without anchoring, which matched every component
; separately (`Outer`, `Inner`, `Test` as three separate
; @definition.module names for the same module node) — confirmed via
; `normalize syntax query`. Mirrors the convention already used by
; `fsharp.imports.scm`'s `(import_decl (long_identifier) @import.path)`,
; which likewise captures the whole dotted path as one unit.
(named_module
  (long_identifier) @name) @definition.module

; Type definitions (records, unions, classes, aliases)
; Use wildcard _ to match record_type_defn, union_type_defn, etc.
;
; `type_name`'s `type_name` field allows both `identifier` (the common
; case) and `long_identifier` (a dotted type name, e.g. `type Foo.Bar =
; ...` — legal per node-types.json and confirmed parseable via `normalize
; syntax ast`). The previous pattern only handled the plain `identifier`
; case; dotted type names produced zero @definition.class matches.
(type_definition
  (_
    (type_name
      (identifier) @name))) @definition.class

(type_definition
  (_
    (type_name
      (long_identifier) @name))) @definition.class
