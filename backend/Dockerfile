# syntax=docker/dockerfile:1.7

FROM rust:1.99.1-slim-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
# Registry, git and target caches persist between builds, so only changed crates recompile.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p loglens-api \
    && install -m 0755 target/release/loglens-api /usr/local/bin/loglens-api

FROM debian:bookworm-20261104-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 loglens \
    && useradd --system --uid 10001 --gid loglens --home-dir /data --shell /usr/sbin/nologin loglens \
    && install -d -o loglens -g loglens -m 0750 /data
COPY --from=build /usr/local/bin/loglens-api /usr/local/bin/loglens-api
ENV LOGLENS_BIND=0.0.0.0:8080 \
    LOGLENS_DB_URL=sqlite:///data/loglens.db \
    RUST_LOG=info
USER loglens
WORKDIR /data
VOLUME ["/data"]
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/8080 && printf 'GET /api/health HTTP/1.0\r\nHost: localhost\r\n\r\n' >&3 && grep -q '\"ok\"' <&3"]
ENTRYPOINT ["/usr/local/bin/loglens-api"]
