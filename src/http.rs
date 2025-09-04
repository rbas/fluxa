use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    routing::get,
    Router,
};
use log::info;
use serde_json::{json, Value};

use crate::{
    error::HttpError,
    model::{MonitoredService, ServiceInfo},
    state::{get_all_service_states, get_service_state, SharedMonitoringState},
};

async fn status_handler() -> &'static str {
    "Ok"
}

async fn get_all_services_handler(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let all_stats = get_all_service_states(&state.monitoring_state).await;
    let services: Vec<ServiceInfo> = state
        .services
        .iter()
        .filter_map(|service| {
            all_stats
                .get(&service.url)
                .map(|stats| service.to_service_info(stats.clone()))
        })
        .collect();

    Ok(Json(json!({
        "services": services,
        "total_count": services.len()
    })))
}

async fn get_service_handler(
    Path(service_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ServiceInfo>, StatusCode> {
    // Find service by URL or index
    let service = if let Ok(index) = service_id.parse::<usize>() {
        state.services.get(index)
    } else {
        state.services.iter().find(|s| s.url == service_id)
    };

    let service = service.ok_or(StatusCode::NOT_FOUND)?;
    let stats = get_service_state(&state.monitoring_state, &service.url)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(service.to_service_info(stats)))
}

#[derive(Clone)]
pub struct AppState {
    pub monitoring_state: SharedMonitoringState,
    pub services: Vec<MonitoredService>,
}

pub async fn spawn_web_server(
    socket_addr: &str,
    app_state: AppState,
) -> Result<(), HttpError> {
    let app = Router::new()
        .route("/", get(status_handler))
        .route("/api/services", get(get_all_services_handler))
        .route("/api/services/{service_id}", get(get_service_handler))
        .with_state(app_state);

    let addr: SocketAddr = socket_addr.parse()?;
    info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let result = axum::serve(listener, app.into_make_service()).await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(HttpError::Server {
            message: e.to_string(),
        }),
    }
}

#[cfg(unix)]
pub async fn spawn_unix_socket_server(
    socket_path: &str,
    app_state: AppState,
) -> Result<(), HttpError> {
    use std::path::Path;
    use tokio::net::UnixListener;

    let app = Router::new()
        .route("/", get(status_handler))
        .route("/api/services", get(get_all_services_handler))
        .route("/api/services/{service_id}", get(get_service_handler))
        .with_state(app_state);

    // Remove existing socket file if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)
            .map_err(|e| HttpError::Server {
                message: format!("Failed to remove existing socket file: {}", e),
            })?;
    }

    info!("Listening on Unix socket: {}", socket_path);
    let listener = UnixListener::bind(socket_path)
        .map_err(|e| HttpError::Server {
            message: format!("Failed to bind to Unix socket: {}", e),
        })?;

    let result = axum::serve(listener, app.into_make_service()).await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(HttpError::Server {
            message: e.to_string(),
        }),
    }
}
