FROM docker.io/oven/bun:1.4.2 AS frontend
WORKDIR /src/frontend_lio
COPY frontend_lio/package.json frontend_lio/bun.lock ./
RUN bun install --frozen-lockfile
COPY frontend_lio/ ./
RUN bun run build

FROM docker.io/library/rust:1.98.1-bookworm AS backend
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY app/ app/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked \
    && cp target/release/lio /lio

FROM docker.io/library/debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/assets /data \
    && chown 10001:10001 /data

WORKDIR /app
COPY --from=backend /lio /app/lio
COPY --from=frontend /src/frontend_lio/dist/ /app/public/

ENV RUST_LOG=info,sqlx=warn \
    LIO__FRONTEND_DIR=/app/public \
    LIO__DATABASE_URL=sqlite:///data/lio.db?mode=rwc

VOLUME /data
EXPOSE 3001
USER 10001:10001
ENTRYPOINT ["/app/lio"]
CMD ["server"]
