use yalc_app::setup as setup_app;
use yalc_logger::setup as setup_logger;
use yalc_errors::setup as setup_errors;

// Database Clients
use yalc_database_seaorm::setup as setup_sql_db;
use yalc_database_mongo::setup as setup_nosql_db;
use yalc_database_redis::setup as setup_cache;

// AG-Grid translators
use yalc_ag_grid_seaorm::setup as setup_ag_grid_sql;

#[tokio::main]
async fn main() {
    println!("Starting Rust Boilerplate with Lego Architecture...");
    
    // Mounting the basic Lego pieces
    setup_logger();
    setup_errors();

    // Mounting the Multi-Database Toolbox
    println!("Initializing Multi-Database Toolbox...");
    setup_sql_db(); // Connect to Primary SQL DB (e.g. Postgres)
    setup_nosql_db(); // Connect to Document DB (e.g. MongoDB for logs/events)
    setup_cache(); // Connect to Cache (e.g. Redis)

    // Setup specialized AG-grid filters
    setup_ag_grid_sql();

    // Start App
    setup_app();

    println!("Boilerplate is running successfully with SQL, NoSQL, and Redis support!");
}
