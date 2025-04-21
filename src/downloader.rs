#![cfg(not(tarpaulin_include))]

use crate::spreadsheet::Spreadsheet;
use std::error::Error;
/// Convert spreadsheet to CSV format
pub fn to_csv(sheet: &Spreadsheet) -> Result<String, Box<dyn Error>> {
    let mut csv_content = String::new();

    // Add header row with column letters
    for c in 1..=sheet.cols {
        if c > 1 {
            csv_content.push(',');
        }
        csv_content.push_str(&column_to_letter(c as u16));
    }
    csv_content.push('\n');

    // Add data rows
    for r in 1..=sheet.rows {
        for c in 1..=sheet.cols {
            if c > 1 {
                csv_content.push(',');
            }

            let index = ((r - 1) * sheet.cols + (c - 1)) as usize;
            if let Some(cell) = &sheet.cells[index] {
                // Handle value - escape commas, quotes, newlines as needed
                let value = cell.value.to_string();
                if value.contains(',') || value.contains('"') || value.contains('\n') {
                    let escaped = value.replace("\"", "\"\"");
                    csv_content.push_str(&format!("\"{}\"", escaped));
                } else {
                    csv_content.push_str(&value);
                }
            }
        }
        csv_content.push('\n');
    }

    Ok(csv_content)
}

/// Convert spreadsheet to XLSX format
pub fn to_xlsx(sheet: &Spreadsheet) -> Result<Vec<u8>, Box<dyn Error>> {
    use rust_xlsxwriter::{Workbook, Worksheet};

    // Create a new workbook and worksheet
    let mut workbook = Workbook::new();
    let mut worksheet = Worksheet::new();

    // Write cell data
    for r in 1..=sheet.rows {
        for c in 1..=sheet.cols {
            let index = ((r - 1) * sheet.cols + (c - 1)) as usize;
            if let Some(cell) = &sheet.cells[index] {
                worksheet.write_number((r-1) as u32, (c - 1) as u16, cell.value as i32)?;
            }
        }
    }

    workbook.push_worksheet(worksheet);

    // Save to memory buffer - corrected method call
    let buffer = workbook.save_to_buffer()?;

    Ok(buffer)
}

// Helper function: Convert column number to letter (1=A, 2=B, etc.)
fn column_to_letter(col: u16) -> String {
    let mut name = String::new();
    let mut n = col;

    while n > 0 {
        n -= 1;
        name.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }

    name
}
