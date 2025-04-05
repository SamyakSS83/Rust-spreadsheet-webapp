mod app;
mod cell;
mod spreadsheet;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Default values for rows and columns
    let mut rows = 10;
    let mut cols = 10;
    
    // Parse command-line arguments
    if args.len() >= 3 {
        rows = args[1].parse().unwrap_or(10);
        cols = args[2].parse().unwrap_or(10);
    }
    
    // Start the web application
    app::run(rows, cols).await?;
    
    Ok(())
}