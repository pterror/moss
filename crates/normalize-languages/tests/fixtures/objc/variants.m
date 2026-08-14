// Completeness matrix for objc query files. One small, commented construct
// per node-type variant found via node-types.json + `normalize syntax
// query`/`normalize syntax ast --lang objc`. See docs/query-testing-methodology.md.

// ---------------------------------------------------------------------------
// tags.scm — classes/categories/protocols (class_interface.name is a
// positional identifier child, NOT a field — anchored `.` selects it and
// excludes the `superclass`/`category` fields, which are the SAME node type)
// ---------------------------------------------------------------------------

// class_interface, no superclass, plain name — definition.class "PlainClass"
@interface PlainClass
@end

// class_interface with superclass field (also type identifier — the anchor
// bug this fixture guards against) — definition.class "SubClass"
@interface SubClass : NSObject
@end

// class_interface with category field (also type identifier) — definition.class
// "CategorizedClass"; the category name "CategoryName" must NOT appear as @name.
@interface CategorizedClass (CategoryName)
@end

// class_interface with protocol conformance list (protocol_reference_list ->
// identifier nested one level deeper than class_interface's direct children,
// so it was never ambiguous with the class name) — definition.class "Conformer"
@protocol ProtoA
@end
@protocol ProtoB
@end
@interface Conformer : NSObject <ProtoA, ProtoB>
@end

// class_implementation, no superclass — definition.class "ImplOnly"
@implementation ImplOnly
@end

// class_implementation with superclass field — definition.class "ImplSub"
@implementation ImplSub : NSObject
@end

// protocol_declaration, no conformance — definition.interface "SimpleProto"
@protocol SimpleProto
- (void)doIt;
@end

// protocol_declaration with conformance list + @optional section (wraps
// following method_declarations in qualified_protocol_interface_declaration,
// one level deeper — verified the method still needs no special handling
// here since tags.scm doesn't target protocol methods differently)
// — definition.interface "ExtendedProto"
@protocol ExtendedProto <NSObject>
- (double)area;
@optional
- (void)describe;
@end

// ---------------------------------------------------------------------------
// tags.scm — functions (function_declarator matches both definitions AND
// bare prototypes; the old objc.tags.scm only matched through
// function_definition, missing every prototype)
// ---------------------------------------------------------------------------

// Bare prototype, no body — definition.function "protoFunc"
double protoFunc(double x);

// Full definition with body — definition.function "definedFunc"
double definedFunc(double x) {
    return x * 2.0;
}

// ---------------------------------------------------------------------------
// tags.scm — macros (preproc_def / preproc_function_def, ported from
// c.tags.scm; previously 0 coverage in objc.tags.scm)
// ---------------------------------------------------------------------------

// Object-like macro — definition.macro "MAX_ITEMS"
#define MAX_ITEMS 64

// Function-like macro — definition.macro "SQUARE_OF"
#define SQUARE_OF(x) ((x) * (x))

// Header guard: valueless #define matching the enclosing #ifndef's name —
// must be suppressed (NOT a definition.macro).
#ifndef VARIANTS_H
#define VARIANTS_H
#endif

// ---------------------------------------------------------------------------
// tags.scm — union / typedef'd function pointer / anonymous struct typedef
// ---------------------------------------------------------------------------

// union_specifier — definition.class "PayloadUnion"
union PayloadUnion {
    int i;
    double d;
};

// Typedef'd function pointer (alias nests 3 levels: function_declarator >
// parenthesized_declarator > pointer_declarator > type_identifier) —
// definition.type "Comparator"
typedef int (*Comparator)(int, int);

// Anonymous struct typedef — definition.type "Vec2" (already-working case,
// included here so the completeness matrix documents it explicitly)
typedef struct {
    double x;
    double y;
} Vec2;

// ---------------------------------------------------------------------------
// tags.scm — methods (method_declaration/method_definition have NO field
// names at all; anchored on the fixed "-"/"+" method_type prefix to select
// only the selector HEAD, excluding later selector segments/parameter names)
// ---------------------------------------------------------------------------

@interface MethodVariants : NSObject
// No-arg instance method — definition.method "noArgMethod"
- (void)noArgMethod;
// No-arg class method — definition.method "classMethod"
+ (instancetype)classMethod;
// Multi-keyword selector: only the HEAD segment "keywordMethod" is
// captured — "withArg" (the second segment) and "arg"/"other" (parameter
// names) must NOT appear as @name (see negative section).
- (void)keywordMethod:(int)arg withArg:(int)other;
@end

