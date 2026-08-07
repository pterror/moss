; Dockerfile imports query
; @import       — the entire FROM instruction (for line number)
; @import.path  — the base image reference
; @import.alias — the AS stage name

; FROM ubuntu:20.04
; FROM ubuntu:20.04 AS builder
; FROM --platform=linux/amd64 ubuntu:20.04@sha256:... AS builder
; FROM builder AS final                (multi-stage: references an earlier
;                                        stage by name — this grammar gives
;                                        stage references and external image
;                                        references the same `image_spec`
;                                        shape, so no query-level distinction
;                                        is possible or needed; both surface
;                                        uniformly as @import.path)
;
; `as:` is a field on `from_instruction` pointing at `image_alias` — there is
; no intermediate `as_instruction` node in this grammar. It is optional, so a
; single pattern with `as: (image_alias)?` covers both the aliased and
; unaliased forms without emitting two overlapping matches per instruction
; (the previous two-pattern version double-matched every aliased FROM).
;
; `--platform=...` (and any other `--flag=value` prefix on FROM) parses as a
; sibling `param` node with no distinguishing field — see the COPY/ADD note
; below for why that node is not captured here.
(from_instruction
  (image_spec) @import.path
  as: (image_alias)? @import.alias) @import

; NOT CAPTURED: COPY --from=<stage-or-image> / ADD --from=<stage-or-image>
; (multi-stage build references via `--from=`).
;
; `copy_instruction`/`add_instruction` have no fields at all (per
; node-types.json) — every `--flag=value` prefix (`--from=`, `--chown=`,
; `--chmod=`) parses as an undifferentiated `param` node whose own children
; are only the bare `--` and `=` punctuation tokens; the flag name
; ("from"/"chown"/...) and its value are not separate nodes, just
; unstructured text inside `param`'s own byte range (confirmed via
; `normalize syntax query -p <probe> '(copy_instruction (param) @p)'
; --show-source`, which returns the whole `--from=builder` string with no
; way to select just `builder`). A `.scm` capture on `(param)` here would
; therefore either capture nothing useful or leak the literal `--from=`
; prefix into `@import.path`, corrupting every consumer that treats
; `@import.path` as a bare module/stage reference (dependency graphs, etc.).
; `Dockerfile::extract_imports` in `dockerfile.rs` handles this instead
; (text-prefix extraction on an identified `param` node, which is exactly
; "classification requires text, not just node kind" territory) — it only
; participates in the trait-fallback path, which effectively means: reachable
; whenever a caller invokes `Language::extract_imports` directly, but not
; reached by the query-first pipeline (`normalize-deps`/`normalize-facts`)
; once this file's `@import` pattern already matched a `FROM` in the same
; file, since query results take priority over the trait fallback.
; Real Docker semantics only support `--from=` on COPY, not ADD (BuildKit
; does not extend it to ADD either) — extraction is COPY-only.
