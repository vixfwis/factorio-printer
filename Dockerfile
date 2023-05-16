FROM rust:1.69-bullseye as rust-build-stage
RUN apt-get update && apt-get upgrade -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.lock Cargo.toml ./
COPY src src/
RUN cargo build --release

FROM gcr.io/distroless/cc-debian10
USER nobody

WORKDIR /app
COPY --from=rust-build-stage --chown=nobody:nogroup /build/target/release/factorio-printer .

ENTRYPOINT ["/app/factorio-printer"]
