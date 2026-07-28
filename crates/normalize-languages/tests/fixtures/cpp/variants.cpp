// Completeness fixture: one construct per grammar-legal variant of each
// field the cpp.{tags,calls,imports,types}.scm queries constrain, cross-
// referenced against arborium-cpp's node-types.json. Every construct here is
// *expected to be captured*; see query_fixtures.rs `cpp_*_completeness_*`
// tests for the matrix.
//
// This file also carries deliberate near-miss constructs (marked NEGATIVE)
// that must NOT be captured, to guard against over-broad patterns.

#include <cstddef>

// --- imports: using declarations/directives/aliases -------------------------

using namespace detail; // using_declaration + "namespace" literal + bare
                        // identifier -> import.path = "detail"
namespace ns_target {
struct Thing {};
} // namespace ns_target
using ns_target::Thing; // using_declaration wrapping a qualified_identifier
                        // -> import.path = "ns_target::Thing"
using IntAlias = int;   // alias_declaration -> import.alias/import.path
namespace short_ns = ns_target; // namespace_alias_definition, single-segment
namespace nested_alias = ns_target::Thing::deeper; // documents the nested
                                                    // path shape is accepted
                                                    // (deliberately unused
                                                    // beyond parse-checking
                                                    // the query pattern)

// --- struct/class/union name+body variants ----------------------------------

struct PlainStruct {
    int x;
}; // struct_specifier: name = type_identifier -> @definition.class

union PlainUnion {
    int i;
    float f;
}; // union_specifier: bare, name = type_identifier, body present ->
   // @definition.class. Same asymmetry bug as C had: the old query only
   // matched `declaration type: (union_specifier ...)`.

typedef union TaggedUnion {
    int i;
    float f;
} TaggedUnionAlias; // both "TaggedUnion" and "TaggedUnionAlias" captured.

template <typename T>
class TemplateClass {
public:
    T value;
}; // class_specifier: name = type_identifier (template parameter list is a
   // sibling of the class_specifier, inside template_declaration; the class
   // name itself is unaffected)

template <>
class TemplateClass<int> {
public:
    int value;
}; // Explicit specialization: name = template_type(name: type_identifier)
   // -> previously unmatched entirely (plain type_identifier pattern only).

// --- namespaces --------------------------------------------------------------

namespace outer_ns {
// namespace_definition: name = namespace_identifier -> @definition.module
struct InOuter {};

namespace inner_ns {
// nested (non path-form) namespace_definition
struct InInner {};
} // namespace inner_ns
} // namespace outer_ns

namespace deep::path::here {
// namespace_definition: name = nested_namespace_specifier ("deep::path::here")
// -> @definition.module. Previously entirely unmatched — namespaces had zero
// tags coverage at all, so everything inside one lost its container.
struct InDeepPath {};
} // namespace deep::path::here

// --- function_declarator.declarator variants: destructors/operators --------

class WithSpecialMembers {
public:
    WithSpecialMembers() {} // constructor: declarator = identifier (class
                             // name), same shape as any method — already
                             // handled by the plain identifier pattern.
    ~WithSpecialMembers() {} // inline destructor: declarator = destructor_name
                             // -> previously unmatched entirely.
    WithSpecialMembers &operator=(const WithSpecialMembers &other) {
        // inline operator overload: declarator = operator_name -> previously
        // unmatched entirely.
        (void)other;
        return *this;
    }
    void plain_method() {} // declarator = field_identifier (already handled)
    static void stat() {}  // static method: declarator = identifier, same
                           // shape as a free function (already handled)
};

// Out-of-line member definitions live on a distinct class from
// WithSpecialMembers (whose destructor/operator/method are already defined
// inline above) to avoid redefining the same members twice in one
// translation unit.
class OutOfLineMembers {
public:
    OutOfLineMembers();
    ~OutOfLineMembers();
    OutOfLineMembers &operator+=(int n);
    void method();
};

