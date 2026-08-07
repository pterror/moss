-- Completeness fixture: one construct per grammar-legal variant of each field
-- the lua.{tags,calls,imports,complexity}.scm queries constrain, cross-
-- referenced against arborium-lua 2.17.0's node-types.json. Every construct
-- here is *expected to be captured*; see query_fixtures.rs `lua_*_completeness_*`
-- tests for the matrix.
--
-- This file also carries deliberate near-miss constructs (marked NEGATIVE)
-- that must NOT be captured by the query under test, to guard against
-- over-broad patterns.

-- --- require(...) variants (lua.imports.scm) --------------------------------

local json = require("paren_module") -- parenthesized string arg
local bare = require "bareword_module" -- bareword call: no parens
local br = require [[bracket_module]] -- long-bracket string arg
local dyn = some_module_var -- NEGATIVE: not a require() call at all

-- --- function_declaration.name variants (lua.tags.scm) ----------------------

function plain_global() -- name: identifier
end

local function plain_local() -- name: identifier (leading `local` token, no distinct node type)
end

local NS = {}
function NS.dotted_fn() -- name: dot_index_expression
end

function NS:method_fn() -- name: method_index_expression
end

-- --- assignment-based function definitions (lua.tags.scm) -------------------

local ident_fn = function() end -- variable_list.name: identifier -> @definition.function
NS.dotted_assign_fn = function() end -- variable_list.name: dot_index_expression -> @definition.method
local dispatch = {}
dispatch["dynamic_key"] = function() end -- NEGATIVE: variable_list.name: bracket_index_expression, no static name

-- --- function_call.name variants (lua.calls.scm) -----------------------------

local function get_handler()
    return plain_global
end

local function call_variants()
    plain_global() -- name: identifier
    NS:method_fn() -- name: method_index_expression
    NS.dotted_fn() -- name: dot_index_expression
    dispatch["key"]() -- name: bracket_index_expression (dispatch-table idiom)
    dispatch[1]() -- name: bracket_index_expression, numeric key
    ;(plain_global)() -- name: parenthesized_expression (IIFE-style)
    get_handler()() -- name: function_call (chained/curried call)
end

-- --- NEGATIVE: near-misses that must not be captured as calls ---------------

local function negative_cases()
    local holder = { field = 1 }
    local _read = holder.field -- bare field access, no call parens: must NOT be a call
    local adder = function(x)
        return function(y) -- closure body must not itself register as a named definition
            return x + y
        end
    end
    local _added = adder(1)(2) -- call chain: only "adder" is a real reference.call target
end

-- --- complexity/nesting variants (lua.complexity.scm) ------------------------

function classify_all(n, tbl)
    if n < 0 then -- if_statement: @complexity, @nesting
        return "negative"
    elseif n == 0 then -- elseif_statement: @complexity (not @nesting)
        return "zero"
    else
        return "positive"
    end

    for i = 1, 10 do -- for_statement (numeric clause): @complexity, @nesting
        print(i)
    end

    for k, v in pairs(tbl) do -- for_statement (generic clause): @complexity, @nesting
        print(k, v)
    end

    local i = 0
    while i < 10 do -- while_statement: @complexity, @nesting
        i = i + 1
    end

    repeat -- repeat_statement: @complexity, @nesting
        i = i - 1
    until i == 0

    local ok = (n > 0) and "pos" or "nonpos" -- "and"/"or": @complexity x2
    return ok
end

-- --- multiple assignment / multiple return (real-world idiom density) -------

local function pair()
    return 1, 2 -- multiple return
end

local a, b = pair() -- multiple assignment from multi-return call
local c, d = 1, 2 -- multiple assignment, literal RHS
