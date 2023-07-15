//! Web backend for LBR.

use axum::Server;
use eyre::WrapErr;
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let server_url = "0.0.0.0:3000".parse().unwrap();
    let private_cookie_password: String =
        "IedeiheeRahnaizohm0aik1dieBoog6Fee3chuo4IumeepeedaiP2mahk0aa2ieF".to_string();

    let router = lbr_server::router_from_vars(&private_cookie_password)
        .await
        .unwrap();

    tracing::info!("Starting server at {server_url}");
    Server::bind(&server_url)
        .serve(router.into_make_service())
        .await
        .wrap_err("Failed to start server")?;
    Ok(())
}