@implementation MethodVariants
- (void)noArgMethod {
}
+ (instancetype)classMethod {
    return [self new];
}
- (void)keywordMethod:(int)arg withArg:(int)other {
}
@end

// ---------------------------------------------------------------------------
// calls.scm — call_expression and message_expression variants
// ---------------------------------------------------------------------------

int callVariants(int seed) {
    // C-style call: function field = identifier — @call "plainCall"
    int a = plainCall(seed);

    // C-style call through a struct pointer field — @call.qualifier
    // "structPtr", @call "fieldCall"
    struct { int (*fieldCall)(int); } *structPtr = 0;
    int b = structPtr->fieldCall(seed);

    // Single-keyword message send — @call.qualifier "MethodVariants",
    // @call "classMethod"
    id obj = [MethodVariants classMethod];

    // Multi-keyword message send — the grammar tags EVERY keyword segment
    // with the SAME `method` field (verified via probe), so this construct
    // is the regression guard for the calls.scm anchor fix: exactly ONE
    // @call ("keywordMethod") may appear for this send, never a second
    // spurious @call for "withArg" (see NEGATIVE section below).
    [obj keywordMethod:a withArg:b];

    return a + b;
}

// ---------------------------------------------------------------------------
// imports.scm — preproc_include path variants + module_import
// ---------------------------------------------------------------------------

// system_lib_string path — import.path "<Foundation/Foundation.h>" (Rust
// side strips the angle brackets; captured node kind is system_lib_string)
#import <Foundation/Foundation.h>

// string_literal path (quoted local header) — import.path "Local.h"
#import "Local.h"

// #include with string_literal path — import.path "cheader.h"
#include "cheader.h"

// module_import, single segment — import.path "UIKit"
@import UIKit;

// module_import, dotted submodule — import.path "Contacts" (only the
// leading/framework segment is captured; ".ContactsUI" is dropped, an
// honest partial fix documented in objc.imports.scm)
@import Contacts.ContactsUI;

// ---------------------------------------------------------------------------
// complexity.scm — branch/loop/exception variants
// ---------------------------------------------------------------------------

int complexityVariants(int n) {
    // if/else — @complexity, @nesting
    if (n > 0) {
        n += 1;
    } else {
        n -= 1;
    }

    // switch + case_statement arms — @complexity (switch AND each case)
    switch (n) {
        case 1:
            n = 1;
            break;
        case 2:
            n = 2;
            break;
        default:
            n = 0;
            break;
    }

    // while — @complexity, @nesting
    while (n > 0) {
        n -= 1;
    }

    // for — @complexity, @nesting
    for (int i = 0; i < 10; i++) {
        n += i;
    }

    // do-while — @complexity, @nesting
    do {
        n -= 1;
    } while (n > 0);

    // short-circuit boolean operators — @complexity
    int flag = (n > 0 && n < 100) || n == -1;

    // ternary — @complexity
    int sign = n >= 0 ? 1 : -1;

    // @try/@catch — @complexity, @nesting
    @try {
        n = 100 / n;
    } @catch (NSException *e) {
        n = 0;
    } @finally {
        n = flag + sign;
    }

    return n;
}

// ---------------------------------------------------------------------------
// decorations.scm — #pragma / __attribute__ (previously uncaptured)
// ---------------------------------------------------------------------------

#pragma mark - Deprecated helpers

__attribute__((deprecated))
void oldHelper(void);

@interface AttributedInterface : NSObject
- (void)oldMethod __attribute__((deprecated));
@end

// ---------------------------------------------------------------------------
// NEGATIVE cases — constructs that must NOT produce the captures they might
// naively be mistaken for.
// ---------------------------------------------------------------------------

// NEGATIVE (tags): the superclass name in `SubClass : NSObject` above must
// never appear as a definition.class @name (see anchor fix in
// objc.tags.scm) — checked in query_fixtures.rs by asserting "NSObject"
// never pairs with definition.class.

// NEGATIVE (tags): the category name "CategoryName" in
// `CategorizedClass (CategoryName)` above must never appear as a
// definition.class @name — checked alongside the superclass case.

// NEGATIVE (tags): in `keywordMethod:(int)arg withArg:(int)other`, neither
// "withArg" (second selector segment) nor "arg"/"other" (parameter names)
// may appear as a definition.method @name — only "keywordMethod" may.

// NEGATIVE (imports): a plain C function call that happens to be named
// `import` must not be treated as an @import — imports.scm only matches
// `preproc_include`/`module_import` node types, so this is inherently safe,
// documented here rather than fabricating a matching construct.
int notAnImport(void) {
    return 0;
}
