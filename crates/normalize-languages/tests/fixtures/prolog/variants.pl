% Prolog completeness matrix — one small construct per node-type variant
% cross-referenced against arborium-prolog-2.17.0's node-types.json, plus a
% NEGATIVE section for constructs that must NOT match.

:- module(variants, [plain_fact/0]).

:- use_module(library(lists)).

% --- tags.scm: clause head variants ---------------------------------------

% Simple fact (clause_term -> atom directly): definition.function
plain_fact.

% Compound fact (clause_term -> functional_notation): definition.function
compound_fact(X).

% Rule with atom head (clause_term -> operator_notation -> . (atom) head):
% definition.function, name = "atom_head_rule"
atom_head_rule :- true.

% Rule with functional_notation head, single-atom body (regression check
% for the anchor fix — the body atom "single_goal_body" must NOT also
% produce a definition.function under this same head clause):
functional_head_rule(X) :- single_goal_body(X).

single_goal_body(_).

% --- tags.scm: directive variants ------------------------------------------

% :- module(...) — the only directive that should tag definition.module.
:- module(inner_probe_unused, []).

% --- calls.scm: functional_notation call (existing coverage) --------------

calls_functional(X) :- helper_predicate(X).

helper_predicate(_).

% --- calls.scm: bare-atom goal call variants -------------------------------

% Single-atom rule body: bare_body_call
bare_body_rule :- bare_body_call.

bare_body_call.

% Conjunction operands (comma): cut (!) plus a user 0-arity predicate
conjunction_rule :- !, conjunction_goal.

conjunction_goal.

% Disjunction operands (semicolon): two bare-atom alternatives
disjunction_rule :- disjunction_left ; disjunction_right.

disjunction_left.
disjunction_right.

% If-then operand (->): bare-atom condition and bare-atom then-branch
if_then_rule :- if_then_condition -> if_then_then.

if_then_condition.
if_then_then.

% --- Prolog idioms beyond the original sample ------------------------------

% DCG rule (--> ), a distinct clause form (operator_notation, operator "-->")
greeting --> [hello], [world].

% Negation as failure (\+)
negation_rule(X) :- \+ member(X, []).

% findall/3 (meta-predicate, ordinary functional_notation call)
findall_rule(L) :- findall(X, member(X, [1, 2, 3]), L).

% --- NEGATIVE cases: must NOT match as @name or @call ----------------------

% Arguments/data atoms inside a functional_notation call must never be
% captured as @name (tags) or as a spurious extra @call beyond the functor
% itself: "not_a_call_arg" here is plain data, not a goal.
negative_data_arg(X) :- helper_predicate(not_a_call_arg).

% An atom that is a list element must never be captured as a call.
negative_list_element :- member(not_a_call_element, [not_a_call_element]).
