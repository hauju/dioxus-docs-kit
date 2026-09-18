# syntax=docker/dockerfile:1.7
# Build the wasm client and the fullstack server, then ship only the server.
ARG RUST_VERSION=1.96.0
ARG DX_VERSION=0.7.10

FROM rust:${RUST_VERSION}-slim-bookworm AS builder

ENV CARGO_TERM_COLOR=never \
    DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl clang pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install dioxus-cli --version "$DX_VERSION" --locked

WORKDIR /app

COPY --link Cargo.toml Cargo.lock ./
COPY --link crates ./crates
COPY --link src ./src
COPY --from=assets /dist ./assets

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    dx bundle --web --release --debug-symbols false

FROM debian:bookworm-slim AS runtime
LABEL org.opencontainers.image.source="https://github.com/hauju/dioxus-docs-kit"

RUN useradd --create-home --uid 10001 app
USER app
WORKDIR /srv

COPY --from=builder --chown=app:app /app/target/dx/web/release/web /srv

EXPOSE 8080
ENV PORT=8080 IP=0.0.0.0
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
    CMD ["/srv/server", "--health"]

ENTRYPOINT ["/srv/server"]
CMD []
