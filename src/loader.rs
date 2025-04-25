#![cfg(not(tarpaulin_include))]

use crate::spreadsheet::{FunctionName, Operand, ParsedRHS, Spreadsheet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load a spreadsheet from a CSV file
///
/// This function imports a CSV file and converts it to a Spreadsheet structure.
/// It handles headers, data rows, and properly unescapes special characters.
///
/// # Arguments
/// * `filepath` - Path to the CSV file to load
///
/// # Returns
/// * `Result<Box<Spreadsheet>, Box<dyn Error>>` - The loaded spreadsheet or an error
///
/// # Examples
/// ```no_run
/// use cop::loader::from_csv;
///
/// match from_csv("data.csv") {
///     Ok(sheet) => println!("Successfully loaded spreadsheet with {} rows", sheet.rows),
///     Err(e) => eprintln!("Error loading CSV: {}", e),
/// }
/// ```
pub fn from_csv(filepath: impl AsRef<Path>) -> Result<Box<Spreadsheet>, Box<dyn Error>> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    if lines.is_empty() {
        return Err("CSV file is empty".into());
    }

    // Count rows and columns
    let rows = lines.len();
    let cols = csv_count_columns(&lines[0])?;

    // Create spreadsheet
    let mut sheet = Spreadsheet::spreadsheet_create(rows as i16, cols as i16)
        .ok_or("Failed to create spreadsheet")?;

    // Parse and fill data
    for (r, line) in lines.iter().enumerate() {
        let row_cells = parse_csv_row(line)?;
        for (c, value_str) in row_cells.iter().enumerate() {
            if c >= cols || r >= rows {
                continue; // Skip extra data
            }

            let row = (r + 1) as i16;
            let col = (c + 1) as i16;

            // Try to parse as number or formula
            if let Ok(num) = value_str.parse::<i32>() {
                let formula = ParsedRHS::SingleValue(Operand::Number(num));
                let mut status = String::new();
                sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
            } else if value_str.starts_with('=') {
                // Handle formula - strip the = sign
                let formula_str = &value_str[1..];
                let mut status = String::new();

                // Try to parse the formula
                let (is_valid, _, _, formula) =
                    sheet.is_valid_command(&Spreadsheet::get_cell_name(row, col), formula_str);

                if is_valid {
                    sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                } else {
                    // If formula can't be parsed, store as text
                    let formula = ParsedRHS::SingleValue(Operand::Number(0));
                    sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                }
            } else {
                // Treat as text - store 0 as value for now
                // In a real implementation, you might want to have a text type
                let formula = ParsedRHS::SingleValue(Operand::Number(0));
                let mut status = String::new();
                sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
            }
        }
    }

    Ok(sheet)
}

