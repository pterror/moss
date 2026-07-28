// Completeness fixture: one construct per grammar-legal variant of each
// field the c.{tags,calls,imports,types}.scm queries constrain, cross-
// referenced against arborium-c's node-types.json. Every construct here is
// *expected to be captured*; see query_fixtures.rs `c_*_completeness_*`
// tests for the matrix.
//
// This file also carries deliberate near-miss constructs (marked NEGATIVE)
// that must NOT be captured, to guard against over-broad patterns.

#include <stddef.h>

// --- struct_specifier / union_specifier name+body variants -----------------

struct PlainStruct {
    int x;
}; // struct_specifier: name = type_identifier, body present -> @definition.class

union PlainUnion {
    int i;
    float f;
}; // union_specifier: bare, name = type_identifier, body present ->
   // @definition.class. Previously entirely unmatched: the old query only
   // handled `declaration type: (union_specifier ...)`, which this bare
   // form never is.

typedef union {
    int i;
    float f;
} AnonUnion; // anonymous union_specifier (no name field) wrapped in a
             // typedef: the alias name "AnonUnion" is captured via the
             // type_definition pattern below; the union itself has no tag
             // name, correctly uncaptured (no name field to report).

typedef union TaggedUnion {
    int i;
    float f;
} TaggedUnionAlias; // named union_specifier nested inside a type_definition
                     // -> both "TaggedUnion" (union_specifier name+body) and
                     // "TaggedUnionAlias" (type_definition declarator) must
                     // be captured.

// --- function_declarator.declarator variants --------------------------------

int plain_function(int x) {
    return x; // declarator: identifier -> @definition.function
} // plain_function

int *pointer_returning_function(int x) {
    // Precedence: `*foo()` parses as "function returning pointer" —
    // pointer_declarator wraps function_declarator, so
    // function_declarator.declarator is still plain `identifier`.
    static int v;
    v = x;
    return &v;
}

int variadic_function(const char *fmt, ...) {
    // Variadic parameter does not change function_declarator.declarator —
    // still identifier. Included to document the construct is handled.
    (void)fmt;
    return 0;
}

int array_param_function(int arr[10]) {
    // `arr[10]` as a parameter is an array_declarator, but that's nested
    // inside parameter_list, not function_declarator.declarator itself
    // (still identifier "array_param_function").
    return arr[0];
}

// --- type_definition.declarator variants ------------------------------------

typedef struct {
    int *data;
} Handle; // plain typedef'd anonymous struct: declarator = type_identifier

typedef int (*FuncPtr)(int, int);
// Typedef'd function pointer: type_definition.declarator is
// function_declarator > parenthesized_declarator > pointer_declarator >
// type_identifier "FuncPtr" — three levels deeper than the plain case, and
// previously completely unmatched (verified via `normalize syntax query`
// against a probe file; this repo has no native C/C++ sources to measure
// density in, but callback-typedef idioms like this are ubiquitous in any
// C codebase with a callback/comparator API, e.g. POSIX signal handlers,
// qsort comparators, GLib/GTK signal callbacks).

// --- preproc_def / preproc_function_def (macro definitions) ----------------

#define MAX_SIZE 100
// preproc_def: name = identifier "MAX_SIZE" -> @definition.macro. Object-like
// macros had zero tags coverage before this fix.

#define SQUARE(x) ((x) * (x))
// preproc_function_def: name = identifier "SQUARE" -> @definition.macro.
// Function-like macros had zero tags coverage before this fix.

// --- enum_specifier ----------------------------------------------------------

enum Color { RED, GREEN, BLUE }; // name = type_identifier -> @definition.type

// --- nested/anonymous struct (documentation of existing correct behavior) --

struct Outer {
    struct {
        int x;
        int y;
    } point; // anonymous nested struct: no name field, correctly uncaptured
    union {
        int i;
        float f;
    }; // anonymous nested union member: same, correctly uncaptured
};

// --- multiple declarators in one declaration --------------------------------

int multi_a, multi_b, *multi_ptr;
// `declaration.declarator` is a multi-field; plain variable declarations
// aren't tags-worthy (consistent with every other language's convention of
// only tagging types/functions/macros, not bare variables), so none of
// these three produce a @name capture. Included to document the construct
// parses cleanly and doesn't trip up any pattern above.

// --- calls: field/pointer member call variants ------------------------------

void call_variants(void) {
    plain_function(1); // function: identifier -> @call
    FuncPtr fp = NULL;
    if (fp) {
        fp(1, 2); // call through a function-pointer variable: function:
                  // identifier "fp" -> @call (same shape as any plain call;
                  // no query change needed, included for documentation)
    }
}

struct WithFuncPtrField {
    FuncPtr fp;
};

int add_two(int a, int b) {
    return a + b;
}

void field_call_variants(struct WithFuncPtrField *p, struct WithFuncPtrField v) {
    p->fp(1, 2); // call_expression.function = field_expression, operator
                 // "->", field: field_identifier "fp" -> @call = "fp",
                 // @call.qualifier = "p"
    v.fp(3, 4);  // same shape, operator "."
    (void)add_two;
}

// --- NEGATIVE: constructs that must NOT match -------------------------------

int (*negative_function_pointer_variable)(int);
// `int (*fp)(int);` declares a variable of function-pointer type, not a
// function. function_declarator.declarator here is `parenthesized_declarator`
// (wrapping pointer_declarator -> identifier), never plain `identifier`, so
// this must NOT produce @definition.function. This is the correctly-excluded
// counterpart to the FuncPtr *typedef* case above (which legitimately should
// be captured, as a type alias, not a function).

union NegativeUsage;
void negative_union_usage_is_not_a_definition(void) {
    // A bodyless union-typed declaration references an existing tag; it must
    // never be reported as @definition.class (that would be true only when
    // the union_specifier also carries a body). This is exactly the false
    // positive the old `declaration type: (union_specifier ...)` pattern
    // produced (mislabeling a usage as @definition.class) and is now fixed
    // by requiring `body: (_)`.
}

struct NegativeHolder {
    int field;
};

int negative_field_access(struct NegativeHolder *holder) {
    // A bare field read through `->` with no call parens must never be
    // reported as a @call (regression guard against an over-eager
    // field_expression pattern matching plain member access).
    return holder->field;
}
