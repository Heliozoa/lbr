FROM rust:bookworm AS chef

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY ./rust-toolchain.toml /app/rust-toolchain.toml
COPY ./Cargo.toml /app/Cargo.toml
COPY ./crates /app/crates

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG RELEASE=""

COPY --from=planner /app/recipe.json recipe.json
COPY ./rust-toolchain.toml /app/rust-toolchain.toml
RUN cargo chef cook $RELEASE --recipe-path recipe.json

COPY ./Cargo.toml /app/Cargo.toml
COPY ./crates /app/crates
RUN cargo build $RELEASE --bin lbr_server

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt update -y && \
    apt install -y libpq5

COPY --from=builder /app/target/debug/lbr_server /usr/local/bin/lbr_server
COPY ./data/ichiran-cli /usr/local/bin/ichiran-cli

ENV SERVER_URL 0.0.0.0:3000
ENV DATABASE_URL postgres://lbr:lbr@host.docker.internal/lbr
ENV ICHIRAN_DATABASE_URL postgres://lbr:lbr@host.docker.internal/ichiran
ENV ICHIRAN_CLI_PATH /usr/local/bin/ichiran-cli
ENV PRIVATE_COOKIE_PASSWORD=abcdefghijklmnopqrstuvwxyzåäöABCDEFGHIJKLMNOPQRSTUVWXYZÅÄÖ

ENTRYPOINT ["/usr/local/bin/lbr_server"]
