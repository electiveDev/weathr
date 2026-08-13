# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.94
ARG DEBIAN_RELEASE=trixie

FROM rust:${RUST_VERSION}-${DEBIAN_RELEASE} AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && printf 'fn main() {}\n' > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release && rm -rf src

COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    mkdir -p /out \
    && cp target/release/weathr /out/weathr \
    && cp --parents /lib64/ld-linux-x86-64.so.2 /out \
    && ldd target/release/weathr | awk '/=> \/|^\// { print $(NF-1) }' | sort -u | xargs -r -I '{}' cp --parents '{}' /out

FROM debian:${DEBIAN_RELEASE}-slim AS runtime-assets

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

FROM scratch

LABEL org.opencontainers.image.source="https://github.com/electiveDev/weathr"
LABEL org.opencontainers.image.description="Terminal-based ASCII weather application"

ENV HOME=/

COPY --from=builder /out/ /
COPY --from=runtime-assets /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=runtime-assets /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=runtime-assets /etc/localtime /etc/localtime

WORKDIR /
ENTRYPOINT ["/weathr"]
