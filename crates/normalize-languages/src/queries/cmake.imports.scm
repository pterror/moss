; CMake imports query
; @import       — the entire include()/find_package()/etc. command (for line number)
; @import.path  — the module/package/subdirectory name (first argument)
;
; CMake command names are case-insensitive by spec (`include(...)`,
; `INCLUDE(...)`, and `Include(...)` all invoke the same builtin command) —
; confirmed via `normalize syntax query` that `(#match? @_cmd
; "^(include|find_package)$")` (case-sensitive) matched 0 times against
; `INCLUDE(SomeModule)`/`Find_Package(Foo REQUIRED)`, while the case
; against real code is common: many CMake style guides use uppercase for
; built-in commands. All command-name predicates below use the `(?i)` flag,
; matching the precedent already set in `cmake.cfg.scm`'s
; break/continue/return patterns.
;
; `add_subdirectory(dir)` and `FetchContent_MakeAvailable(name)` were
; entirely absent — the only import-like commands previously recognized
; were `include`/`find_package`. `add_subdirectory` is CMake's closest
; analog to a module/file import (it pulls another CMakeLists.txt's
; declarations into the current build) and is used in nearly every
; multi-directory CMake project. `FetchContent_MakeAvailable` is the modern
; standard way to pull in an external dependency (superseding
; `find_package` for git-based deps in current CMake style). `find_library`/
; `find_path`/`find_program` are the other `find_package`-family lookup
; commands for external dependencies.
;
; All patterns anchor `.` on the first `argument`: `find_package(Foo
; REQUIRED)`/`find_library(VAR name)` etc. take trailing keyword arguments
; or an output-variable-then-name pair, and an unanchored `(argument)
; @import.path` (as this file previously had) matches EVERY argument —
; confirmed via `normalize syntax query` that `find_package(Foo REQUIRED)`
; captured "REQUIRED" as a second, bogus @import.path alongside "Foo".

; include(SomeModule)
(normal_command
  (identifier) @_cmd
  (argument_list
    .
    (argument) @import.path)
  (#match? @_cmd "(?i)^include$")) @import

; find_package(SomePackage ...)  — path is the first argument
(normal_command
  (identifier) @_cmd
  (argument_list
    .
    (argument) @import.path)
  (#match? @_cmd "(?i)^find_package$")) @import

; find_library(VAR name ...) / find_path(VAR name ...) /
; find_program(VAR name ...) — path is the SECOND argument (the thing being
; searched for), not the output variable name.
(normal_command
  (identifier) @_cmd
  (argument_list
    .
    (argument)
    .
    (argument) @import.path)
  (#match? @_cmd "(?i)^(find_library|find_path|find_program)$")) @import

; add_subdirectory(dir) — pulls another CMakeLists.txt's declarations into
; the current build; the closest CMake has to a module/file import.
(normal_command
  (identifier) @_cmd
  (argument_list
    .
    (argument) @import.path)
  (#match? @_cmd "(?i)^add_subdirectory$")) @import

; FetchContent_MakeAvailable(name ...) — modern standard for pulling in an
; external dependency previously declared via FetchContent_Declare. Each
; argument names an already-declared dependency to make available, so
; (unlike the single-target commands above) every argument is a real path,
; not just the first.
(normal_command
  (identifier) @_cmd
  (argument_list
    (argument) @import.path)
  (#match? @_cmd "(?i)^FetchContent_MakeAvailable$")) @import
