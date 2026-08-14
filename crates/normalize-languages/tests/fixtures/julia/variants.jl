# Completeness-matrix fixture for julia.*.scm query files.
#
# Each construct is commented with the node-type/field variant it exercises,
# cross-referenced against arborium-julia 2.17.0's node-types.json and
# verified via `normalize syntax ast` / `normalize syntax query`
# (docs/query-testing-methodology.md). A NEGATIVE section at the end holds
# near-miss constructs that must NOT match specific captures.

# --- imports: import_statement / using_statement shapes --------------------

import Statistics                       # import_statement . (identifier)
import Base.Iterators                   # import_statement . (import_path)
import JSON as J                        # import_statement (import_alias . identifier identifier)
import Base.Iterators as BI             # import_statement (import_alias . import_path identifier)
import Statistics: mean, std            # import_statement (selected_import . identifier identifier+)
import Base.Iterators: take, drop       # import_statement (selected_import . import_path identifier+)
using LinearAlgebra                     # using_statement . (identifier)
using Base.Iterators                    # using_statement . (import_path)
using Random as Rnd                     # using_statement (import_alias . identifier identifier)
using LinearAlgebra: norm, dot          # using_statement (selected_import . identifier identifier+)
using CSV: read as csv_read             # using_statement (selected_import . identifier (import_alias . identifier identifier))

# --- calls: call_expression / broadcast_call_expression callee shapes ------

simple_call(1)                          # call_expression (identifier)
Base.Iterators.take([1, 2], 1)          # call_expression (field_expression (_) (identifier))
Vector{Int}(undef, 3)                   # call_expression (parametrized_type_expression (identifier))
Dict(1 => "a")[1]("x")                  # call_expression (index_expression) -- call on an indexed result
(x -> x * 2)(10)                        # call_expression (parenthesized_expression) -- IIFE
sqrt.([1.0, 4.0])                       # broadcast_call_expression (identifier)
Base.sqrt.([1.0])                       # broadcast_call_expression (field_expression (_) (identifier))
Vector{Int}.([1, 2])                    # broadcast_call_expression (parametrized_type_expression (identifier))

# --- cfg: if_statement alternative shapes -----------------------------------

function branch_else(n)
    if n < 0                            # if_statement alternative: (else_clause), single match
        1
    else
        2
    end
end

function branch_elseif_only(n)
    if n < 0                            # if_statement alternative: (elseif_clause) first,
        1                               # no trailing else_clause -- previously unmatched outer branch
    elseif n == 0
        2
    end
end

function branch_bare(n)
    if n < 0                            # if_statement with no alternative field at all
        1
    end
end

# --- complexity: short-circuit / comprehension / generator / catch ---------

function complexity_probe(a, b, xs)
    r1 = a && b                         # binary_expression operator "&&"
    r2 = a || b                         # binary_expression operator "||"
    r3 = [x^2 for x in xs if x > 0]     # comprehension_expression
    r4 = sum(x for x in xs)             # generator (bare parenthesized form)
    try
        risky()
    catch e                             # catch_clause
        r1
    end
    return (r1, r2, r3, r4)
end

# --- tags: definitions -------------------------------------------------------

module Inner end                       # module_definition name: (identifier)

function traditional_def(x)            # function_definition (signature)
    return x
end

macro my_macro(x)                      # macro_definition (signature)
    return x
end

struct PlainStruct                     # struct_definition (type_head) -- bare name
    a::Int
end

struct BoundStruct <: Number           # struct_definition (type_head) -- binary_expression "<:"
    b::Int
end

abstract type AbstractThing end        # abstract_definition (type_head)

short_def(x) = x + 1                   # assignment . (call_expression . (identifier))
typed_short_def(x)::Int = x + 1        # assignment . (typed_expression . (call_expression . (identifier)))

const PLAIN_CONST = 1                  # const_statement (assignment . (identifier))
const TYPED_CONST::Int = 2             # const_statement (assignment . (typed_expression . (identifier)))

# --- types: typed_expression / parametrized_type_expression / curly --------

function types_probe(
    a::Int,                            # typed_expression (identifier)
    b::Base.Int,                       # typed_expression (field_expression (_) (identifier)) -- qualified
    c::Vector{Int},                    # parametrized_type_expression (identifier) "Vector" + curly_expression (identifier) "Int"
    d::Dict{String,Any},               # curly_expression with two (identifier) type args
)
    return (a, b, c, d)
end

# --- NEGATIVE: constructs that must NOT match specific captures ------------

# Not a call: this is `function even_ratio(x) ... end`'s signature head --
# the same call_expression shape as a real call, so calls.scm's broad
# identifier-callee pattern DOES also match it (documented KNOWN LIMITATION
# in julia.calls.scm -- not something this fixture can assert away).
function even_ratio(x)
    return x / 2
end

# Not a selected import: a plain dotted import has no selected_import node
# at all, so it must never produce an @import.name capture.
import Base.Iterators.Stateful          # import_path only, no selected_import

# Not a qualified-type match: a bare (unqualified) type annotation must not
# be captured by the field_expression-based qualified-type pattern.
function bare_type_negative(x::Int)
    return x
end

# Not a supertype/bound reference: a `<` (strict less-than) comparison must
# not be captured by the `<:` supertype pattern -- different operator token.
function not_a_bound(n)
    return n < 10
end

# Not a short-circuit complexity node: arithmetic/comparison binary_expression
# operators (+, *, ==, <) must not match the &&/|| complexity pattern.
function not_short_circuit(a, b)
    return a + b == a * b
end
