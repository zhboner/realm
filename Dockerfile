ARG TARGETARCH

FROM scratch AS realm-builder-amd64
COPY out/x86_64-unknown-linux-musl/realm /realm
COPY out/x86_64-unknown-linux-musl/realm-slim /realm-slim

FROM scratch AS realm-builder-arm64
COPY out/aarch64-unknown-linux-musl/realm /realm
COPY out/aarch64-unknown-linux-musl/realm-slim /realm-slim

FROM realm-builder-${TARGETARCH} AS realm-builder

FROM alpine:latest

ARG BINARY_NAME=realm
COPY --from=realm-builder /${BINARY_NAME} /usr/bin/realm

ENTRYPOINT [ "/usr/bin/realm" ]
