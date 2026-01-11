FROM scratch AS realm-builder-amd64
COPY out/x86_64-unknown-linux-musl/realm /realm
COPY out/x86_64-unknown-linux-musl/realm-slim /realm-slim

FROM scratch AS realm-builder-arm64
COPY out/aarch64-unknown-linux-musl/realm /realm
COPY out/aarch64-unknown-linux-musl/realm-slim /realm-slim

FROM alpine:latest

ARG TARGETARCH
ARG BINARY_NAME=realm

COPY --from=realm-builder-${TARGETARCH} /${BINARY_NAME} /usr/bin/realm

ENTRYPOINT [ "/usr/bin/realm" ]
