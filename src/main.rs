use axum::{routing::get, Router};
use std::sync::Arc;
use tokio;
use yalc_app::YalcApp;
use yalc_database_seaorm::SeaOrmClient;
use yalc_errors::AppError;
use yalc_logger::setup as setup_logger;

#[derive(Clone)]
struct AppState {
    db: Arc<SeaOrmClient>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    println!("Starting Rust Boilerplate with Lego Architecture...");
    
    // 1. Logger Setup
    setup_logger(); // Assuming logger still has a basic setup fn, or tracing is used.

    // 2. Database Connection (SeaORM)
    println!("Connecting to primary database...");
    // Mock URL for now. In a real app, load from config.
    let db_client = SeaOrmClient::connect("postgres://postgres:password@localhost/yalc_db")
        .await
        .unwrap_or_else(|_| {
            println!("Warning: Could not connect to DB. Ensure Postgres is running.");
            // Returning a dummy client or panicking in a real scenario
            // For boilerplate demonstration, we will just panic if required, but let's handle it gracefully for the demo.
            SeaOrmClient { db: sea_orm::DatabaseConnection::Disconnected }
        });

    let state = AppState {
        db: Arc::new(db_client),
    };

    // 3. Router Setup
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .with_state(state);

    // 4. App Bootstrap & Graceful Shutdown
    let app = YalcApp::new(router).with_port(3000);
    
    app.start().await?;

    Ok(())
}
