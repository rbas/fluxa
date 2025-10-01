use std::net::SocketAddr;
use std::str::FromStr;
use axum::{routing::get, Router};
use log::{info};

use crate::error::{HttpError};


pub struct WebServer {
    listen_address: String,
}

impl WebServer {
    pub fn new(listen_address: String) -> Self {
        Self { listen_address }
    }

    pub async fn run(self) -> Result<(), HttpError> {
        let app = Router::new().route("/", get(|| async {"OK"}));

        let addr = SocketAddr::from_str(self.listen_address.as_str())?;

        info!("🌐 Web server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        match axum::serve(listener, app.into_make_service()).await {
            Ok(_) => {
                info!("Web server completed gracefully");
                Ok(())
            },
            Err(e) => Err(HttpError::Server {
                message: e.to_string()
            })
        }

    }
}
