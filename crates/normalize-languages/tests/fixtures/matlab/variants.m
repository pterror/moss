% Completeness fixture: one construct per grammar-legal variant of each
% field the matlab.{tags,calls,imports,complexity,cfg,types}.scm queries
% constrain, cross-referenced against arborium-matlab 2.17.0's
% node-types.json. Every construct here is *expected to be captured*
% (except the NEGATIVE section); see query_fixtures.rs `matlab_*_completeness_*`
% tests for the matrix.
%
% NOTE: `.m` collides with Objective-C's extension in normalize's CLI, so
% this fixture is never resolved by extension guessing anywhere in the test
% harness — tests load it via `GrammarLoader::get("matlab")` explicitly (see
% query_fixtures.rs), the same mechanism sample.m's tests already use.

% --- function_call.name variants (matlab.calls.scm) -------------------------

function test_calls()
    plainCall(1); % name: identifier
    s.dynamicField = @sin;
    y = s.(pickField())(1); % name: indirect_access wrapping identifier -> @call
end

function name = pickField()
    name = 'dynamicField';
end

% --- command-syntax calls vs. import statement (matlab.calls.scm) -----------

function test_commands()
    disp hello % command: real call, captured as @call
    clear all % command: real call, captured as @call
    import matlab.io.* % command syntax but a language statement, NOT a call:
                        % NEGATIVE for matlab.calls.scm, POSITIVE for matlab.imports.scm
end

% --- function_definition / function_signature name variants (matlab.tags.scm)

function plain_function() % function_definition, single return via no function_output
end

function out = single_return_function() % function_definition, function_output: identifier
    out = 1;
end

function [a, b] = multi_return_function() % function_definition, function_output: multioutput_variable
    a = 1;
    b = 2;
end

classdef Interface
    methods (Abstract)
        result = mustImplement(obj) % function_signature.name: identifier -> @definition.function
    end
end

% --- class_definition.name / superclasses variants (matlab.tags.scm, matlab.types.scm)

classdef SingleParent < handle % superclasses -> property_name -> identifier: "handle"
end

classdef MultiParent < handle & matlab.mixin.Copyable
    % superclasses -> property_name (x2) -> identifier: "handle", (chained dotted name)
end

classdef NoParent
    % class_definition with no superclasses child at all: NEGATIVE for matlab.types.scm
    methods
        function obj = NoParent()
            % obj@Superclass(...) qualified call: superclass -> identifier -> @type.reference
            obj = obj@handle();
        end
    end
end

% --- complexity / nesting variants (matlab.complexity.scm) ------------------

function classify_all(n)
    if n < 0 % if_statement: @complexity, @nesting
        result = 'negative';
    elseif n == 0 % elseif_clause: @complexity (not @nesting)
        result = 'zero';
    else
        result = 'positive';
    end

    switch n % switch_statement: @complexity, @nesting
        case 1 % case_clause: @complexity
            result = 'one';
        otherwise % otherwise_clause: @complexity
            result = 'other';
    end

    for k = 1:10 % for_statement: @complexity, @nesting
        disp(k);
    end

    i = 0;
    while i < 10 % while_statement: @complexity, @nesting
        i = i + 1;
    end

    try
        risky();
    catch % catch_clause: @complexity (not its own @nesting entry)
        handle_error();
    end
end

% --- cfg.scm exit variants ---------------------------------------------------

function test_exits(n)
    for k = 1:n
        if k == 5
            break % break_statement: @cfg.exit.break
        end
        if k == 3
            continue % continue_statement: @cfg.exit.continue
        end
    end
    if n < 0
        error('bad n'); % function_call name "error": @cfg.exit.throw
    end
    if n == 0
        rethrow(lasterror()); % function_call name "rethrow": @cfg.exit.throw
    end
    result = n; % NEGATIVE: plain assignment is not an exit node
    return % return_statement: @cfg.exit.return
end

% --- NEGATIVE section: constructs that must NOT match ------------------------

function test_negatives()
    x = 5; % NEGATIVE: plain assignment, not a call, not a definition
    y = x + 1; % NEGATIVE: binary_operator, not a call
    z = notACall; % NEGATIVE: bare identifier reference, not a function_call node
    % NEGATIVE: a comment is never @call, @definition.*, or @complexity
end
