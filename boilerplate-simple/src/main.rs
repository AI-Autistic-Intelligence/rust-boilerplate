use axum::{routing::get, Router};
use std::env;
use dotenvy::dotenv;

use ferrox_app::FerroxApp;
use ferrox_transports::http::HttpTransport;
use ferrox_logger::{setup_logger, LoggerConfig};
use ferrox_errors::AppError;

async fn health_check() -> &'static str {
    "OK - Ferrox Simple Boilerplate"
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Load Environment Variables
    let _ = dotenv(); // Ignore if .env is missing

    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap();

    // 2. Telemetry & Observability
    let _sentry_guard = setup_logger(LoggerConfig {
        service_name: "ferrox-boilerplate-simple".into(),
        otlp_endpoint: None,
        sentry_dsn: None,
    })?;

    tracing::info!("Starting Ferrox Simple Boilerplate...");

    // 3. Router Setup
    let router = Router::new()
        .route("/health", get(health_check));

    // 4. HTTP Transport
    let http_transport = HttpTransport::new(router, port);

    // 5. App Bootstrap & Graceful Shutdown
    let app = FerroxApp::new().add_transport(http_transport);
    
    app.start().await?;

    Ok(())
}
