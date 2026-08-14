#version 450 core

// ---------------------------------------------------------------------------
// tags: struct_specifier variants
// ---------------------------------------------------------------------------

// Named struct with body — matches (struct_specifier name: (type_identifier) body: (_))
struct NamedStruct {
    vec3 field;
};

// NEGATIVE: forward declaration, no body — struct_specifier.body is optional
// and absent here, so this must NOT produce a @definition.class match.
struct ForwardDeclared;

// NEGATIVE: anonymous struct, no name — struct_specifier.name is optional
// and absent here, so this must NOT produce a @name match.
struct {
    vec3 x;
} anonInstance;

// ---------------------------------------------------------------------------
// tags: interface block variants (declaration node, unfielded — see
// glsl.tags.scm comment). One per storage-qualifier keyword.
// ---------------------------------------------------------------------------

uniform UniformBlock {
    vec4 a;
} uniformInstance;

buffer BufferBlock {
    float data[];
} bufferInstance;

in InBlock {
    vec3 normal;
} inInstance;

out OutBlock {
    vec3 normal;
} outInstance;

// NEGATIVE: plain (non-block) uniform declaration — no field_declaration_list
// child, so this must NOT produce an interface-block @definition.class match.
uniform vec3 u_PlainUniform;

// NEGATIVE: plain uniform of a custom struct type — still no
// field_declaration_list child of the declaration itself.
uniform NamedStruct u_PlainStructUniform;

// ---------------------------------------------------------------------------
// calls: call_expression.function variants
// ---------------------------------------------------------------------------

vec3 callVariants(vec3 v) {
    // Simple call: function: (identifier)
    vec3 a = normalize(v);

    // Field/member call: function: (field_expression argument: (_) field: (field_identifier))
    // (Array .length() is GLSL's only field-call-like builtin; exercised on
    // the SSBO in sample.glsl. Structs have no methods, so field_expression
    // as a call target otherwise only arises from constructs the grammar
    // doesn't model — nothing further to add here.)
    float n = dot(a, a);

    return a * n;
}

// ---------------------------------------------------------------------------
// complexity / cfg: control-flow node variants (case_statement with and
// without a `value` field — plain `case N:` vs `default:`)
// ---------------------------------------------------------------------------

int switchVariants(int mode) {
    switch (mode) {
        case 0:
            return 1;
        default:
            return 0;
    }
}

// ---------------------------------------------------------------------------
// cfg: discard quirk — `discard;` has no dedicated grammar node. It parses
// as an expression_statement wrapping a bare identifier "discard", matched
// in glsl.cfg.scm via #eq?. NEGATIVE case: a user identifier that merely
// looks similar must not match.
// ---------------------------------------------------------------------------

void discardVariants(float alpha) {
    if (alpha < 0.01) {
        discard;
    }
}

void discardNegative() {
    // NEGATIVE: a bare identifier expression statement that is NOT "discard"
    // must not match the #eq? predicate.
    int discardCount;
    discardCount;
}

// ---------------------------------------------------------------------------
// imports: preproc_include.path variants
// ---------------------------------------------------------------------------

#include "quoted_variant.glsl"
#include <angle_bracket_variant.glsl>
