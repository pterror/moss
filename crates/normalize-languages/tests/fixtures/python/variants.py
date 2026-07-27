# Completeness fixture: one construct per grammar-legal variant of each field
# the python.{tags,calls,imports,complexity,types}.scm queries constrain,
# cross-referenced against arborium-python's node-types.json and verified
# against real parse output via `normalize syntax ast`/`normalize syntax
# query`. Every construct here is *expected to be captured*; see
# query_fixtures.rs `python_*_completeness_*` tests for the matrix.
#
# This file also carries a NEGATIVE section (near-miss constructs) that must
# NOT be captured by the query under test, to guard against over-broad
# patterns.

import typing
from typing import Callable, Optional

# --- call.function variants -------------------------------------------------


def plain_call():
    identity(1)  # function: identifier


def method_call():
    items = []
    items.append(1)  # function: attribute (object: identifier, attribute: identifier)


def subscript_call():
    handlers = {"go": identity}
    handlers["go"](1)  # function: subscript (value: identifier)


def subscript_attribute_call(self_like):
    self_like.handlers["go"](1)  # function: subscript (value: attribute)


def chained_call():
    # function: call (the inner `get_func()` is independently matched as a
    # plain identifier call — no separate handling needed for the outer
    # invocation, which has no static name).
    get_func()(1)


def identity(x):
    return x


def get_func():
    return identity


# --- import variants ---------------------------------------------------------

import os  # import_statement.name: dotted_name
import os.path  # import_statement.name: dotted_name (multi-segment)
import os as os_alias  # import_statement.name: aliased_import

from collections import OrderedDict  # import_from_statement.name: dotted_name
from collections import OrderedDict as OD  # import_from_statement.name: aliased_import
from collections import (defaultdict, Counter)  # parenthesized multi-name
from . import sibling  # import_from_statement.module_name: relative_import (bare dot)
from ..pkg import cousin  # relative_import with dotted_name suffix
from os.path import *  # import_from_statement wildcard_import

# --- tags: module-level constant variants ------------------------------------

PLAIN_CONSTANT = 1  # module (assignment left: identifier)
TUPLE_A, TUPLE_B = 1, 2  # module (assignment left: pattern_list)
ANNOTATED_CONSTANT: int = 3  # module (assignment left: identifier, with type field)

# --- types: annotation variants -----------------------------------------------


def plain_type(x: int) -> None:
    pass


def dotted_type(x: os.Kind) -> None:
    # type: (type (attribute object: (identifier) attribute: (identifier)))
    # (a made-up dotted attribute for fixture purposes; the grammar parses
    # any dotted name here, regardless of whether it actually resolves)
    pass


def multi_segment_dotted_type(x: os.path.Kind) -> None:
    # type: (type (attribute object: (attribute ...) attribute: (identifier)))
    # 3-segment dotted annotation: `object:` nests as another `attribute`,
    # not `identifier` — a distinct completeness case from dotted_type above.
    pass


def forward_ref_string_type(x: "module.Kind") -> None:
    # NEGATIVE-adjacent: string forward-references are NOT structurally
    # parsed as dotted names — the annotation is a `string` node, and its
    # contents are opaque to a single tree-sitter query pattern (no
    # sub-parsing). "module"/"Kind" must NOT appear as @type.reference from
    # this line; see the NEGATIVE section for the explicit assertion.
    pass


def generic_type_bare(x: list) -> Optional[str]:
    # return_type: (type (generic_type (identifier) "Optional" ...))
    pass


def generic_type_multi(x: dict) -> "typing.Dict[str, int]":
    pass


def dotted_generic_type(x: typing.List[int]) -> None:
    # type: (type (subscript value: (attribute) subscript: (identifier)))
    pass


def dotted_generic_type_multi_arg(x: typing.Dict[str, os.PathLike]) -> None:
    # multiple `subscript:` fields, one plain one dotted
    pass


def union_type_two(x: int | None) -> None:
    pass


def union_type_three(x: int | str | None) -> None:
    pass


def splat_type_param[*Ts](x: tuple) -> None:
    pass


def paramspec_type_param[**P](x: int) -> None:
    pass


def callable_arg_list(x: Callable[[int, str], bool]) -> None:
    pass


class TypedClass:
    a: list
    b: int | str
    c: "ForwardRef"


# --- complexity/nesting variants ----------------------------------------------


def branching(items):
    total = 0
    for item in items:  # for_statement: @complexity, @nesting
        if item > 0:  # if_statement: @complexity, @nesting
            total += item
        elif item < -10:  # elif is a nested if_statement: independently counted
            total -= 1
        else:
            continue
    while total > 100:  # while_statement: @complexity, @nesting
        total -= 1
    try:  # try_statement: @complexity, @nesting
        risky()
    except ValueError:  # except_clause: @complexity
        pass
    with open("f") as fh:  # with_statement: @complexity, @nesting
        pass
    return total and total or 0  # "and"/"or": @complexity


def risky():
    pass


def structural_matching(command):
    match command:  # match_statement: @complexity, @nesting
        case {"action": action, **rest}:  # case_clause: @complexity
            return action
        case [first, *rest]:
            return first
        case _:
            return None


def comprehensions(items):
    doubled = [x * 2 for x in items]  # list_comprehension: @complexity
    keyed = {x: x for x in items}  # dictionary_comprehension: @complexity
    uniq = {x for x in items}  # set_comprehension: @complexity
    lazy = (x for x in items)  # generator_expression: @complexity
    return doubled, keyed, uniq, lazy


def conditional_expr(x):
    return "pos" if x > 0 else "neg"  # conditional_expression: @complexity


class NestedClass:  # class_definition: @nesting
    def nested_method(self):  # function_definition: @nesting
        pass


# --- NEGATIVE cases: must not be captured -------------------------------------


def negative_cases(container):
    # A lambda is not a function_definition; must never appear as
    # @definition.function/@definition.method.
    add_one = lambda x: x + 1

    # Bare attribute access with no call parens must never appear in any
    # @call/@reference.call capture (regression guard against over-eager
    # attribute patterns matching plain member access).
    _read = container.field

    # A plain list literal (outside a generic_type/type_parameter position)
    # must never have its elements captured as @type.reference — regression
    # guard against the Callable-arg-list pattern over-firing on ordinary
    # runtime lists.
    _plain_list = [add_one, container]

    # A bare bitwise-or expression outside annotation position must never be
    # captured as @type.reference — regression guard against the PEP 604
    # union-type pattern over-firing on runtime bitwise-or (e.g. combining
    # flag constants).
    _flags = os.O_RDONLY | os.O_CREAT

    # Local (function-scoped) assignment must never be captured as
    # @definition.constant — that capture is module-scope only.
    local_not_constant = 42

    return add_one, _read, _plain_list, _flags, local_not_constant
