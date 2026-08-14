// ---------------------------------------------------------------------------
// tags: struct_specifier / cbuffer_specifier / class_specifier variants
// ---------------------------------------------------------------------------

// Named struct with body
struct NamedStruct {
    float3 field;
};

// NEGATIVE: forward declaration, no body — struct_specifier.body is
// optional and absent here, so this must NOT produce a @definition.class
// match.
struct ForwardDeclared;

// cbuffer with body, no register binding — the grammar mis-parses this as
// a function_definition (see hlsl.tags.scm), which is exactly what the
// workaround pattern targets.
cbuffer NamedCbuffer {
    float4 value;
};

// NEGATIVE / KNOWN GAP: cbuffer with a `: register(...)` binding — the
// overwhelmingly common real-world shape. The grammar's GLR resolution
// disconnects the `{ ... }` body from the declaration entirely (it becomes
// an unrelated sibling `compound_statement`), so this cannot produce a
// @definition.class match. Documented in hlsl.tags.scm as an unfixable
// grammar limitation.
cbuffer RegisterBoundCbuffer : register(b3) {
    float4 boundValue;
};

// class with a body and a method — method uses field_identifier declarator.
class NamedClass {
    float3 state;

    float3 GetState() {
        return state;
    }
};

// NEGATIVE: `interface Foo { ... }` has no dedicated grammar node in this
// cpp-derived HLSL grammar — it mis-parses as a function_definition whose
// return type is the bare identifier "interface" and whose declarator is a
// parameterless identifier "IFoo", indistinguishable from a malformed
// function. Documented in hlsl.tags.scm; left uncaptured (grammar
// limitation, not a query bug). This construct is included here only to
// confirm it does NOT falsely produce a @definition.class match.
interface IFoo {
    float3 Eval();
};

// ---------------------------------------------------------------------------
// tags: function_declarator.declarator variants (identifier vs
// field_identifier)
// ---------------------------------------------------------------------------

// Free function: declarator: (identifier)
float3 FreeFunction(float3 v) {
    return v;
}

// (NamedClass.GetState above exercises declarator: (field_identifier).)

// ---------------------------------------------------------------------------
// calls: call_expression.function / field_expression.field variants
// ---------------------------------------------------------------------------

Texture2D gTex : register(t0);
ByteAddressBuffer gBuf : register(t1);

float4 callVariants(SamplerState samp, float2 uv) {
    // Simple call: function: (identifier)
    float3 n = normalize(float3(uv, 0));

    // Constructor-style call also parses as function: (identifier)
    float3 c = float3(1, 0, 0);

    // Member call: function: (field_expression field: (field_identifier))
    float4 sampled = gTex.Sample(samp, uv);

    // Templated member call: function: (field_expression field:
    // (template_method name: (field_identifier)))
    float4 loaded = gBuf.Load<float4>(0);

    return sampled + loaded + float4(n + c, 0);
}

template<typename T>
T TemplatedFree(T v) {
    return v;
}

float3 templateCallSite(float3 v) {
    // Templated free function call: function: (template_function name: (identifier))
    return TemplatedFree<float3>(v);
}

// ---------------------------------------------------------------------------
// cfg: discard_statement (HLSL has a dedicated node, unlike GLSL)
// ---------------------------------------------------------------------------

float4 discardVariant(float alpha) {
    if (alpha < 0.01) {
        discard;
    }
    return float4(alpha, alpha, alpha, alpha);
}

// ---------------------------------------------------------------------------
// complexity / cfg: switch case_statement with and without a `value` field
// ---------------------------------------------------------------------------

int switchVariants(int mode) {
    switch (mode) {
        case 0:
            return 1;
        default:
            return 0;
    }
}

// do-while: previously missing entirely from hlsl.complexity.scm's
// @complexity/@nesting sets (do_statement was never listed).
int doWhileVariant(int limit) {
    int total = 0;
    int i = 0;
    do {
        total += i;
        i++;
    } while (i < limit);
    return total;
}

// ---------------------------------------------------------------------------
// imports: preproc_include.path variants
// ---------------------------------------------------------------------------

#include "quoted_variant.hlsli"
#include <angle_bracket_variant.hlsli>
