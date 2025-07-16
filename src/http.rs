use std::net::SocketAddr;

use axum::{routing::get, Router};
use log::info;

use crate::error::HttpError;

async fn status_handler() -> &'static str {
    "Ok"
}

pub async fn spawn_web_server(
    socket_addr: &str,
) -> Result<(), HttpError> {
    let app = Router::new().route("/", get(status_handler));

    let addr: SocketAddr = socket_addr.parse()?;
    info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let result = axum::serve(listener, app.into_make_service()).await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(HttpError::Server { 
            message: e.to_string() 
        }),
    }
}
