# Completeness fixture: one construct per grammar-legal variant of each field
# the ruby.{tags,calls,imports,complexity,types}.scm queries constrain, cross-
# referenced against arborium-ruby's node-types.json (v2.17.0). Every
# construct here is *expected to be captured*; see query_fixtures.rs's
# `ruby_*_completeness_*` tests for the matrix. A dedicated NEGATIVE section
# at the end holds near-miss constructs that must NOT match.

require 'json'                      # require: string import.path
require_relative 'other'            # require_relative: string import.path
load 'plain.rb'                     # load: string import.path
using RefinementModule               # using: constant import.path (refinement activation)

module Outer
  # class.name: constant
  class Inner < StandardError
    # superclass: constant (StandardError above)

    include Comparable                    # include: constant argument
    include ActiveSupport::Concern         # include: scope_resolution argument
    extend Forwardable                     # extend: constant argument
    extend MyLib::Extensions                # extend: scope_resolution argument
    prepend MyModule::Prependable           # prepend: scope_resolution argument

    attr_accessor :name
    attr_reader :id
    attr_writer :value

    # singleton_class value: (self) — intentionally NOT captured as a
    # definition (no name field exists for `self`); the method inside still
    # gets picked up as a plain `method` node.
    class << self
      def build
        new
      end
    end

    def initialize(name)
      @name = name
    end

    # method.name: operator ( def + ) — a definition, distinct from the
    # call-side `method: operator` variant that ruby.calls.scm documents as
    # out of scope.
    def +(other)
      Inner.new(@name + other.name)
    end

    # method.name: setter ( def foo= )
    def name=(value)
      @name = value
    end

    def method_missing(name, *args, &block)
      super
    end
  end
end

# --- call_expression.method variants ----------------------------------------

def plain_call
  identity(1) # method: identifier
end

def method_call_with_receiver
  v = []
  v.length # method: identifier, receiver: _primary (array literal)
end

def bare_constant_call
  Integer("5") # method: constant (Kernel-style bare call)
end

def identity(x)
  x
end

# --- class.name / module.name variants --------------------------------------

class Plain # name: constant
end

module Outer2 # name: constant
  class Nested < Plain # nested container + plain superclass
  end
end

class Namespaced::Deep < Outer2::Nested # class.name: scope_resolution (rare
  # but legal top-level form: reopening/defining via an explicit path)
end

# --- superclass variants -----------------------------------------------------

class PlainSuper < Plain
end

class NamespacedSuper < Outer2::Nested
end

# Dynamic/computed superclass: Struct.new(...) — no static name for the
# resulting anonymous class; ruby.types.scm captures the call's receiver
# constant ("Struct") as a best-effort partial signal.
class DynamicSuper < Struct.new(:x, :y)
end

# --- complexity: statement-modifier and pattern-matching variants -----------

def statement_modifiers(x)
  puts "positive" if x > 0      # if_modifier
  puts "not zero" unless x == 0 # unless_modifier
  puts "loop" while x < 10      # while_modifier
  puts "loop2" until x > 10     # until_modifier
  value = risky rescue nil      # rescue_modifier
  value
end

def risky
  1
end

def elsif_chain(n)
  if n < 0
    :negative
  elsif n == 0 # elsif (first)
    :zero
  elsif n < 10 # elsif (second) — each elsif is an independent branch node
    :small
  else
    :large
  end
end

def pattern_match(value)
  case value
  in Integer => n if n.negative? # in_clause, wrapped in case_match
    :negative_int
  in Integer
    :int
  in [Integer, Integer] # in_clause with array pattern
    :pair
  else
    :other
  end
end

# --- NEGATIVE cases: must not be captured -----------------------------------

class NegativeHolder
  def initialize
    @field = 1
  end

  def field
    @field
  end
end

def negative_cases(holder)
  # A hash literal is not a block: must never be captured as @nesting via
  # the (block) pattern, and must never appear as a call.
  h = { a: 1, b: 2 }

  # A bare method call with no parens (`holder.field`) IS a call (method:
  # identifier "field") — included here as a contrast to the next line, not
  # as a negative case itself.
  read_via_call = holder.field

  # A local-variable reference must never be captured by the bare
  # identifier/constant reference.call pattern (`#is-not? local` guards
  # this) — `read_via_call` here is a plain local read, not a call.
  bound = read_via_call

  # An ordinary `if` must not itself be double-counted as an `if_modifier`;
  # the two are mutually exclusive node types for the same keyword.
  if bound
    :truthy
  end

  h
end
