use std::net::SocketAddr;

use axum::{routing::get, Router};
use log::info;

async fn status_handler() -> &'static str {
    "Ok"
}

pub async fn spawn_web_server(
    socket_addr: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new().route("/", get(status_handler));

    let addr: SocketAddr = socket_addr.parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
