module MathTools

import Statistics
import Statistics: mean, std
using LinearAlgebra: norm, dot
using Random as Rnd

export classify, sum_evens, factorial, square, distance, area, describe

# Module-level constant
const DEFAULT_TOLERANCE::Float64 = 1e-9

# Classify a number
@inline function classify(n::Int)::String
    if n < 0
        return "negative"
    elseif n == 0
        return "zero"
    else
        return "positive"
    end
end

# Sum even numbers in a vector
function sum_evens(values::Vector{Int})::Int
    total = 0
    for v in values
        if v % 2 == 0
            total += v
        end
    end
    return total
end

# Compute factorial recursively
function factorial(n::Int)::Int
    if n <= 1
        return 1
    end
    return n * factorial(n - 1)
end

# Short-form function: square
square(x) = x * x

# Struct definition
struct Point
    x::Float64
    y::Float64
end

# Method on struct
function distance(a::Point, b::Point)::Float64
    return norm([b.x - a.x, b.y - a.y])
end

# Abstract type hierarchy + multiple dispatch
abstract type Shape end

struct Circle <: Shape
    r::Float64
end

mutable struct Rectangle <: Shape
    w::Float64
    h::Float64
end

area(s::Circle) = pi * s.r^2
area(s::Rectangle) = s.w * s.h
function area(s::T) where T <: Shape
    error("area not implemented for $T")
end

# Macro definition + invocation
macro timeit(label, expr)
    return :(println($label, ": ", $expr))
end

@timeit "square" square(4)

# Comprehension, generator, broadcast, ternary, do-block, short-circuit
function describe(shapes::Vector{<:Shape})
    areas = [area(s) for s in shapes if area(s) > 0]
    total = sum(area(s) for s in shapes)
    label = length(areas) > 0 && total > DEFAULT_TOLERANCE ? "nonempty" : "empty"
    radii = Circle.(1:3)
    open("/dev/null") do io
        println(io, label, " ", total)
    end
    try
        Statistics.mean(areas)
    catch e
        rethrow(e)
    end
    return (label, radii)
end

end # module MathTools
