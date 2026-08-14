; MATLAB tags query

; Function definitions
(function_definition
  name: (identifier) @name) @definition.function

; Abstract method declaration: `methods (Abstract) ... end` blocks in a
; classdef contain a *signature only* (no body). This is a structurally
; distinct node type from `function_definition` — verified via
; arborium-matlab 2.17.0 node-types.json (`function_signature` has its own
; `name: (identifier)` field) and a real parse of
; `methods (Abstract)\n  result = computeArea(obj)\nend` — which the
; original query missed entirely, making every abstract method in
; interface-style classdef code invisible to tags extraction.
(function_signature
  name: (identifier) @name) @definition.function

; Class definitions
(class_definition
  name: (identifier) @name) @definition.class
