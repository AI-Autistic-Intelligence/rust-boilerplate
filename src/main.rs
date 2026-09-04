use rust_yalc::core::hello_from_yalc;

#[tokio::main]
async fn main() {
    println!("Starting Rust Boilerplate...");
    
    // Call the shared library function
    hello_from_yalc();

    // Additional boilerplate setup like Axum router could go here.
    println!("Boilerplate is running successfully.");
}
