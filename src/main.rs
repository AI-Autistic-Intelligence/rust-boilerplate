use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tokio;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use yalc_app::YalcApp;
use yalc_database_mongo::MongoClient;
use yalc_database_redis::RedisClient;
use yalc_database_seaorm::SeaOrmClient;
use yalc_errors::AppError;
use yalc_logger::{setup_logger, LoggerConfig};
use yalc_notifications_slack::SlackClient;
use yalc_payments_stripe::StripeClient;
use yalc_storage::S3StorageClient;
use yalc_mailer::MailerClient;
use yalc_feature_flags::FeatureFlagsClient;
use yalc_rate_limiter::RateLimiter;
use yalc_jobs::start_worker;

#[derive(OpenApi)]
#[openapi(
    paths(health_check),
    tags(
        (name = "system", description = "System and health endpoints")
    )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    )
)]
async fn health_check() -> &'static str {
    "OK"
}

#[derive(Clone)]
struct AppState {
    pub pg_db: Arc<SeaOrmClient>,
    pub mongo_db: Arc<MongoClient>,
    pub redis_db: Arc<RedisClient>,
    pub slack: Arc<SlackClient>,
    pub stripe: Arc<StripeClient>,
    pub storage: Arc<S3StorageClient>,
    pub mailer: Arc<MailerClient>,
    pub feature_flags: Arc<FeatureFlagsClient>,
    pub rate_limiter: Arc<RateLimiter>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Load Environment Variables (FORCED)
    dotenv().expect("Failed to load .env file. Please create one with required variables.");

    let otlp_endpoint = env::var("OTLP_ENDPOINT").ok(); // Optional
    let pg_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL must be set in .env");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in .env");
    
    // Integrations Config
    let sentry_dsn = env::var("SENTRY_DSN").ok();
    let slack_webhook = env::var("SLACK_WEBHOOK_URL").expect("SLACK_WEBHOOK_URL must be set in .env");
    let stripe_key = env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set in .env");
    let stripe_webhook_secret = env::var("STRIPE_WEBHOOK_SECRET").expect("STRIPE_WEBHOOK_SECRET must be set in .env");
    let s3_bucket = env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "my-app-storage".into());
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.sendgrid.net".into());
    let smtp_user = env::var("SMTP_USER").unwrap_or_else(|_| "apikey".into());
    let smtp_pass = env::var("SMTP_PASS").unwrap_or_else(|_| "".into());

    // 2. Telemetry & Observability
    let _sentry_guard = setup_logger(LoggerConfig {
        service_name: "rust-boilerplate".into(),
        otlp_endpoint,
        sentry_dsn,
    })?;
    setup_metrics()?; // Initializes Prometheus exporter

    tracing::info!("Starting Rust Boilerplate with Lego Architecture...");

    // 3. Parallel Database Bootstrap (Fail-Fast)
    tracing::info!("Connecting to primary databases (Postgres, Mongo, Redis)...");
    
    // We use tokio::try_join! to connect to all core databases and storage concurrently.
    // If ANY of them fails, the app aborts immediately (Fail-Fast).
    let (pg_client, mongo_client, redis_client, storage_client) = tokio::try_join!(
        SeaOrmClient::connect(&pg_url),
        MongoClient::connect(&mongo_url, "yalc_db_mongo"),
        RedisClient::connect(&redis_url),
        S3StorageClient::connect(&s3_bucket)
    )?;

    // 4. Background Workers & Integrations
    start_worker(&redis_url).await?; // Spawns the apalis background job processor

    let state = AppState {
        pg_db: Arc::new(pg_client),
        mongo_db: Arc::new(mongo_client),
        redis_db: Arc::new(redis_client),
        slack: Arc::new(SlackClient::new(&slack_webhook)),
        stripe: Arc::new(StripeClient::new(&stripe_key, &stripe_webhook_secret)),
        storage: Arc::new(storage_client),
        mailer: Arc::new(MailerClient::new(&smtp_server, &smtp_user, &smtp_pass, "noreply@yalc.io", "Yalc System")?),
        feature_flags: Arc::new(FeatureFlagsClient::new(&redis_url)?),
        rate_limiter: Arc::new(RateLimiter::new(&redis_url)?),
    };

    // 4. OpenAPI Setup
    let openapi = ApiDoc::openapi();

    // 5. Router Setup
    let router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .route("/health", get(health_check))
        .with_state(state);

    // 6. App Bootstrap & Graceful Shutdown
    // YalcApp now automatically includes Helmet-like security (timeout, CORS, CatchPanic, HSTS, Sentry).
    let app = YalcApp::new(router).with_port(3000);
    
    app.start().await?;

    Ok(())
}
