-module(sample).

-import(lists, [foldl/3, filter/2]).

-export([classify/1, sum_evens/1, factorial/1, member/2]).

-record(point, {x :: integer(), y :: integer()}).

-type coordinate() :: {integer(), integer()}.

-type handler() :: fun((coordinate()) -> ok | error).

-callback handle(Msg :: term()) -> ok | {error, atom()}.

-spec classify(integer()) -> negative | zero | positive.
%% Classify a number as negative, zero, or positive
classify(N) when N < 0 ->
    negative;
classify(0) ->
    zero;
classify(_N) ->
    positive.

%% Sum the even numbers in a list
sum_evens(List) ->
    Evens = filter(fun(X) -> X rem 2 =:= 0 end, List),
    foldl(fun(X, Acc) -> X + Acc end, 0, Evens).

%% Compute factorial recursively
factorial(0) ->
    1;
factorial(N) when N > 0 ->
    N * factorial(N - 1).

%% Check if an element is in a list
member(_Elem, []) ->
    false;
member(Elem, [Elem | _Rest]) ->
    true;
member(Elem, [_ | Rest]) ->
    member(Elem, Rest).

%% Create a point record
make_point(X, Y) ->
    #point{x = X, y = Y}.

%% Sort a list via a remote (module-qualified) call
sorted(List) ->
    lists:sort(List).

%% Classify via an if-expression (guards as conditions)
sign(X) ->
    if
        X > 0 -> positive;
        X < 0 -> negative;
        true -> zero
    end.

%% Receive a message with a timeout branch
wait_for_message() ->
    receive
        {msg, X} -> X
    after 1000 ->
        timeout
    end.

%% Try/catch/after around a fallible call
safe_call() ->
    try
        do_something()
    catch
        error:Reason -> {error, Reason};
        throw:Value -> {caught, Value}
    after
        cleanup()
    end.

%% Explicit throw via the erlang module
fail(Reason) ->
    erlang:throw(Reason).

%% Bare throw/exit/error (auto-imported BIFs, no `erlang:` prefix — the
%% common real-world form)
bare_fail(bad) ->
    error(badarg);
bare_fail(stop) ->
    exit(normal);
bare_fail(Reason) ->
    throw(Reason).

%% Invoke a callback held in a variable (higher-order function idiom)
apply_callback(Fun, Arg) ->
    Fun(Arg).

%% Dynamic dispatch: module and/or function resolved at runtime
dispatch(Mod, Fun, Args) ->
    Mod:Fun(Args).

%% Dynamic dispatch with a literal function name
dispatch_module(Mod, Arg) ->
    Mod:handle(Arg).

%% try ... of ... after ... end (pattern-match the result, no catch)
try_of_after(List) ->
    try lists:sort(List) of
        [] -> empty;
        Sorted -> Sorted
    after
        cleanup()
    end.

%% try ... catch Class:Reason -> ... end (variable class — catches any
%% exception class, the far more common form than a literal atom class)
try_catch_any_class() ->
    try do_something() catch
        Class:Reason -> {Class, Reason}
    end.

%% try ... catch Pattern -> ... end (bare pattern, no explicit class —
%% implicitly catches `throw` only)
try_catch_bare_pattern() ->
    try do_something() catch
        Reason -> {caught, Reason}
    end.

%% receive with only a timeout arm (non-blocking mailbox drain idiom)
flush() ->
    receive
    after 0 ->
        ok
    end.
