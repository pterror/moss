; Ruby imports query
; @import       — the entire call statement (for line number)
; @import.path  — the module path string

; require 'module'
(call
  method: (identifier) @_method
  (#eq? @_method "require")
  arguments: (argument_list
    (string
      (string_content) @import.path))) @import

; require_relative 'path'
(call
  method: (identifier) @_method
  (#eq? @_method "require_relative")
  arguments: (argument_list
    (string
      (string_content) @import.path))) @import

; load 'path' — like require, but always re-evaluates the file. Same
; string-literal argument shape as require/require_relative.
(call
  method: (identifier) @_method
  (#eq? @_method "load")
  arguments: (argument_list
    (string
      (string_content) @import.path))) @import

; using Module — activates a refinement module in the current scope. Not a
; file-level import, but the closest thing Ruby's refinement system has to
; one: it brings a module's (monkey-patch) behavior into scope, the same
; role include/extend/prepend play for mixins.
(call
  method: (identifier) @_method
  (#eq? @_method "using")
  arguments: (argument_list
    (constant) @import.path)) @import

; include Module
(call
  method: (identifier) @_method
  (#eq? @_method "include")
  arguments: (argument_list
    (constant) @import.path)) @import

; include Namespace::Module — the argument is namespaced, so it parses as
; scope_resolution rather than a bare constant. Extremely common (e.g. Rails
; `include ActiveSupport::Concern`); the bare-constant-only pattern above
; silently dropped every namespaced include/extend/prepend.
(call
  method: (identifier) @_method
  (#eq? @_method "include")
  arguments: (argument_list
    (scope_resolution) @import.path)) @import

; extend Module
(call
  method: (identifier) @_method
  (#eq? @_method "extend")
  arguments: (argument_list
    (constant) @import.path)) @import

; extend Namespace::Module
(call
  method: (identifier) @_method
  (#eq? @_method "extend")
  arguments: (argument_list
    (scope_resolution) @import.path)) @import

; prepend Module
(call
  method: (identifier) @_method
  (#eq? @_method "prepend")
  arguments: (argument_list
    (constant) @import.path)) @import

; prepend Namespace::Module
(call
  method: (identifier) @_method
  (#eq? @_method "prepend")
  arguments: (argument_list
    (scope_resolution) @import.path)) @import
