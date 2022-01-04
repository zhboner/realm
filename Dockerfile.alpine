FROM alpine:latest

COPY target/x86_64-unknown-linux-musl/release/realm /usr/bin
RUN chmod +x /usr/bin/realm

ENTRYPOINT ["/usr/bin/realm"]