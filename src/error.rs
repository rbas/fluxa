use thiserror::Error;

use crate::{model::MonitoredServiceError, settings::ServiceConfigurationError};

/// Top-level application errors
#[derive(Debug, Error)]
pub enum FluxaError {
    #[error("Configuration error: {0}")]
    Configuration(#[from] ServiceConfigurationError),

    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    #[error("HTTP server error: {0}")]
    Http(#[from] HttpError),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Address parsing error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
}

/// Service monitoring and operation errors
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Monitored service error: {0}")]
    MonitoredService(#[from] MonitoredServiceError),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Notification error: {0}")]
    Notification(#[from] NotificationError),
}

/// HTTP server errors
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Address parsing error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("TCP binding error: {0}")]
    TcpBind(#[from] std::io::Error),

    #[error("Server error: {message}")]
    Server { message: String },
}

/// Notification service errors
#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Failed to send notification: {message}")]
    SendFailed { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MonitoredServiceError;
    use crate::settings::ServiceConfigurationError;

    #[test]
    fn test_fluxa_error_from_configuration_error() {
        let config_error =
            ServiceConfigurationError::ErrorInConfiguration("test error".to_string());
        let fluxa_error = FluxaError::from(config_error);

        assert!(matches!(fluxa_error, FluxaError::Configuration(_)));
        assert_eq!(
            fluxa_error.to_string(),
            "Configuration error: Configuration error test error"
        );
    }

    #[test]
    fn test_service_error_from_monitored_service_error() {
        let monitored_error = MonitoredServiceError::InvalidUrl("invalid".to_string());
        let service_error = ServiceError::from(monitored_error);

        assert!(matches!(service_error, ServiceError::MonitoredService(_)));
        assert_eq!(
            service_error.to_string(),
            "Monitored service error: invalid is not valid url"
        );
    }

    #[test]
    fn test_notification_error_send_failed() {
        let notification_error = NotificationError::SendFailed {
            message: "server error".to_string(),
        };

        assert_eq!(
            notification_error.to_string(),
            "Failed to send notification: server error"
        );
    }

    #[test]
    fn test_http_error_addr_parse() {
        let addr_parse_error = "invalid:address:format"
            .parse::<std::net::SocketAddr>()
            .unwrap_err();
        let http_error = HttpError::from(addr_parse_error);

        assert!(matches!(http_error, HttpError::AddrParse(_)));
    }
}
