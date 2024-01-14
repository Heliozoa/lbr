Thanks for your interest in contributing to LBR! Issues, pull requests and discussion threads are welcome.


## Project structure
The project is split into a few crates in the `./crates` directory:

### `lbr_api` (`./crates/api`)
Contains types for communicating between the backend and frontend.

### `lbr_server` (`./crates/backend`)
A web server offering the functionality of LBR in its endpoints.

Uses [axum](https://docs.rs/axum) for the web server and [diesel](https://docs.rs/diesel) with postgres for the database.

### `lbr_core` (`./crates/core`)
Contains some core types shared by the main library and frontends.

### `lbr_frontend` (`./crates/frontend`)
A thin wrapper around `lbr_web`.

### `lbr_web` (`./crates/web`)
A web frontend that works with the backend.

Uses the [Leptos](https://docs.rs/leptos) web framework and [Bulma](https://bulma.io/) for the styling.

### `lbr` (`./crates/lbr`)
The main library of the project that offers all the core functionality.


## Setting up LBR locally
### Prerequisites
- Rust: https://www.rust-lang.org/tools/install
- cargo-about: https://crates.io/crates/cargo-about
- Postgres: https://www.postgresql.org/
- Common Lisp (SBCL): http://www.sbcl.org/
- cargo-leptos (optional): https://crates.io/crates/cargo-leptos
- Docker (optional): https://www.docker.com/
- just (optional): https://crates.io/crates/just
- diesel_cli (optional): https://crates.io/crates/diesel_cli (`cargo install diesel_cli --no-default-features --features postgres`)
- jq (optional): https://jqlang.github.io/jq/
- Locale `ja_JP.utf8` (optional)

### justfile
The `justfile` in the repository root contains convenient commands for setting everything up that can be ran with `just`. You can also follow them along manually and run the commands in your shell.

Running `just prepare-repository` will prepare the repository by
- copying the `example.env` to `.env`
- setting up quicklisp and ichiran in `./data`
- creating a `lbr` postgres user
- creating the `lbr` and `ichiran` databases
- downloading and generating various files related to Japanese words/kanji
- generating the license.html
- building the ichiran-cli
If something goes wrong, rerunning the command is safe though it may do unnecessary extra work. You can also check the `justfile` and execute the individual steps.

After setup is finished, you can start the dev server with `just watch` (or `cargo leptos watch`).


## Development
LBR uses the nightly toolchain.

### Logging
Setting the logging level for the backend is done with the `RUST_LOG` environment variable. For the frontend, the `WASM_LOG` environment variable is used. The levels available are the usual `trace`, `debug`, `info`, `warn` and `error`.

### Formatting
- Rust: `cargo fmt`

- TOML with [Taplo](https://taplo.tamasfe.dev/): `taplo fmt`

### Linting
`cargo clippy`

### Running the project
#### Without Docker
Install `cargo leptos` with `cargo install cargo-leptos`

Run `cargo leptos watch`

LBR will be available at `http://0.0.0.0:3000`.

#### With Docker
Build the image with `just docker-build`

Run the image with `just docker-run`

LBR will be available at `http://0.0.0.0:3000`. The container will use your localhost `lbr` and `ichiran` databases.


## Deployment
### Using Docker
A Docker image is available at https://hub.docker.com/repository/docker/heliozoagh/lbr/general. The image requires a connection to both the `lbr` and `ichiran` databases, configured with the environment variables `DATABASE_URL`, `ICHIRAN_DATABASE_URL`, `ICHIRAN_CONNECTION` and `PRIVATE_COOKIE_PASSWORD`. This image can be deployed at https://render.com/ etc.

### Databases

To set up the databases at some remote host, you can set them up locally and then copy them over with
```bash
pg_dump --no-owner --dbname=postgres://lbr:lbr@localhost/ichiran | psql <ichiran-connection-string>
pg_dump --no-owner --dbname=postgres://lbr:lbr@localhost/lbr | psql <lbr-connection-string>
```
where the connection strings are databases at something like https://neon.tech/ for example.
