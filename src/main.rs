use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tokio;
use yalc_app::YalcApp;
use yalc_database_mongo::MongoClient;
use yalc_database_redis::RedisClient;
use yalc_database_seaorm::SeaOrmClient;
use yalc_errors::AppError;
use yalc_logger::{setup_logger, LoggerConfig};
use yalc_metrics::setup_metrics;

#[derive(Clone)]
struct AppState {
    pub pg_db: Arc<SeaOrmClient>,
    pub mongo_db: Arc<MongoClient>,
    pub redis_db: Arc<RedisClient>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Load Environment Variables (FORCED)
    dotenv().expect("Failed to load .env file. Please create one with required variables.");

    let otlp_endpoint = env::var("OTLP_ENDPOINT").ok(); // Optional
    let pg_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL must be set in .env");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in .env");

    // 2. Telemetry & Observability
    setup_logger(LoggerConfig {
        service_name: "rust-boilerplate".into(),
        otlp_endpoint,
    })?;
    setup_metrics()?; // Initializes Prometheus exporter

    tracing::info!("Starting Rust Boilerplate with Lego Architecture...");

    // 3. Parallel Database Bootstrap (Fail-Fast)
    tracing::info!("Connecting to primary databases (Postgres, Mongo, Redis)...");
    
    // We use tokio::try_join! to connect to all databases concurrently.
    // If ANY of them fails, the app aborts immediately (Fail-Fast).
    let (pg_client, mongo_client, redis_client) = tokio::try_join!(
        SeaOrmClient::connect(&pg_url),
        MongoClient::connect(&mongo_url, "yalc_db_mongo"),
        RedisClient::connect(&redis_url)
    )?;

    let state = AppState {
        pg_db: Arc::new(pg_client),
        mongo_db: Arc::new(mongo_client),
        redis_db: Arc::new(redis_client),
    };

    // 4. Router Setup
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .with_state(state);

    // 5. App Bootstrap & Graceful Shutdown
    // YalcApp now automatically includes Helmet-like security (timeout, CORS, CatchPanic, HSTS).
    let app = YalcApp::new(router).with_port(3000);
    
    app.start().await?;

    Ok(())
}
