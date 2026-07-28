require 'json'
require 'set'
require_relative 'support/helpers'

# Mixin module used by Stack below.
module Loggable
  include Comparable
  include ActiveSupport::Concern # namespaced include (scope_resolution arg)

  def log(msg)
    puts "[#{self.class}] #{msg}"
  end
end

# A simple stack data structure
class Stack
  include Loggable
  extend Forwardable

  attr_accessor :label
  attr_reader :max_size

  # Class-level factory method via singleton-class reopening. Methods inside
  # `class << self ... end` parse as plain `method` nodes, not
  # `singleton_method` — see ruby.tags.scm's comment on this.
  class << self
    def empty
      new
    end
  end

  def initialize
    @data = []
  end

  def push(item)
    @data.push(item)
    self
  end

  def pop
    if @data.empty?
      raise "Stack is empty"
    end
    @data.pop
  rescue StandardError => e
    log("pop failed: #{e.message}")
    nil
  end

  def peek
    @data.last
  end

  def empty?
    @data.empty?
  end

  def size
    @data.size
  end

  def top_label
    label&.upcase
  end
end

# Inheritance: BoundedStack < Stack (plain constant superclass).
class BoundedStack < Stack
  def initialize(max)
    super()
    @max_size = max
  end

  def push(item)
    raise "full" if size >= max_size
    super
  end
end

# Struct-based value class: a very common lightweight-record idiom.
Point = Struct.new(:x, :y) do
  def distance
    Math.sqrt(x**2 + y**2)
  end
end

# Classify a number as negative, zero, or positive
def classify(n)
  if n < 0
    :negative
  elsif n == 0
    :zero
  else
    :positive
  end
end

# Sum elements in an array that satisfy a predicate
def sum_if(arr, &block)
  total = 0
  arr.each do |x|
    total += x if block.call(x)
  end
  total
end

# Pattern-matching classification (Ruby 2.7+ `case ... in ...`).
def describe(value)
  case value
  in Integer => n if n.negative?
    "negative int"
  in Integer
    "int"
  in String
    "string"
  else
    "other"
  end
end

# Keyword args + splat/double-splat, statement modifiers, and a block passed
# via `&block` forwarded to `yield`.
def build_report(title:, sections: [], *extra, **opts)
  puts "building #{title}" if opts[:verbose]
  sections.each { |s| puts s }
  extra
end

def with_yield
  yield 1 if block_given?
end

stack = Stack.new
stack.push(1).push(2).push(3)
puts stack.pop
puts classify(-5)
puts sum_if([1, 2, 3, 4, 5]) { |x| x.even? }
puts describe(42)
puts Integer("5")
