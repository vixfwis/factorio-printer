FROM rust:1.69-bullseye as rust-build-stage
RUN apt-get update && apt-get upgrade -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.lock Cargo.toml ./
COPY src src/
RUN cargo build --release

FROM gcr.io/distroless/cc@sha256:f81e5db8287d66b012d874a6f7fea8da5b96d9cc509aa5a9b5d095a604d4bca1
USER nobody

WORKDIR /app
COPY --from=rust-build-stage --chown=nobody:nogroup /build/target/release/factorio-printer .

ENTRYPOINT ["/app/factorio-printer"]
