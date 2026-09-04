use yalc_app::setup as setup_app;
use yalc_logger::setup as setup_logger;
use yalc_errors::setup as setup_errors;
use yalc_database::setup as setup_db;

#[tokio::main]
async fn main() {
    println!("Starting Rust Boilerplate with Lego architecture...");
    
    // Mounting the Lego pieces!
    setup_logger();
    setup_errors();
    setup_db();
    setup_app();

    // Additional boilerplate setup like Axum router could go here.
    println!("Boilerplate is running successfully.");
}