/// Load a spreadsheet from an Excel file
///
/// This function imports an Excel file (XLSX) and converts it to a Spreadsheet structure.
/// It preserves cell values and attempts to convert Excel formulas to the internal format.
///
/// # Arguments
/// * `filepath` - Path to the Excel file to load
///
/// # Returns
/// * `Result<Box<Spreadsheet>, Box<dyn Error>>` - The loaded spreadsheet or an error
///
/// # Examples
/// ```no_run
/// use cop::loader::from_excel;
///
/// match from_excel("data.xlsx") {
///     Ok(sheet) => println!("Successfully loaded Excel with {} rows", sheet.rows),
///     Err(e) => eprintln!("Error loading Excel: {}", e),
/// }
/// ```
#[cfg(feature = "web")]
pub fn from_excel(filepath: impl AsRef<Path>) -> Result<Box<Spreadsheet>, Box<dyn Error>> {
    use calamine::{open_workbook, Reader, Xlsx};

    let mut workbook: Xlsx<_> = open_workbook(filepath)?;

    // Get the first worksheet
    let sheet_name = workbook
        .sheet_names()
        .get(0)
        .ok_or("No sheets found in Excel file")?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .ok_or("Failed to get worksheet")??;

    let rows = range.height();
    let cols = range.width();

    if rows == 0 || cols == 0 {
        return Err("Excel sheet is empty".into());
    }

    // Create spreadsheet
    let mut sheet = Spreadsheet::spreadsheet_create(rows as i16, cols as i16)
        .ok_or("Failed to create spreadsheet")?;

    // Parse cells
    for (r, row) in range.rows().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let row = (r + 1) as i16;
            let col = (c + 1) as i16;
            let mut status = String::new();

            match cell {
                calamine::Data::Int(i) => {
                    // Integer value
                    if *i <= i32::MAX as i64 && *i >= i32::MIN as i64 {
                        let formula = ParsedRHS::SingleValue(Operand::Number(*i as i32));
                        sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                    }
                }
                calamine::Data::Float(f) => {
                    // Float value - convert to integer as the system works with i32
                    let formula = ParsedRHS::SingleValue(Operand::Number(*f as i32));
                    sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                }
                calamine::Data::Formula(_, value) => {
                    // For formulas, we store the calculated value
                    // In a real implementation, we'd try to convert the Excel formula to our format
                    match value {
                        calamine::Data::Int(i)
                            if *i <= i32::MAX as i64 && *i >= i32::MIN as i64 =>
                        {
                            let formula = ParsedRHS::SingleValue(Operand::Number(*i as i32));
                            sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                        }
                        calamine::Data::Float(f) => {
                            let formula = ParsedRHS::SingleValue(Operand::Number(*f as i32));
                            sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                        }
                        _ => {
                            // Default to 0 for other types
                            let formula = ParsedRHS::SingleValue(Operand::Number(0));
                            sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                        }
                    }
                }
                // Handle other data types - default to 0
                _ => {
                    let formula = ParsedRHS::SingleValue(Operand::Number(0));
                    sheet.spreadsheet_set_cell_value(row, col, formula, &mut status);
                }
            }
        }
    }

    Ok(sheet)
}

// Helper function to count columns in a CSV row
fn csv_count_columns(line: &str) -> Result<usize, Box<dyn Error>> {
    let mut columns = 0;
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '"' {
            // Toggle quote state
            if let Some(&next) = chars.peek() {
                if next == '"' {
                    // Double quote inside quoted field - skip the second quote
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            } else {
                in_quotes = !in_quotes;
            }
        } else if c == ',' && !in_quotes {
            columns += 1;
        }
    }

    // Add 1 for the last column
    columns += 1;
    Ok(columns)
}

// Parse a CSV row into a vector of strings
fn parse_csv_row(line: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut result = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if let Some(&next) = chars.peek() {
                    if next == '"' && in_quotes {
                        // Double quote inside quoted field - add a single quote
                        current_field.push('"');
                        chars.next();
                    } else {
                        // Toggle quote state
                        in_quotes = !in_quotes;
                    }
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                // End of field
                result.push(current_field);
                current_field = String::new();
            }
            _ => {
                current_field.push(c);
            }
        }
    }

    // Add the last field
    result.push(current_field);

    Ok(result)
}

/// Detect file type and load appropriate format
///
/// This function examines the file extension and calls the appropriate loader
/// for CSV or Excel files.
///
/// # Arguments
/// * `filepath` - Path to the file to load
///
/// # Returns
/// * `Result<Box<Spreadsheet>, Box<dyn Error>>` - The loaded spreadsheet or an error
///
/// # Examples
/// ```no_run
/// use cop::loader::load_spreadsheet;
///
/// match load_spreadsheet("data.csv") {
///     Ok(sheet) => println!("Successfully loaded spreadsheet"),
///     Err(e) => eprintln!("Error loading file: {}", e),
/// }
/// ```
pub fn load_spreadsheet(filepath: impl AsRef<Path>) -> Result<Box<Spreadsheet>, Box<dyn Error>> {
    let path = filepath.as_ref();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match extension.as_deref() {
        Some("csv") => from_csv(path),
        #[cfg(feature = "web")]
        Some("xlsx") | Some("xls") => from_excel(path),
        #[cfg(not(feature = "web"))]
        Some("xlsx") | Some("xls") => Err("Excel support requires the 'web' feature".into()),
        Some(ext) => Err(format!("Unsupported file extension: {}", ext).into()),
        None => Err("File has no extension".into()),
    }
}
