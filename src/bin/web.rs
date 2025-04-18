use cop::app;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments for rows and cols
    let args: Vec<String> = env::args().collect();
    
    let mut rows = 10;
    let mut cols = 10;

    if args.len() >= 3 {
        rows = args[1].parse().unwrap_or(10);
        cols = args[2].parse().unwrap_or(10);
    }

    println!("Starting web server with spreadsheet of {} rows x {} columns", rows, cols);
    app::run(rows, cols).await
}