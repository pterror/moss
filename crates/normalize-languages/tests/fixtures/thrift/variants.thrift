// Completeness matrix for thrift.{tags,imports,decorations,complexity}.scm.
// One small, clearly commented construct per node-type variant found by
// cross-referencing node-types.json (arborium-thrift 2.17.0), verified
// against real parse output via `normalize syntax ast` / `syntax query`.
// A NEGATIVE section at the bottom documents near-miss constructs that must
// NOT produce the same captures as their positive counterparts.

namespace py variants

// --- imports.scm: @import / @import.path -----------------------------
include "shared.thrift"
cpp_include "custom.h"

// NEGATIVE: a `package_declaration` is not an include/import -- it names
// the compilation unit's package for codegen and references no other
// file, so it must NOT be captured by imports.scm's @import pattern.
// (Header directives -- namespace/include/package -- must all precede any
// definition in this grammar, so this is placed here rather than at the
// end of the file.)
package "com.example.variants"

// --- tags.scm: struct_definition -> type: (identifier) @name ----------
struct PlainStruct {
  1: string field1,
}

// --- tags.scm: union_definition -> type: (identifier) @name -----------
union PlainUnion {
  1: string a,
  2: i32 b,
}

// --- tags.scm: exception_definition (positional identifier) @name -----
exception PlainException {
  1: string reason,
}

// exception_definition with exception_modifier keywords before "exception"
// (safe/transient/permanent/client/server) -- modifiers precede the
// keyword, not the name, so the name match is unaffected.
transient exception ModifiedException {
  1: string reason,
}

// --- tags.scm: enum_definition -> type: (identifier) @name ------------
// MUST use the `type` field: enum values (ACTIVE/INACTIVE below) are also
// direct `identifier` children of enum_definition, and a positional match
// on the first identifier child incorrectly matched enum name only by
// luck of ordering -- the bug was that ALL enum value identifiers matched
// too when the query lacked the field constraint. See thrift.tags.scm.
enum PlainEnum {
  FIRST_VALUE = 1,
  SECOND_VALUE = 2,
}

// --- tags.scm: senum_definition (legacy string enum) -------------------
// Deprecated but still parsed by the grammar; same `type`-field shape as
// enum_definition.
senum LegacyStringEnum {
  "OPTION_A",
  "OPTION_B",
}

// --- tags.scm: service_definition -> "service" . (identifier) @name ---
service BaseService {
  void ping(),
}

// service_definition with `extends`: the `type` field in node-types.json
// is `multiple: true` and covers BOTH the service's own name and the
// extends-clause identifier -- the anchor (`.` immediately after
// `"service"`) is required to capture only the service's own name.
// The extends target itself is captured separately as @reference.interface.
service DerivedService extends BaseService {
  void pong(),
}

// --- tags.scm: interaction_definition -> type: (identifier) @name -----
// Thrift's RPC "interaction" construct: a service-like container of
// function_definitions, entered via a service's `performs` statement.
interaction MyInteraction {
  void doThing(),
}

service ServiceWithInteraction {
  performs MyInteraction;
  void ping(),
}

// --- tags.scm: function_definition (positional identifier) @name ------
// oneway modifier + throws clause: neither interferes with the name match
// (return type is a `type` child, throws targets are nested in
// `throws` -> `parameters` -> `parameter`, not direct identifier children).
service FunctionVariants {
  oneway void fireAndForget(1: string msg),
  User getUser(1: UUID id) throws (1: NotFoundError notFound, 2: SystemError sysErr),
  list<User> listUsers(),
  map<string, i32> countsByName(),
  set<UUID> allIds(),
}

// --- tags.scm: typedef_definition -> (typedef_identifier) @name -------
typedef string SimpleAlias
typedef list<i32> ListAlias
typedef map<string, i32> MapAlias

// --- tags.scm: const_definition -> (identifier) @name ------------------
// Scalar, container, and identifier-valued (enum-reference) constants --
// the enum-reference identifier (`PlainEnum.FIRST_VALUE`) is nested inside
// a `literal` child, not a direct child, so it does not collide with the
// const's own name match.
const i32 SCALAR_CONST = 42
const list<string> LIST_CONST = ["a", "b"]
const map<string, i32> MAP_CONST = {"a": 1, "b": 2}
const PlainEnum ENUM_REF_CONST = PlainEnum.FIRST_VALUE

// --- decorations.scm: annotation_definition @decoration ----------------
// Trailing `(cpp.type = "...")`-style codegen annotation on a field.
struct AnnotatedStruct {
  1: string name (cpp.type = "std::string", presence = "required"),
} (cpp.type = "AnnotatedStructRecord")

// --- decorations.scm: fb_annotation_definition @decoration --------------
// Facebook fbthrift-style prefix annotation before a definition.
@fb.Foo
struct FbAnnotatedStruct {
  1: string name,
}

// --- complexity.scm: nesting nodes --------------------------------------
// service_definition, interaction_definition, and function_definition are
// each asserted as @nesting elsewhere in this file (BaseService,
// MyInteraction, FunctionVariants' methods).

// =========================================================================
// NEGATIVE cases -- constructs that must NOT match a tags.scm pattern
// they superficially resemble.
// =========================================================================

// NEGATIVE: enum value identifiers (FIRST_VALUE, SECOND_VALUE above) must
// NOT be captured by the enum_definition name pattern -- only the type-
// field identifier ("PlainEnum") should match. Verified: the field-based
// query yields exactly one @name per enum_definition, not one per value.

// NEGATIVE: the `extends` target identifier ("BaseService" in
// DerivedService above) must NOT be captured by the service's own
// @definition.interface `@name` pattern -- it is only reachable via the
// separate `"extends" . (identifier)` @reference.interface pattern.

// NEGATIVE: field names inside struct/union/exception bodies (e.g.
// "field1" in PlainStruct, "a"/"b" in PlainUnion, "reason" in
// PlainException) are nested inside `field` child nodes, not direct
// identifier children of the definition node, so they must NOT be
// captured as the struct/union/exception's own @name.
struct FieldNamesAreNotDefinitionNames {
  1: string thisFieldNameMustNotAppearAsADefinitionName,
}

// (The `namespace py variants` header line at the top of this file is
// itself the `namespace_declaration` negative case: it must NOT be
// captured by imports.scm's @import pattern.)
