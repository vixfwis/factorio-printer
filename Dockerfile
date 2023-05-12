FROM rust:1.68.1 as rust-build-stage
RUN apt-get update && apt-get upgrade -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.lock Cargo.toml ./
COPY src src/
RUN cargo build --release

FROM rust-build-stage as rust-run-stage
RUN addgroup --system rustapp \
    && adduser --system --ingroup rustapp rustapp
USER rustapp

WORKDIR /app
COPY --from=rust-build-stage --chown=rustapp:rustapp /build/target/release/factorio-printer .

ENTRYPOINT ["/app/factorio-printer"]
