# Completeness matrix for Dockerfile query files (tags.scm/imports.scm).
# One small, commented construct per node-type/field variant exercised by
# the queries, plus a NEGATIVE section of near-miss constructs that must
# NOT match. See docs/query-testing-methodology.md.

# === imports.scm: @import / @import.path / @import.alias variants ===

# @import.path only — bare image:tag, no alias
FROM ubuntu:20.04

# @import.path with an image digest (image_spec.digest field)
FROM ubuntu:20.04@sha256:abc123

# @import.path with no tag at all (image_spec.tag is optional)
FROM ubuntu AS bare_name

# @import.path + @import.alias — the common `AS` stage-naming form
FROM golang:1.21-alpine AS builder

# @import.path + @import.alias with a leading `--platform=` param sibling —
# regression fixture for the FROM `--platform=` flag; the flag itself is
# NOT captured (see imports.scm's comment on why `param` can't be
# field-selected), only image_spec/image_alias are.
FROM --platform=linux/amd64 golang:1.21 AS platform_stage

# @import.path referencing an EARLIER STAGE BY NAME, not an external image —
# multi-stage builds. Structurally identical to any other FROM (image_spec
# has no way to distinguish "this text is a prior stage name" from "this
# text is a real registry image"), so it surfaces the same way.
FROM builder AS from_stage_ref

# === tags.scm: @definition.module (build stage names) ===
# Already exercised above via every `AS <alias>` — `@name`/kind `image_alias`.

# === tags.scm: @definition.constant (ARG) variants ===

# ARG name: (unquoted_string), default: (unquoted_string) — regression
# fixture for the field-anchoring bug: an unconstrained query captured
# both "VERSION" (the name) and "1.0" (the default) as spurious symbols.
ARG VERSION=1.0

# ARG default: (double_quoted_string) — default's text must not appear as
# a second @name capture either.
ARG NAME="quoted"

# ARG default: (single_quoted_string)
ARG OTHER='single'

# ARG with no default at all (default field is optional)
ARG NOEQ

# === tags.scm: @definition.constant (ENV) variants ===

# ENV with multiple `KEY=value` pairs on one instruction — one env_pair per
# name, each with a name: (unquoted_string) field. Values ("val1"/"val2")
# must not appear as spurious @name captures (same field-anchoring bug as
# ARG, doubled since ENV values are unquoted_string too).
ENV KEY1=val1 KEY2=val2

# ENV legacy single-pair form without `=` (`ENV name value`) — env_pair.name
# is still unquoted_string; the value is unquoted_string too and must not
# double-match.
ENV LEGACY val3

# === dockerfile.rs Language::extract_imports: COPY --from= variants ===
# (trait-level extraction — see dockerfile.rs's doc comment on why this
# can't live in imports.scm as a field-constrained pattern)

# COPY --from=<stage-name> — references an earlier named stage
COPY --from=builder /a /b

# COPY --from=<index> — references an earlier stage by its 0-based index
COPY --from=0 /a /b

# === NEGATIVE: constructs that must NOT match ===

# COPY --chown= is a `param` too, but must NOT be picked up as a --from
# reference by dockerfile.rs's prefix match.
COPY --chown=user:group /a /b

# Bare COPY (no --from=, no --chown=) has no param children at all — must
# produce zero imports.
COPY /a /b

# ADD --from is not real Docker syntax (--from is COPY-only, even under
# BuildKit) — even though the grammar parses `param` here just like COPY
# (add_instruction has the same undifferentiated param/path/heredoc_block
# children, per node-types.json), dockerfile.rs intentionally does not
# extract imports from add_instruction at all.
ADD --chown=user:group /a /b

# RUN/CMD/ENTRYPOINT/etc. shell-form and exec-form variants — none of these
# instruction kinds are in tags.scm's documented_unused list turned into
# @definition/@reference captures; they must produce zero tags/imports
# matches.
RUN echo shell form
RUN ["echo", "exec form"]
RUN --mount=type=cache,target=/x echo hi
CMD ["echo", "hi"]
CMD echo hi
ENTRYPOINT ["run"]
ENTRYPOINT run
LABEL key="value"
LABEL a="1" b="2"
EXPOSE 8080
EXPOSE 8080/tcp
USER app
USER app:group
VOLUME ["/data"]
VOLUME /data
WORKDIR /app
STOPSIGNAL SIGTERM
ONBUILD RUN echo build
HEALTHCHECK CMD curl -f http://localhost/ || exit 1
HEALTHCHECK NONE
MAINTAINER someone
SHELL ["/bin/bash", "-c"]
