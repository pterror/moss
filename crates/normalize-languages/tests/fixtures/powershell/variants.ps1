# Completeness matrix for PowerShell query files (tags/calls/complexity/imports/types/cfg).
#
# Every construct below is commented with the field/node-type variant it
# exercises, cross-referenced against node-types.json (arborium-powershell
# 2.17.0) and verified against real parse output via `normalize syntax ast`
# / `normalize syntax query`. A dedicated NEGATIVE section at the bottom
# holds near-miss constructs that must NOT match specific patterns.

# ---------------------------------------------------------------------------
# tags.scm: function_statement / class_statement / class_method_definition /
# enum_statement / enum_member
# ---------------------------------------------------------------------------

function Get-PlainFunction {
    # function_statement -> function_name  (@definition.function)
    return 1
}

class PlainClass {
    # class_statement -> simple_name  (@definition.class)
    [int]$Field  # class_property_definition -- intentionally untagged, see powershell.tags.scm

    PlainClass() {
        # class_method_definition (constructor, no return type) -> simple_name (@definition.method)
    }

    [string] Greet() {
        # class_method_definition (typed return) -> simple_name (@definition.method);
        # the leading [string] return type is a type_literal, NOT a second
        # simple_name collision candidate for the method's own name.
        return "hi"
    }
}

enum PlainEnum {
    # enum_statement -> simple_name (@definition.type)
    First    # enum_member -> simple_name (@definition.constant)
    Second   # enum_member -> simple_name (@definition.constant)
}

# ---------------------------------------------------------------------------
# calls.scm: command.command_name field variants + invokation_expression
# ---------------------------------------------------------------------------

Get-Item .                       # command_name: (command_name) -- static call

$cmdName = "Get-Item"
& $cmdName .                     # command_name: (command_name_expr) -> path_command_name -> variable

Invoke-Expression "Get-Item ."   # command_name: (command_name) again, ordinary static call

$obj = New-Object PSObject
$obj.ToString()                  # invokation_expression (method call on member_access)
[Math]::Abs(-1)                  # invokation_expression (static call on type_literal)

# ---------------------------------------------------------------------------
# complexity.scm: do_statement (do-while / do-until), trap_statement,
# per-clause switch_clause counting (default clause excluded)
# ---------------------------------------------------------------------------

function Test-DoWhile {
    $i = 0
    do {
        # do_statement, condition: while_condition, keyword "while"
        $i++
    } while ($i -lt 3)
}

function Test-DoUntil {
    $i = 0
    do {
        # do_statement, condition: while_condition, keyword "until" -- same
        # node type/field shape as do-while; only the literal keyword differs.
        $i++
    } until ($i -ge 3)
}

trap {
    # trap_statement -- alternate script-level error handler, structurally
    # analogous to catch_clause (statement_block + type_literal children).
    Write-Host "trapped"
    continue
}

function Test-SwitchClauses {
    param($x)
    switch ($x) {
        1 { "one" }          # switch_clause with non-empty switch_clause_condition -> +1 complexity
        2 { "two" }          # switch_clause with non-empty switch_clause_condition -> +1 complexity
        default { "other" }  # switch_clause with EMPTY switch_clause_condition -> must NOT count
    }
}

function Test-SwitchRegexClauses {
    param($x)
    switch -Regex ($x) {
        # under -Regex, EVERY switch_clause_condition is childless (grammar
        # quirk verified via `normalize syntax ast`) -- distinguished from
        # `default` only by the condition's own text, not by node structure.
        "^a" { "starts with a" }   # non-default -> +1 complexity
        default { "no match" }     # default -> must NOT count
    }
}

# ---------------------------------------------------------------------------
# imports.scm: Import-Module, dot-sourcing (command_invokation_operator "."
# + command_name_expr), using module / using namespace
# ---------------------------------------------------------------------------

Import-Module PSReadLine                    # plain Import-Module
. "$PSScriptRoot/lib.ps1"                    # dot-sourcing, quoted path (string_literal)
. $PSScriptRoot\lib2.ps1                     # dot-sourcing, bare path (path_command_name -> variable)
using namespace System.Collections.Generic   # using namespace -- positional generic_token pair
using module MyModule                        # using module -- positional generic_token pair

# ---------------------------------------------------------------------------
# types.scm: type_spec's five child variants -- type_name, array_type_name,
# generic_type_name (+ nested generic_type_arguments), dimension
# ---------------------------------------------------------------------------

[int]$plainType = 0                                          # type_spec -> type_name (plain)
[int[]]$arrayType = @(1, 2)                                  # type_spec -> array_type_name -> type_name
[System.Collections.Generic.List[int]]$genericType = $null   # type_spec -> generic_type_name -> type_name (dotted path)
                                                               # + nested generic_type_arguments -> type_spec -> type_name ("int")
[int[,]]$multiDimType = New-Object 'int[,]' 2, 2              # type_spec -> array_type_name -> dimension (rank 2)

# ---------------------------------------------------------------------------
# cfg.scm: if/elseif/else, switch, for/foreach, while/do, try/catch/finally,
# flow_control_statement exits (return/break/continue/throw)
# ---------------------------------------------------------------------------

function Test-Cfg {
    param($n)
    if ($n -gt 0) {
        # if_statement: condition, then (unnamed statement_block), elseif_clauses field
        return "positive"
    } elseif ($n -eq 0) {
        # elseif_clause: condition + unnamed statement_block
        return "zero"
    } else {
        # else_clause: unnamed statement_block
        return "negative"
    }

    for ($i = 0; $i -lt 3; $i++) {
        # for_statement: for_condition field, unnamed statement_block
        if ($i -eq 1) { continue }   # flow_control_statement "continue"
        if ($i -eq 2) { break }      # flow_control_statement "break"
    }

    foreach ($x in @(1, 2, 3)) {
        # foreach_statement: no field names, "in" literal + condition + unnamed statement_block
        Write-Host $x
    }

    while ($n -gt 0) {
        # while_statement: condition: while_condition, unnamed statement_block
        $n--
    }

    try {
        throw "boom"   # flow_control_statement "throw" + trailing pipeline
    } catch {
        # catch_clause
        Write-Host "caught"
    } finally {
        # finally_clause
        Write-Host "cleanup"
    }
}

# ---------------------------------------------------------------------------
# NEGATIVE section: constructs that must NOT match specific patterns
# ---------------------------------------------------------------------------

# Not a call: bare member access with no invocation (no argument_list/
# invokation_expression node at all -- this is a `member_access` read, not
# a call) -- must NOT appear in calls.scm's @call captures.
$plainMemberAccess = $obj.SomeProperty

# Not a dot-source import: "." used as a decimal point / range operator
# context is structurally impossible to confuse with command_invokation_operator
# since it never appears as a `command` child there -- included as documentation
# that the disambiguating field is command_invokation_operator, not the "."
# text alone appearing anywhere in the file.
$notDotSource = 3.14

# Not "using module/namespace": a plain command literally named "using" with
# an unrelated first argument keyword must NOT match the using-import pattern
# (the keyword guard requires "module"/"namespace"/"assembly").
using system

# Not a default-clause miss: a switch clause whose pattern is the quoted
# string "default" (not the bare keyword) must still count as a non-default
# branch -- distinguishing token identity ("default") from token text that
# merely contains the same letters inside quotes.
function Test-QuotedDefaultClause {
    param($x)
    switch ($x) {
        "default" { "quoted, not the keyword" }
        default { "the real default" }
    }
}
