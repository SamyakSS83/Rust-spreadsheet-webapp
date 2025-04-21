#![cfg(not(tarpaulin_include))]

use cop::app;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments for rows and cols

    let mut rows: i16 = 10;
    let mut cols: i16 = 10;

    // println!("Starting web server with spreadsheet of {} rows x {} columns", rows, cols);
    app::run(rows, cols).await
}
