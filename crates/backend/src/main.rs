//! Web backend for LBR.

use axum::Server;
use eyre::WrapErr;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let server_url = "0.0.0.0:3000".parse().unwrap();

    let router = lbr_server::router().await;

    tracing::info!("Starting server at {server_url}");
    Server::bind(&server_url)
        .serve(router.into_make_service())
        .await
        .wrap_err("Failed to start server")?;
    Ok(())
}
