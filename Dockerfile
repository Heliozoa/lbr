# installs cargo-chef for the planner and builder
FROM rust:bookworm AS chef
COPY ./rust-toolchain.toml /lbr/rust-toolchain.toml
WORKDIR /lbr

RUN cargo install cargo-chef


# creates recipe.json for the builder
FROM chef AS planner
COPY ./rust-toolchain.toml /lbr/rust-toolchain.toml

COPY ./Cargo.toml /lbr/Cargo.toml
COPY ./crates /lbr/crates
RUN cargo chef prepare --recipe-path recipe.json


# builds the application
FROM chef AS builder
COPY ./rust-toolchain.toml /lbr/rust-toolchain.toml

# set up for chef and leptos
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos
COPY --from=planner /lbr/recipe.json recipe.json
ADD https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css /lbr/style/css/bulma.css
ARG RELEASE=""

# cook and build
RUN cargo chef cook $RELEASE --recipe-path recipe.json
COPY ./Cargo.toml /lbr/Cargo.toml
COPY ./crates /lbr/crates
RUN cargo leptos build $RELEASE


# sets up the env and entrypoint for the application
FROM debian:bookworm-slim AS runtime
WORKDIR /lbr

# install deps
RUN apt update -y
RUN apt install -y libpq5
COPY ./data/ichiran-cli /lbr/ichiran-cli

# set up env
ENV RUST_LOG                debug
ENV SERVER_URL              0.0.0.0:3000
ENV DATABASE_URL            postgres://lbr:lbr@host.docker.internal/lbr
ENV ICHIRAN_DATABASE_URL    postgres://lbr:lbr@host.docker.internal/ichiran
ENV ICHIRAN_CLI_PATH        /lbr/ichiran-cli
ENV PRIVATE_COOKIE_PASSWORD abcdefghijklmnopqrstuvwxyzåäöABCDEFGHIJKLMNOPQRSTUVWXYZÅÄÖ
ENV LEPTOS_OUTPUT_NAME      lbr
ENV LEPTOS_SITE_ROOT        site
ENV LEPTOS_SITE_PKG_DIR     pkg
ENV LEPTOS_SITE_ADDR        127.0.0.1:3000
ENV LEPTOS_RELOAD_PORT      3001

# server entrypoint
COPY --from=builder /lbr/target/server/*/lbr_server /lbr/lbr_server
COPY --from=builder /lbr/target/site /lbr/site
ENTRYPOINT ["/lbr/lbr_server"]
