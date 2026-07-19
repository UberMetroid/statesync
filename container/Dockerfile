# Stage 1: Build the static Rust application using Musl target
FROM docker.io/clux/muslrust:stable-2025-04-01 as builder
WORKDIR /usr/src/statesync
RUN apt-get update && apt-get install -y --no-install-recommends protobuf-compiler 2>/dev/null || true
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Alpine Linux runtime container (Safe, lightweight, shell and TUI support)
FROM alpine:3.20
RUN apk add --no-cache tzdata wget && \
    addgroup -S -g 1000 statesync && \
    adduser -S -u 1000 -G statesync statesync
WORKDIR /app
COPY --from=builder /usr/src/statesync/target/x86_64-unknown-linux-musl/release/statesync /usr/local/bin/statesync
RUN chown statesync:statesync /usr/local/bin/statesync /app
USER statesync
ENV RUST_LOG=info
ENV TZ=UTC
EXPOSE 8754
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8754/healthz || exit 1
ENTRYPOINT ["/usr/local/bin/statesync"]