OutOfLineMembers::OutOfLineMembers() {}
// out-of-line constructor: qualified_identifier scope: namespace_identifier
// ("OutOfLineMembers" resolves as namespace_identifier in this grammar
// without full semantic context), name: identifier -- already handled.

OutOfLineMembers::~OutOfLineMembers() {}
// out-of-line destructor: qualified_identifier name: destructor_name ->
// previously unmatched entirely (every user-defined destructor defined
// out-of-line, a routine pattern in non-header-only C++, was invisible).

OutOfLineMembers &OutOfLineMembers::operator+=(int n) {
    // out-of-line operator overload: qualified_identifier name: operator_name
    // -> previously unmatched entirely.
    (void)n;
    return *this;
}

void OutOfLineMembers::method() {}
// out-of-line plain method: already handled (name: identifier).

template <typename T>
class OutOfLineTemplateMethods {
public:
    T get();
};

template <typename T>
T OutOfLineTemplateMethods<T>::get() {
    // out-of-line method of a template class: qualified_identifier scope is
    // template_type ("OutOfLineTemplateMethods<T>"), not namespace_identifier
    // -> previously unmatched entirely (same root cause as Rust's generic-impl
    // gap: the qualifier itself carries template arguments).
    return T{};
}

// --- preproc_def / preproc_function_def (shared C preprocessor syntax) -----

#define VARIANTS_MAX 256
#define VARIANTS_SQUARE(x) ((x) * (x))

// --- calls: field_expression.field variants ---------------------------------

struct CallTarget {
    ~CallTarget() {}
    void plain_method() {}
    template <typename T>
    T templated_method(T x) {
        return x;
    }
};

struct DerivedCallTarget : CallTarget {
    void plain_method() {} // shadows base
};

void field_call_variants() {
    CallTarget obj;
    CallTarget *ptr = &obj;
    obj.plain_method();          // field: field_identifier (already handled)
    ptr->plain_method();         // field: field_identifier via "->" (already
                                  // handled)
    obj.templated_method<int>(1); // field: template_method -> previously
                                   // unmatched entirely.
    obj.~CallTarget();            // field: destructor_name -> previously
                                   // unmatched entirely (explicit destructor
                                   // call, e.g. in placement-new patterns).
    DerivedCallTarget derived;
    derived.CallTarget::plain_method();
    // field: qualified_identifier ("CallTarget::plain_method") -> explicit
    // base-class-qualified call, disambiguating which override to invoke;
    // previously unmatched entirely.
}

// --- calls: qualified_identifier.name and template_function variants -------

template <typename T>
T identity(T x) {
    return x;
}

namespace call_ns {
template <typename T>
T helper(T x) {
    return x;
}
} // namespace call_ns

void qualified_call_variants() {
    WithSpecialMembers::stat(); // call_expression.function = qualified_identifier,
                                // name: identifier -> static method call
                                // (already handled by the pre-existing
                                // qualified-call pattern; included for
                                // documentation/regression coverage).
}

void scoped_template_call_variants() {
    int a = identity<int>(5);
    // call_expression.function = template_function directly (no qualifier
    // at all) -> previously unmatched entirely; direct analogue of Rust's
    // turbofish gap.
    int b = call_ns::helper<int>(3);
    // call_expression.function = qualified_identifier, name: template_function
    // -> previously unmatched entirely.
    (void)a;
    (void)b;
}

// --- NEGATIVE: constructs that must NOT match -------------------------------

struct NegativeHolder {
    int field;
};

int negative_field_access(NegativeHolder *holder) {
    // A bare field read must never be reported as a @call.
    return holder->field;
}

void negative_lambda_is_not_a_tag() {
    // A lambda is not a function_declarator/class_specifier/etc.; its
    // parameter/body identifiers must never appear as @definition.function
    // or @definition.method.
    auto add_one = [](int x) { return x + 1; };
    (void)add_one(1);
}
