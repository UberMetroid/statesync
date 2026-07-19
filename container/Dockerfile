# Stage 1: Build the static Rust application using Musl target
FROM docker.io/clux/muslrust:stable AS builder
WORKDIR /usr/src/statesync
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Alpine Linux runtime container.
#
# Permission model for the persistent /config volume:
#   - We create /config at build time, owned by the unprivileged
#     'statesync' user (UID/GID 1000). If you mount a fresh Docker
#     volume, it inherits this ownership and the daemon can write
#     to it immediately.
#   - If you bind-mount a host directory (e.g. -v /srv/statesync:/config),
#     the host content takes over with whatever ownership the host
#     had. The entrypoint below runs as root for a single instruction
#     to 'chown -R statesync:statesync /config' (best-effort), then
#     exec's the daemon as the unprivileged user.
#   - The daemon itself is resilient: if /config is still unwritable
#     after that (read-only mount, ACL, etc.), it falls back to
#     /app/config.json (ephemeral) and then to an in-memory default,
#     each with a clear warning logged.
#   - No user action is ever required to start the daemon.
FROM alpine:3.20
RUN apk add --no-cache tzdata wget && \
    addgroup -S -g 1000 statesync && \
    adduser -S -u 1000 -G statesync statesync && \
    mkdir -p /config /app && \
    chown -R statesync:statesync /config /app
WORKDIR /app
COPY --from=builder /usr/src/statesync/target/x86_64-unknown-linux-musl/release/statesync /usr/local/bin/statesync
COPY container/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chown -R statesync:statesync /usr/local/bin && \
    chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/statesync
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
USER statesync
ENV RUST_LOG=info
ENV TZ=UTC
EXPOSE 8754
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8754/healthz || exit 1