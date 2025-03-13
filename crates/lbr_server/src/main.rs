//! Web backend for LBR.

use eyre::WrapErr;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let server_url = env::var("SERVER_URL")
        .wrap_err("Missing SERVER_URL")?
        .parse::<SocketAddr>()
        .wrap_err("Invalid SERVER_URL")?;

    let lbr_database_url = env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let ichiran_database_url =
        env::var("ICHIRAN_DATABASE_URL").wrap_err("Missing ICHIRAN_DATABASE_URL")?;
    let ichiran_cli_path = env::var("ICHIRAN_CLI_PATH").wrap_err("Missing ICHIRAN_CLI_PATH")?;
    let private_cookie_password =
        env::var("PRIVATE_COOKIE_PASSWORD").wrap_err("Missing PRIVATE_COOKIE_PASSWORD")?;

    let router = lbr_server::router_from_vars(
        &lbr_database_url,
        &ichiran_database_url,
        ichiran_cli_path.into(),
        &private_cookie_password,
    )
    .await
    .wrap_err("Failed to build router")?;

    tracing::info!("Starting server at {server_url}");
    let server_addr = TcpListener::bind(server_url)
        .await
        .wrap_err("Failed to bind to address")?;
    axum::serve(server_addr, router.into_make_service())
        .await
        .wrap_err("Failed to start server")?;
    Ok(())
}
