// Completeness fixture: one construct per grammar-legal variant of each
// field the zig.{tags,calls,imports,complexity,types,decorations}.scm
// queries constrain, cross-referenced against arborium-zig 2.17.0's
// node-types.json and real parse output (`normalize syntax ast` /
// `normalize syntax query`). Every construct here is *expected to be
// captured* by the relevant query; see query_fixtures.rs `zig_completeness_*`
// tests for the matrix. A NEGATIVE section at the bottom holds deliberate
// near-miss constructs that must NOT be captured, guarding against
// over-broad patterns (the root cause of every bug this pass found).

//! Container-level doc comment: decorations.scm container_doc_comment variant.

const std = @import("std");

// --- calls.scm: SuffixExpr call-site variants -------------------------------

fn plainCall() void {
    identity(); // variable_type_function: (IDENTIFIER) directly followed by FnCallArguments
}

fn dottedCall() void {
    const obj = Wrapper{};
    obj.method(); // FieldOrFnCall function_call: (IDENTIFIER) — the previously-broken shape
}

fn chainedDottedCall() void {
    const obj = Wrapper{};
    obj.inner().method(); // two FieldOrFnCall links, each its own @call
}

fn builtinCall() void {
    _ = @TypeOf(1); // BUILTINIDENTIFIER directly followed by FnCallArguments
}

const Wrapper = struct {
    pub fn method(self: @This()) void {
        _ = self;
    }
    pub fn inner(self: @This()) @This() {
        return self;
    }
};

// --- imports.scm: @import(...) call-site variants ---------------------------

const direct_import = @import("direct.zig"); // VarDecl -> @import(...)

const NestedImports = struct {
    const nested_import = @import("nested.zig"); // @import nested inside a container
};

fn importInFunction() void {
    const local_import = @import("local.zig"); // @import inside a function body
    _ = local_import;
}

// --- types.scm: type-position variants --------------------------------------

// ParamType: plain identifier
fn paramPlainType(p: Point) void {
    _ = p;
}

// ParamType: qualified/dotted identifier (FieldOrFnCall field_access leaf)
fn paramQualifiedType(a: std.mem.Allocator) void {
    _ = a;
}

// ParamType: generic instantiation (FieldOrFnCall function_call leaf) —
// Zig generics are ordinary calls, so this is structurally identical to
// paramQualifiedType except the leaf carries FnCallArguments.
fn paramGenericType(l: std.ArrayList(u8)) void {
    _ = l;
}

// ParamType wrapped in a PrefixTypeOp (slice) — the type identifier is
// still reachable; the slice prefix must not block the match.
fn paramSliceType(items: []const Point) void {
    _ = items;
}

// FnProto return type: plain identifier
fn returnsPlainType() Point {
    return Point{ .x = 0, .y = 0 };
}

// FnProto return type: qualified/dotted identifier
fn returnsQualifiedType() std.mem.Allocator {
    unreachable;
}

// VarDecl type annotation with an initializer (anchored before "=")
const typed_var_with_init: Point = Point{ .x = 1, .y = 1 };

// VarDecl type annotation with a qualified type and an initializer
const typed_qualified_var: std.mem.Allocator = undefined;

// VarDecl type annotation with NO initializer (extern decl, anchored as
// last child instead of before "=")
extern var typed_var_no_init: Point;

// ContainerField type with a default value (anchored before "=")
const HasDefaultField = struct {
    field_with_default: Point = Point{ .x = 0, .y = 0 },
};

// ContainerField type with NO default value (anchored as last child)
const NoDefaultField = struct {
    field_no_default: Point,
};

// ContainerField qualified type, no default
const QualifiedField = struct {
    alloc: std.mem.Allocator,
};

const Point = struct {
    x: f64,
    y: f64,
};

// --- tags.scm: definition variants ------------------------------------------

pub fn taggedFunction() void {} // FnProto function: (IDENTIFIER) -> @definition.function

const TaggedStruct = struct { x: i32 }; // VarDecl -> ContainerDecl (struct) -> @definition.class
const TaggedEnum = enum { a, b }; // VarDecl -> ContainerDecl (enum) -> @definition.class
const TaggedUnion = union(enum) { a: i32, b: i32 }; // VarDecl -> ContainerDecl (union) -> @definition.class

// --- complexity.scm: complexity/nesting variants ----------------------------

fn tryComplexity() !void {
    const x = try mayFail(); // UnaryExpr operator: (PrefixOp) "try" -> @complexity
    _ = x;
}

fn mayFail() !i32 {
    return 1;
}

fn catchComplexity() i32 {
    return mayFail() catch 0; // BinaryExpr operator: (BitwiseOp) "catch" -> @complexity via generic BinaryExpr
}

// --- decorations.scm: doc_comment variant -----------------------------------

/// Function-level doc comment: decorations.scm doc_comment variant.
pub fn documented() void {}

// =============================================================================
// NEGATIVE: constructs that must NOT be captured by the relevant query.
// =============================================================================

fn negativeNonCallIdentifier() void {
    const plain_value = 42; // bare VarDecl init with no ":" type annotation —
    // must NOT be captured by types.scm (no type slot exists here at all;
    // this is exactly the shape the old blanket `(IDENTIFIER) @type.reference`
    // wrongly matched).
    _ = plain_value;
}

fn negativeFieldAccessNoCall() void {
    const w = Wrapper{};
    _ = w.method; // FieldOrFnCall field_access (no FnCallArguments sibling) —
    // a bare field/method reference with no call parens must NOT be
    // captured by calls.scm's @call patterns (no FnCallArguments present).
}

fn negativeIndexingIsNotACall(items: []const i32) i32 {
    return items[0]; // SuffixOp (array indexing) — must NOT be captured by
    // calls.scm's method-call pattern; SuffixOp is unrelated to
    // FieldOrFnCall and was the prior (broken) pattern's incorrect anchor.
}

fn negativeVarDeclInitializerIsNotAType() void {
    // The initializer ErrorUnionExpr must NOT be captured as a type
    // reference even though it has the same SuffixExpr/variable_type_function
    // shape as a real type slot — it comes after "=", not after ":".
    const untyped_alias = Point;
    _ = untyped_alias;
}

fn negativeBuiltinPrimitiveTypeIsNotIdentifier(n: i32) i32 {
    // i32 is a BuildinTypeExpr keyword, not an IDENTIFIER — never captured
    // by types.scm regardless of position.
    return n;
}
