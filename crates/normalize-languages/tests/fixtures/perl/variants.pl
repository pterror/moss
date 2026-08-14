#!/usr/bin/perl
# Completeness matrix for Perl queries (tags/calls/imports/complexity).
# One small, commented construct per node-type field variant found by
# cross-referencing perl.{tags,calls,imports,complexity}.scm's field
# constraints against arborium-perl's node-types.json (see
# docs/query-testing-methodology.md). A dedicated NEGATIVE section at the
# bottom holds near-miss constructs that must NOT match.
#
# Note: subroutine_declaration_statement.name, package_statement.name,
# function_call_expression.function, method_call_expression.method, and
# use_statement.module are each constrained to exactly one node-type
# variant in node-types.json (bareword/package/function/method/package
# respectively) — there is no field-completeness gap to exercise for
# those; this fixture instead focuses on the two structurally distinct
# node *types* the field-complete-but-narrow queries were missing:
# ambiguous_function_call_expression (parenless calls) and the two
# require_expression argument shapes.

use strict;
use warnings;

# ---------------------------------------------------------------------------
# function_call_expression.function vs ambiguous_function_call_expression.function
# (perl.calls.scm @call)
# ---------------------------------------------------------------------------

# function_call_expression: parenthesized call
sub paren_call {
    return plain_func(1, 2);
}

sub plain_func {
    return 1;
}

# ambiguous_function_call_expression: parenless call with arguments
sub parenless_call {
    print "hello\n";
    return 1;
}

# method_call_expression: $obj->method(args) and Class->method(args)
package Widget;

sub new {
    my ($class) = @_;
    return bless {}, $class;
}

sub render {
    my ($self) = @_;
    return 'rendered';
}

package main;

my $widget = Widget->new();
$widget->render();

# ---------------------------------------------------------------------------
# require_expression argument variants (perl.imports.scm @import.path)
# ---------------------------------------------------------------------------

# bareword module form
require Scalar::Util;

# string-literal file-path form
require 'legacy_helpers.pl';

# ---------------------------------------------------------------------------
# use_statement.module (perl.imports.scm @import.path) — single field-type
# variant, included for completeness-matrix symmetry with require above.
# ---------------------------------------------------------------------------

use List::Util qw(sum);

# ---------------------------------------------------------------------------
# for_statement vs cstyle_for_statement (perl.complexity.scm @complexity/@nesting)
# ---------------------------------------------------------------------------

sub foreach_form {
    my @xs = (1, 2, 3);
    for my $x (@xs) {
        print $x;
    }
}

sub cstyle_form {
    for (my $i = 0; $i < 3; $i++) {
        print $i;
    }
}

# ---------------------------------------------------------------------------
# NEGATIVE cases — must not match any of the above queries
# ---------------------------------------------------------------------------

# func1op_call_expression (builtin operator with fixed arity, e.g. `shift`)
# is a structurally distinct node type from both function_call_expression
# and ambiguous_function_call_expression — its `function` field is a
# builtin-keyword token type, not the `function` bareword node type either
# query matches. Confirmed via node-types.json: func1op_call_expression's
# `function` field type list is the closed set of builtin keyword literals
# (shift, keys, pop, ...), never the generic `function` node.
sub uses_builtin {
    my @xs = @_;
    return shift @xs;
}

# A bareword that is merely a hash key or string-ish literal, never inside
# a call_expression's `function` field, must not appear as an @call
# capture.
my %h = (plain_func => 1);

# A bareword statement with NO trailing arguments and no parens (`foo;`
# alone) is a genuine grammar limitation, not a missing query clause: it
# parses as a bare `bareword` expression statement, not as any kind of
# call_expression node at all (verified via `normalize syntax ast` — no
# ambiguous_function_call_expression or function_call_expression wraps
# it). The grammar can't disambiguate a zero-argument bareword from a
# plain string/list constant without a following argument list, so there
# is nothing for the query to capture; this is honestly documented here
# rather than papered over. `bareword_no_args_call` below must NOT be
# captured as an @call.
sub bareword_no_args_call {
    return 1;
}

sub calls_bareword_no_args {
    bareword_no_args_call;
    return 1;
}
