use std::net::SocketAddr;

use axum::{routing::get, Router};
use log::info;

async fn status_handler() -> &'static str {
    "Ok"
}

pub async fn spawn_web_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new().route("/", get(status_handler));

    // TODO reading the port from configuration
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
