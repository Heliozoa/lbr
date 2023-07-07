//! Web backend for LBR.

use axum::Server;
use eyre::WrapErr;
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let server_url = env::var("SERVER_URL")
        .wrap_err("Missing SERVER_URL")?
        .parse()
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
    Server::bind(&server_url)
        .serve(router.into_make_service())
        .await
        .wrap_err("Failed to start server")?;
    Ok(())
}
