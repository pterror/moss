@0x1234567890abcdef;

# Completeness matrix for capnp.imports.scm / capnp.decorations.scm.
#
# capnp.imports.scm handles exactly one grammar shape for real imports:
# `using_directive > import_using`, with children `(type_identifier)
# (import_path)` in that order. The node-types.json-listed bare `import`
# node type is never actually produced by the parser (verified via
# `normalize syntax ast`) and is intentionally NOT queried.

# POSITIVE: import_using — basic form.
using Cxx = import "/capnp/c++.capnp";

# POSITIVE: import_using — a second import, different alias/path, to make
# sure the query isn't accidentally singleton-anchored to the first match.
using Other = import "other/thing.capnp";

# NEGATIVE: replace_using — `using X = SomeType;` aliases a type, not an
# import. It shares the `using` keyword and the `using_directive` wrapper
# with import_using, but its child is `type_identifier`/`generics`/
# `type_definition`, never `import_path`. Must NOT contribute an
# @import/@import.path capture.
using MyAlias = UInt32;

# A struct exercising the aliased type above, so MyAlias isn't dead code
# (keeps the fixture realistic / avoids an unused-alias warning if capnp
# ever grows one).
struct UsesAlias {
  x @0 :MyAlias;
}
