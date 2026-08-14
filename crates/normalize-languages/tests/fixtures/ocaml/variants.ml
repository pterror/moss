(* Completeness matrix: one construct per node-type variant found while
   cross-referencing ocaml.{calls,tags,imports,decorations,complexity}.scm
   against arborium-ocaml 2.17.0's node-types.json (see
   docs/query-testing-methodology.md). Each construct is commented with the
   field/variant it exercises. A NEGATIVE section at the bottom lists
   near-miss constructs that must NOT match the patterns they resemble. *)

(* --- calls.scm: application_expression.function variants --------------- *)

let plain_fn x = x

(* function: (value_path . (value_name)) — unqualified call *)
let call_plain = plain_fn 1

module Qual = struct
  let f x = x
end

(* function: (value_path (module_path) (value_name)) — qualified call *)
let call_qualified = Qual.f 1

(* function: (value_path . (parenthesized_operator)) — bare operator call *)
let call_operator = ( + ) 1 2

module Qual_op = struct
  let ( + ) a b = a - b
end

(* function: (value_path (module_path) (parenthesized_operator)) — qualified
   operator call *)
let call_qualified_operator = Qual_op.( + ) 1 2

type has_run = { run : unit -> int }

let hr = { run = (fun () -> 1) }

(* function: (field_get_expression record: (_) field: (field_path
   (field_name))) — record field holding a function, applied directly *)
let call_field_function = hr.run ()

(* function: (parenthesized_expression) — dynamically-selected callee, no
   static name; best-effort whole-node capture *)
let call_parenthesized = (if true then plain_fn else plain_fn) 1

class point_obj =
  object
    method dist = 0.0
  end

let po = new point_obj

(* function: (method_invocation object: (_) method: (method_name)) — object
   method call *)
let call_method = po#dist

(* --- tags.scm: let_binding.pattern / type_binding.name / external -------- *)

(* pattern: (value_name) — plain value/function definition *)
let plain_value = 1

(* pattern: (parenthesized_operator) — operator definition *)
let ( >< ) a b = a + b

(* pattern: (typed_pattern pattern: (value_name)) — parenthesized
   type-annotated name *)
let (typed_value : int) = 2

(* pattern: (typed_pattern pattern: (parenthesized_operator)) — annotated
   operator definition *)
let ((><<) : int -> int -> int) = ( + )

(* external + value_name — FFI/primitive declaration *)
external raw_add : int -> int -> int = "%addint"

(* external + parenthesized_operator — stdlib-style primitive operator
   external *)
external ( +% ) : int -> int -> int = "%addint"

module type SIG_WITH_VAL = sig
  (* value_specification — signature-level val declaration (.mli idiom) *)
  val sig_value : int -> int
end

(* name: (type_constructor) — plain type definition *)
type plain_type = int

(* name: (type_constructor_path (type_constructor)) — type extension *)
type extensible = ..

type extensible += Extra_case of string

exception Sample_error of string

class sample_class =
  object
    val mutable state = 0
    method get_state = state
    method set_state n = state <- n
  end

class type sample_class_type =
  object
    method get_state : int
  end

(* --- imports.scm: open_module vs include_module ------------------------ *)

open List

include List

(* --- decorations.scm: attribute vs item_attribute vs floating_attribute - *)

(** Doc comment above a definition. *)
[@inline]
let decorated_fn x = x

type deriving_type = { field : int } [@@deriving show]

[@@@warning "-30"]

(* --- complexity.scm: for/while/try alongside if/match ------------------- *)

let uses_for n =
  let acc = ref 0 in
  for i = 1 to n do
    acc := !acc + i
  done;
  !acc

let uses_while n =
  let i = ref n in
  while !i > 0 do
    i := !i - 1
  done;
  !i

let uses_try () = try raise Sample_error "boom" with Sample_error _ -> ()

(* ------------------------------------------------------------------------
   NEGATIVE cases — constructs that must NOT match the patterns above.
   ------------------------------------------------------------------------ *)

(* Tuple destructuring bind: no single static name, tags.scm intentionally
   does not tag this as @definition.function (matches the convention other
   languages use for destructuring lets). *)
let (neg_a, neg_b) = (1, 2)

(* List/constructor-pattern destructuring bind: same reasoning as above. *)
let neg_head :: neg_tail = [ 1; 2; 3 ]

(* Plain field access (no application) — must NOT match calls.scm's
   field_get_expression-as-callee pattern (there is no enclosing
   application_expression). *)
let neg_field_read = hr.run

(* Bare value reference (no application) — must NOT match any @call pattern,
   only a value_path with no enclosing application_expression. *)
let neg_bare_ref = plain_fn

(* `open` (not `include`) of the same module — the import path text is
   identical to the include case above but the encompassing @import span
   must come from the open_module pattern, not include_module. *)
open Printf
