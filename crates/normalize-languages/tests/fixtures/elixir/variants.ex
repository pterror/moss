defmodule Variants do
  # --- tags: def/defp/defmacro/defmacrop/defguard/defguardp/defdelegate ----

  # call.target: identifier, arguments -> call target: identifier (plain args form)
  def plain_call(x), do: x

  # call.target: identifier, arguments -> identifier (no-args form)
  def plain_noargs, do: :ok

  # arguments -> binary_operator (operator "when") left: call (guarded, args form)
  def guarded_call(x) when is_integer(x), do: x

  # arguments -> binary_operator (operator "when") left: identifier (guarded, no-args form)
  def guarded_noargs when true, do: :ok

  defp private_plain(x), do: x
  defp private_guarded(x) when is_atom(x), do: x

  defmacro macro_plain(x) do
    quote do: unquote(x)
  end

  defmacro macro_guarded(x) when is_atom(x) do
    quote do: unquote(x)
  end

  defmacrop macrop_plain(x) do
    quote do: unquote(x)
  end

  defmacrop macrop_guarded(x) when is_atom(x) do
    quote do: unquote(x)
  end

  defguard guard_expr(x) when is_number(x) and x > 0
  defguardp guardp_expr(x) when is_number(x) and x < 0

  defdelegate delegated_call(x), to: Kernel
  defdelegate delegated_noargs, to: Kernel

  # defmodule name: alias (plain)
  defmodule Plain do
    def inner, do: :ok
  end

  # defmodule name: alias (dotted path lexes as ONE alias token, not nested
  # dot nodes — verified via `normalize syntax ast`)
  defmodule Deep.Nested do
    def inner, do: :ok
  end

  defprotocol PlainProtocol do
    def op(value)
  end

  defimpl PlainProtocol, for: Plain do
    def op(_value), do: :implemented
  end

  # --- calls: local / remote / anon-invocation / dynamic-target -----------

  def calls_demo do
    identity(1)
    Kernel.identity(1)
    holder = %{field: 1}
    holder.field
    add_one = fn x -> x + 1 end
    add_one.(5)
  end

  def identity(x), do: x

  # --- imports: alias/import/use/require, plain / multi / dot-qualified ---

  alias Plain
  alias Deep.Nested, as: DN
  import Kernel, only: [is_atom: 1]
  require Logger
  use Application

  # Multi-alias form: dot right: tuple(alias, alias)
  alias Deep.{Nested}

  # Dot-qualified single form: dot left: identifier ("__MODULE__"), right: alias
  alias __MODULE__.Plain

  # --- complexity: branching macros, stab_clause arms, boolean operators --

  def branch_if(x) do
    if x > 0 do
      :pos
    else
      :nonpos
    end
  end

  def branch_unless(x) do
    unless x > 0 do
      :nonpos
    else
      :pos
    end
  end

  def branch_case(x) do
    case x do
      1 -> :one
      2 -> :two
      _ -> :other
    end
  end

  def branch_cond(x) do
    cond do
      x > 90 -> :a
      x > 80 -> :b
      true -> :f
    end
  end

  def branch_with(map) do
    with {:ok, a} <- Map.fetch(map, :a),
         {:ok, b} <- Map.fetch(map, :b) do
      {:ok, a + b}
    else
      :error -> :fail
    end
  end

  def branch_for(list) do
    for x <- list, x > 0, do: x * 2
  end

  def branch_try(a, b) do
    try do
      a / b
    rescue
      ArithmeticError -> :error
    catch
      :exit, _ -> :exited
    end
  end

  def branch_receive do
    receive do
      {:msg, x} -> x
      _ -> :ignored
    end
  end

  def branch_bool(a, b) do
    (a && b) || (a and b) || (a or b)
  end

  # --- types: alias references in @spec/@behaviour/defimpl ----------------

  @spec typed(Plain.t()) :: {:ok, DN.t()}
  def typed(x), do: {:ok, x}

  @behaviour Application

  # === NEGATIVE: constructs that must NOT match as the wrong kind =========

  def negative_examples do
    # Ordinary arithmetic/comparison operators must never contribute to
    # @complexity — only boolean short-circuit operators (&&, ||, and, or) do.
    plain_arithmetic = 1 + 2 - 3 * 4 / 5
    plain_comparison = 1 == 2

    # A local variable read must never be captured as an @import.path or a
    # @definition.* tag.
    bound = plain_arithmetic

    # `holder2.other_field` IS a call (method: identifier "other_field") —
    # not a negative case by itself, but must never ALSO match the
    # anonymous-invocation `!right` pattern (it has a `right`, so it's a
    # remote call, not an anon-invocation).
    holder2 = %{other_field: 1}
    _ = holder2.other_field

    {plain_comparison, bound}
  end
end
