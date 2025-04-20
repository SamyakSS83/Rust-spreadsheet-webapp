#![cfg(not(tarpaulin_include))]
use cop::spreadsheet::Spreadsheet;
use std::io::{self, Write};

fn execute_command(sheet: &mut Box<Spreadsheet>, cmd: &str, status: &mut String) {
    if cmd.len() == 1 && "wasd".contains(cmd) {
        match cmd {
            "w" if sheet.view_row > 0 => {
                sheet.view_row = (sheet.view_row - 10).max(0);
            }
            "s" if sheet.view_row < sheet.rows - 10 => {
                sheet.view_row = (sheet.view_row + 10).min(sheet.rows - 10);
            }
            "a" if sheet.view_col > 0 => {
                sheet.view_col = (sheet.view_col - 10).max(0);
            }
            "d" if sheet.view_col < sheet.cols - 10 => {
                sheet.view_col = (sheet.view_col + 10).min(sheet.cols - 10);
            }
            _ => {}
        }
        *status = "ok".to_string();
    } else if cmd.starts_with("scroll_to") {
        let cell_name = &cmd[10..];
        if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(cell_name) {
            sheet.view_row = row - 1;
            sheet.view_col = col - 1;
            *status = "ok".to_string();
        } else {
            *status = "invalid cell".to_string();
        }
    } else {
        *status = "invalid command".to_string();
    }
}

fn set_cell(sheet: &mut Box<Spreadsheet>, cell_name: &str, formula: &str) {
    let mut status = String::new();

    if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(cell_name) {
        let (valid, _, _, rhs) = sheet.is_valid_command(cell_name, formula);
        if valid {
            sheet.spreadsheet_set_cell_value(row, col, rhs, &mut status);
        }
    }
}

fn main() {
    println!("Starting interactive scroll test");

    // Create a 25x25 spreadsheet
    let mut sheet = Spreadsheet::spreadsheet_create(25, 25).unwrap();

    // Fill the spreadsheet with values for easy visual testing
    println!("Filling spreadsheet with test data...");
    for i in 1..=25 {
        for j in 1..=25 {
            let cell_name = Spreadsheet::get_cell_name(i, j);
            let formula = format!("{}", i * 100 + j); // Value will be row*100+col
            set_cell(&mut sheet, &cell_name, &formula);
        }
    }

    let mut status = "ok".to_string();

    println!("\n=== SCROLL TEST INTERACTIVE SIMULATION ===");
    println!("Initial view (top-left corner):");
    sheet.spreadsheet_display();

    // Test a series of commands
    let commands = [
        "s",             // scroll down
        "d",             // scroll right
        "s",             // scroll down more
        "scroll_to P20", // jump to cell P20
        "w",             // scroll up
        "a",             // scroll left
    ];

    for cmd in commands.iter() {
        println!("\nExecuting command: '{}'", cmd);
        execute_command(&mut sheet, cmd, &mut status);
        println!("Status: {}", status);
        println!(
            "View position: row={}, col={}",
            sheet.view_row, sheet.view_col
        );
        sheet.spreadsheet_display();
    }

    // Try boundary cases
    println!("\n=== TESTING BOUNDARY CASES ===");

    // Scroll to a corner
    println!("\nScrolling to top-left (A1):");
    execute_command(&mut sheet, "scroll_to A1", &mut status);
    sheet.spreadsheet_display();

    // Try to scroll beyond left boundary
    println!("\nTrying to scroll left beyond boundary ('a'):");
    execute_command(&mut sheet, "a", &mut status);
    println!(
        "Status: {}, view_row={}, view_col={}",
        status, sheet.view_row, sheet.view_col
    );

    // Scroll to bottom-right corner
    println!("\nScrolling to bottom-right (Y25):");
    execute_command(&mut sheet, "scroll_to Y25", &mut status);
    sheet.spreadsheet_display();

    // Try to scroll beyond right/bottom boundary
    println!("\nTrying to scroll down beyond boundary ('s'):");
    execute_command(&mut sheet, "s", &mut status);
    println!(
        "Status: {}, view_row={}, view_col={}",
        status, sheet.view_row, sheet.view_col
    );

    println!("\nTrying to scroll right beyond boundary ('d'):");
    execute_command(&mut sheet, "d", &mut status);
    println!(
        "Status: {}, view_row={}, view_col={}",
        status, sheet.view_row, sheet.view_col
    );

    println!("\nScroll test completed successfully!");
}
