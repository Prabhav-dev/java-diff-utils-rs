FROM rust:1-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches
COPY tests ./tests

RUN cargo test --all --quiet
RUN cargo build --release --bin java-diff-utils-rs

FROM debian:bookworm-slim AS runtime

RUN useradd --create-home --uid 10001 appuser
COPY --from=builder --chown=appuser:appuser \
    /app/target/release/java-diff-utils-rs \
    /usr/local/bin/java-diff-utils-rs

USER appuser
ENTRYPOINT ["/usr/local/bin/java-diff-utils-rs"]
