# SPDX-License-Identifier: GPL-3.0-only
FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p crewsync-server

FROM debian:bookworm-slim
LABEL org.opencontainers.image.source="https://github.com/goncalvesl/crewsync" \
      org.opencontainers.image.licenses="GPL-3.0-only" \
      org.opencontainers.image.title="CrewSync Server"
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -o 10001 -g 10001 -m 0700 /etc/crewsync /var/lib/crewsync
COPY --from=builder /src/target/release/crewsync-server /usr/local/bin/crewsync-server
COPY docker/entrypoint.sh /usr/local/bin/crewsync-entrypoint
RUN chmod 0755 /usr/local/bin/crewsync-entrypoint
ENV HOME=/tmp
USER 10001:10001
WORKDIR /var/lib/crewsync
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --silent --show-error --max-time 3 http://127.0.0.1:8080/ || exit 1
ENTRYPOINT ["/usr/local/bin/crewsync-entrypoint"]
CMD ["serve-http", "/etc/crewsync/server.toml", "0.0.0.0", "8080"]
