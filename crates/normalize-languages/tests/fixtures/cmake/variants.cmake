# Completeness-matrix fixture for CMake query files.
# One small, commented construct per node-type / command-family found by
# cross-referencing arborium-cmake 2.17.0's node-types.json against
# cmake.{tags,imports,calls,complexity,cfg}.scm. This grammar declares no
# `fields` at all (like css/lua), so completeness here means auditing every
# distinct command-family and node-type shape, not field variants.
#
# NOTE: this fixture uses the `.cmake` extension, not the canonical
# `CMakeLists.txt` filename real CMake projects always use for their root
# build file. `normalize_languages::support_for_path`/`resolve_language`
# resolve languages purely from `Path::extension()`, with no filename-stem
# fallback for extensionless-or-differently-extensioned canonical names
# (`CMakeLists.txt` has extension "txt", not "cmake"). This means a real
# project's actual `CMakeLists.txt` is currently NEVER routed to the CMake
# grammar by `normalize view`/`normalize structure rebuild`/etc. — verified
# via `normalize syntax query -p CMakeLists.txt ...` returning 0 matches for
# every pattern, including `(source_file)` itself, on this crate's own
# `tests/fixtures/cmake/CMakeLists.txt`. That's a language-registry gap
# (missing filename-based routing, shared across every canonical-filename
# language: Dockerfile, Makefile, CMakeLists.txt, ...), not a `.scm` query
# gap, so it's out of scope for this fixture/test file to fix — recorded
# here so it isn't lost, and reported upstream.

# --- tags.scm: definition forms --------------------------------------------

# function(...) — lowercase, @definition.function
function(plain_function arg1 arg2)
    message(STATUS "hi")
endfunction()

# FUNCTION(...) — uppercase; CMake commands are case-insensitive and the
# grammar assigns the same function_def node type regardless of case
# (verified via `normalize syntax query`; no query fix needed here since
# tags.scm matches structurally on node type, not command-name text).
FUNCTION(upper_function arg)
ENDFUNCTION()

# macro(...) — @definition.function (CMake macros and functions are both
# tagged definition.function; no distinct "macro" SymbolKind exists)
macro(plain_macro arg)
endmacro()

# --- imports.scm: case-insensitivity and command-family coverage ----------

# include() — lowercase
include(LowercaseModule)

# INCLUDE() — uppercase; the original query used a case-sensitive #match?
# and silently matched 0 commands written in uppercase, which is a common
# CMake style-guide convention.
INCLUDE(UppercaseModule)

# find_package() — @import.path is the FIRST argument only; a following
# REQUIRED/QUIET/COMPONENTS keyword argument must not also be captured
# (this was the actual bug: an unanchored (argument) pattern captured every
# argument in the list).
find_package(SomePackage REQUIRED)

# find_library()/find_path()/find_program() — the SECOND argument is the
# thing being searched for; the first is the output variable name and must
# NOT be captured as @import.path.
find_library(FOO_LIB_VAR foo_lib_name)
find_path(FOO_INCLUDE_VAR foo_header.h)
find_program(FOO_PROGRAM_VAR foo_program)

# add_subdirectory() — CMake's closest analog to a module/file import;
# entirely unhandled before this fix.
add_subdirectory(some_subdir)
add_subdirectory(other_subdir EXCLUDE_FROM_ALL)

# FetchContent_MakeAvailable() — modern external-dependency import idiom;
# every argument is a real dependency name (unlike find_package's trailing
# keyword arguments), so all arguments are captured.
FetchContent_MakeAvailable(some_dep other_dep)

# --- complexity.scm: nesting for scope-introducing constructs -------------

# block()...endblock() — CMake 3.25+ scope-isolation construct; matches the
# cross-language convention (rust's impl_item/mod_item, python's
# function_definition/class_definition) of counting scope-introducing
# constructs as @nesting even though they aren't decision points.
block()
    set(scoped_var 1)
endblock()

# --- NEGATIVE: must NOT match ----------------------------------------------

function(negative_check)
    # a plain command call inside a function body is not itself a definition
    message(STATUS "not a definition")
    # find_package's REQUIRED keyword argument must not appear as an
    # @import.path (see the find_package pattern above)
endfunction()
