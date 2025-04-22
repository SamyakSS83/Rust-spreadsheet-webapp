#![cfg(not(tarpaulin_include))]

use cop::app;

/// Main entry point for the web application
///
/// This is the main function for the Rust spreadsheet web application.
/// It initializes and runs the web server with the specified spreadsheet dimensions.
///
/// # Arguments
/// * Command line arguments (not directly processed in this function)
///
/// # Default Configuration
/// * Creates a spreadsheet with 10 rows and 10 columns by default
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error object
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments for rows and cols
    // Currently using default values; command line parsing could be added here

    let rows: i16 = 10;
    let cols: i16 = 10;

    // Start the web application with the specified dimensions
    app::run(rows, cols).await
}
