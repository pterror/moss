open List
open Printf

(* Module for stack operations *)
module Stack = struct
  type 'a t = { mutable items : 'a list }

  let create () = { items = [] }

  let push s x = s.items <- x :: s.items

  let pop s =
    match s.items with
    | [] -> None
    | x :: rest ->
        s.items <- rest;
        Some x

  let is_empty s = s.items = []
end

(* Type definition for a binary tree *)
type 'a tree =
  | Leaf
  | Node of 'a * 'a tree * 'a tree

(* Insert into BST *)
let rec insert x = function
  | Leaf -> Node (x, Leaf, Leaf)
  | Node (y, left, right) ->
      if x < y then Node (y, insert x left, right)
      else if x > y then Node (y, left, insert x right)
      else Node (y, left, right)

(** Classify a number as negative, zero, or positive. *)
[@inline]
let classify n =
  if n < 0 then "negative"
  else if n = 0 then "zero"
  else "positive"

(* Sum of even numbers in a list *)
let sum_evens lst =
  fold_left (fun acc x -> if x mod 2 = 0 then acc + x else acc) 0 lst

let () =
  let s = Stack.create () in
  Stack.push s 1;
  Stack.push s 2;
  printf "%s\n" (classify 5);
  printf "%d\n" (sum_evens [1; 2; 3; 4; 5])

(* Exception raised when a queue is drained empty. *)
exception Empty_queue

(* Record type with a deriving attribute (ppx_deriving) and a
   first-class-function field. *)
type 'a queue = { mutable front : 'a list; run_hook : unit -> unit }
[@@deriving show]

let make_queue () = { front = []; run_hook = (fun () -> ()) }

(* Field-function call: q.run_hook () — record field holding a function. *)
let notify q = q.run_hook ()

let dequeue q =
  match q.front with
  | [] -> raise Empty_queue
  | x :: rest ->
      q.front <- rest;
      x

let try_dequeue q = try Some (dequeue q) with Empty_queue -> None

(* Operator definition: idiomatic option-monad bind. *)
let ( >>= ) opt f = match opt with None -> None | Some x -> f x

let chained = Some 1 >>= (fun x -> Some (x + 1)) >>= fun y -> Some (y * 2)

(* Module type (signature) for a comparable element. *)
module type COMPARABLE = sig
  type t

  val compare : t -> t -> int
end

(* Functor parameterized over a COMPARABLE module. *)
module Make_set (Ord : COMPARABLE) = struct
  type elt = Ord.t

  let is_sorted lst =
    let rec go = function
      | a :: (b :: _ as rest) -> Ord.compare a b <= 0 && go rest
      | _ -> true
    in
    go lst
end

module IntCompare = struct
  type t = int

  let compare = Stdlib.compare
end

module IntSet = Make_set (IntCompare)

(* Re-export List's interface under a local module, plus a local addition. *)
module ListExt = struct
  include List

  let last lst = List.nth lst (List.length lst - 1)
end

(* for/while loops — real-world control-flow idioms not otherwise exercised. *)
let sum_to n =
  let total = ref 0 in
  for i = 1 to n do
    total := !total + i
  done;
  !total

let count_down n =
  let i = ref n in
  while !i > 0 do
    i := !i - 1
  done;
  !i
