#![cfg(not(tarpaulin_include))]

use cop::spreadsheet::{ParsedRHS, Spreadsheet};

// use crate::spreadsheet::{Spreadsheet, Spreadsheet as SpreadsheetTrait};
use std::env;
use std::io::{self, Write};
// use std::os::macos::raw::stat;
use std::time::Instant;

// #[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = Instant::now(); // Start time for the entire program
    let args: Vec<String> = env::args().collect();

    // if args.len() > 1 && args[1] == "-dassi_nahi_to_rassi" {
    //     // Run the web application
    //     let mut rows = 10;
    //     let mut cols = 10;

    //     if args.len() >= 4 {
    //         rows = args[2].parse().unwrap_or(10);
    //         cols = args[3].parse().unwrap_or(10);
    //     }

    //     app::run(rows, cols).await?;
    // } else {
    // Run the spreadsheet functionality
    if args.len() != 3 {
        eprintln!("Usage: {} <rows> <cols>", args[0]);
        return Ok(());
    }

    let rows: i32 = args[1].parse().unwrap_or(0);
    let cols: i32 = args[2].parse().unwrap_or(0);

    if rows < 1 || rows > 999 || cols < 1 || cols > 18278 {
        eprintln!("Error: Invalid dimensions");
        return Ok(());
    }

    let mut start_time = Instant::now(); // Start time for the first command
    let mut sheet = Spreadsheet::spreadsheet_create(rows as u16, cols as u16).unwrap();
    let mut elapsed_time;
    let mut status = String::from("ok");
    let mut show = true;
    loop {
        if show {
            sheet.spreadsheet_display();
        }

        elapsed_time = start_time.elapsed().as_secs_f64(); // Calculate time since the last command
        print!("[{:.1}] ({}) > ", elapsed_time, status);
        io::stdout().flush().unwrap();

        let mut command = String::new();
        if io::stdin().read_line(&mut command).is_err() {
            break;
        }
        let command = command.trim();

        start_time = Instant::now(); // Reset the start time for the next command

        if command.is_empty() {
            status = String::from("invalid command");
            continue;
        }

        if command == "help" {
            println!("Commands:");
            println!("  q: Quit");
            println!("  w: Move up");
            println!("  s: Move down");
            println!("  a: Move left");
            println!("  d: Move right");
            println!("  disable_output: Disable output display");
            println!("  enable_output: Enable output display");
            println!("  scroll_to <cell>: Scroll to the specified cell");
            println!("  <cell>=<formula>: Set the formula for the specified cell");
            continue;
        }

        if command == "q" {
            break;
        } else if command.len() == 1 && "wasd".contains(command) {
            match command {
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
            status = String::from("ok");
        } else if command == "disable_output" {
            show = false;
            status = String::from("ok");
        } else if command == "enable_output" {
            show = true;
            status = String::from("ok");
        } else if command.starts_with("scroll_to") {
            let cell_name = command[10..].trim();
            if let Some((row, col)) = sheet.spreadsheet_parse_cell_name(cell_name) {
                sheet.view_row = row - 1;
                sheet.view_col = col - 1;
                status = String::from("ok");
            } else {
                status = String::from("invalid cell");
            }
        } else if let Some(equal_pos) = command.find('=') {
            sheet.undo_stack.clear();
            let cell_name = &command[..equal_pos];
            let formula = &command[equal_pos + 1..];
            let (valid,row,col,rhs) = sheet.is_valid_command(cell_name, formula);
            if !valid {
                status = String::from("invalid command");
            } else {
                sheet.spreadsheet_set_cell_value(row,col,rhs, &mut status);
            }
        } else if command == "UNDO" {
            // if sheet.undo_stack.is_empty() {
            //     status = String::from("no undo");
            // } else if sheet.undo_stack.len() == 1 {
            //     let (formula, row, col, value, err_state) = (
            //         sheet.undo_stack[0].0.clone(),
            //         sheet.undo_stack[0].1,
            //         sheet.undo_stack[0].2,
            //         sheet.undo_stack[0].3,
            //         sheet.undo_stack[0].4,
            //     );
            //     let cell_name = Spreadsheet::get_cell_name(row, col);
            //     println!("Undoing: {} {} {} {}", cell_name, row, col, value);
            //     if let Some(formula) = formula {
            //         println!("Setting formula: {} {}", cell_name, formula);
            //         sheet.undo_stack.clear();
            //         sheet.spreadsheet_set_cell_value(row as usize, col as usize,rhs, &mut status);
            //     } else {
            //         println!("Setting value: {} {}", cell_name, value);
            //         sheet.spreadsheet_undo();
            //         status = String::from("ok");
            //     }
            // } else {
            //     sheet.spreadsheet_undo();
            //     status = String::from("ok");
            // }
        } else if command == "REDO" {
            if sheet.redo_stack.is_empty() {
                status = String::from("no redo");
            } else {
                sheet.spreadsheet_redo();
                status = String::from("ok");
            }
        } else {
            status = String::from("invalid command 3");
        }

        // Update the start_time after processing the command
        // start_time = Instant::now();
    }
    // }
    let e = s.elapsed().as_secs_f64(); // Calculate total elapsed time
    println!("Total elapsed time: {:.1} seconds", e);

    Ok(())
}
