defmodule MathUtils do
  alias Enum, as: E

  @doc "Classify a number as :negative, :zero, or :positive"
  def classify(n) do
    cond do
      n < 0 -> :negative
      n == 0 -> :zero
      true -> :positive
    end
  end

  @doc "Sum elements matching the predicate"
  def sum_if(list, predicate) do
    Enum.reduce(list, 0, fn x, acc ->
      if predicate.(x), do: acc + x, else: acc
    end)
  end

  def sum_evens(numbers) do
    sum_if(numbers, fn n -> rem(n, 2) == 0 end)
  end

  # Guard clauses: a very common idiom for dispatching on argument shape
  # without a full pattern match. Guarded heads are structurally distinct
  # from plain ones (arguments -> binary_operator "when", not arguments ->
  # call/identifier directly).
  def double(n) when is_integer(n) or is_float(n), do: n * 2
  def double(n) when is_list(n), do: Enum.map(n, &(&1 * 2))

  defguard is_percentage(x) when is_number(x) and x >= 0 and x <= 100

  defp clamp(x) when is_percentage(x), do: x
  defp clamp(x) when x < 0, do: 0
  defp clamp(_x), do: 100
end

defmodule ControlFlowDemo do
  def describe(n) do
    if n < 0 do
      :negative
    else
      :positive
    end
  end

  def doubled_positives(list) do
    for x <- list, x > 0 do
      x * 2
    end
  end

  def safe_div(a, b) do
    try do
      a / b
    rescue
      ArithmeticError -> :error
    catch
      :exit, _ -> :exited
    after
      :ok
    end
  end

  # `with` — chained pattern matches with a fall-through `else`. Idiomatic
  # for railway-style error handling in real Elixir code.
  def fetch_pair(map) do
    with {:ok, x} <- Map.fetch(map, :a),
         {:ok, y} <- Map.fetch(map, :b) do
      {:ok, x + y}
    else
      :error -> {:error, :missing_key}
    end
  end
end

defmodule Stack do
  import Enum, only: [reverse: 1]

  defstruct items: []

  def new(), do: %Stack{}

  def push(%Stack{items: items}, item) do
    %Stack{items: [item | items]}
  end

  # Multiple pattern-matched function clauses dispatching on struct shape —
  # each clause is a separate top-level `def` call, not a branch inside one.
  def pop(%Stack{items: []}) do
    {:error, :empty}
  end

  def pop(%Stack{items: [head | tail]}) do
    {:ok, head, %Stack{items: tail}}
  end

  def peek(%Stack{items: []}), do: nil
  def peek(%Stack{items: [head | _]}), do: head

  def to_list(%Stack{items: items}), do: reverse(items)
end

defmodule Stack.Namespaced do
  @moduledoc "Nested module name: a dotted alias is a single leaf token."
  def ping, do: :pong
end

defprotocol Sized do
  @spec size(t) :: non_neg_integer()
  def size(value)
end

defimpl Sized, for: Stack do
  def size(%Stack{items: items}), do: length(items)
end

defmodule Registry.Multi do
  # Multi-alias/import form: `alias Foo.{Bar, Baz}` — extremely common
  # idiomatic Elixir for pulling in several sibling modules from the same
  # namespace in one line.
  alias Stack.{Namespaced}
  alias MathUtils, as: Utils

  def run do
    Namespaced.ping()
    Utils.double(2)
  end
end

defmodule Counter do
  @moduledoc """
  A minimal GenServer-style behaviour, exercising `use`, module attributes,
  and message-receive control flow — idiomatic OTP.
  """
  use GenServer

  @impl true
  def init(state), do: {:ok, state}

  @impl true
  def handle_call(:get, _from, state), do: {:reply, state, state}

  def loop(state) do
    receive do
      {:inc, n} -> loop(state + n)
      :stop -> state
    end
  end
end

defmodule Macros.Demo do
  # Macro definitions: defmacro/defmacrop are calls just like def/defp, but
  # generate code at compile time via `quote`/`unquote`.
  defmacro trace(expr) do
    quote do
      IO.inspect(unquote(expr))
    end
  end

  defmacrop double_expr(x) when is_atom(x) do
    quote do: unquote(x) * 2
  end
end
