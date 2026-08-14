module MathTools

open System
open System.Collections.Generic

// Type definition: a discriminated union
type Shape =
    | Circle of radius: float
    | Rectangle of width: float * height: float
    | Triangle of base_: float * height: float

// Record type
type Point = { X: float; Y: float }

// Class with instance methods, an instance property, and a static factory
type Accumulator(initial: int) =
    let mutable total = initial
    member this.Add(x: int) =
        total <- total + x
        total
    member this.Total = total
    static member Create() = Accumulator(0)

// Compute area of a shape
let area shape =
    match shape with
    | Circle r -> Math.PI * r * r
    | Rectangle(w, h) -> w * h
    | Triangle(b, h) -> 0.5 * b * h

// Classify a number
let classify n =
    if n < 0 then "negative"
    elif n = 0 then "zero"
    else "positive"

// Sum even numbers in a list
let sumEvens values =
    let mutable total = 0
    for v in values do
        if v % 2 = 0 then
            total <- total + v
    total

// Compute factorial recursively
let rec factorial n =
    if n <= 1 then 1
    else n * factorial (n - 1)

// Distance between two points
let distance (a: Point) (b: Point) =
    let dx = b.X - a.X
    let dy = b.Y - a.Y
    Math.Sqrt(dx * dx + dy * dy)

// Loop until a condition holds
let countDown n =
    let mutable i = n
    while i > 0 do
        i <- i - 1
    i

// Safe division with try/with, and a try/finally cleanup
let safeDivide a b =
    try
        a / b
    with
    | :? System.DivideByZeroException -> 0
    | ex -> failwith ex.Message

let withCleanup () =
    try
        1
    finally
        printfn "cleanup"

// Validation that raises on bad input
let validate x =
    if x < 0 then
        raise (System.ArgumentException("must be non-negative"))
    x

// Computation expressions: explicit return/yield
let asyncClassify x =
    async {
        if x < 0 then
            return "negative"
        else
            return "non-negative"
    }

let evenSeq upper =
    seq {
        for i in 1 .. upper do
            if i % 2 = 0 then
                yield i
    }

[<EntryPoint>]
let main _ =
    printfn "%s" (classify -5)
    printfn "%d" (sumEvens [1..10])
    printfn "%d" (factorial 5)
    printfn "%d" (countDown 3)
    printfn "%d" (safeDivide 4 2)
    withCleanup ()
    printfn "%d" (validate 5)
    let acc = Accumulator.Create()
    printfn "%d" (acc.Add(10))
    printfn "%d" acc.Total
    0
