# syntax=docker/dockerfile:1
# Real-world-shaped multi-stage Go build: platform pin, digest pin, ARG
# defaults, multi-name ENV, a stage referencing an earlier stage by name,
# a BuildKit cache mount, and both exec- and shell-form instructions.

ARG GO_VERSION=1.21
ARG BUILD_ENV=production

FROM --platform=$BUILDPLATFORM golang:${GO_VERSION}-alpine AS builder
ARG GO_VERSION
ENV CGO_ENABLED=0 GOOS=linux
WORKDIR /src
COPY go.mod go.sum ./
RUN --mount=type=cache,target=/root/go/pkg/mod go mod download
COPY . .
RUN go build -o /out/app ./cmd/app

FROM builder AS test
RUN go test ./...

FROM gcr.io/distroless/static-debian12@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd AS final
ENV BUILD_ENV=${BUILD_ENV}
COPY --from=builder /out/app /usr/local/bin/app
LABEL org.opencontainers.image.source="https://example.com/app"
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/app"]
CMD ["--config", "/etc/app/config.yaml"]
