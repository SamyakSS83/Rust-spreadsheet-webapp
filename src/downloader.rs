#![cfg(not(tarpaulin_include))]

use crate::spreadsheet::Spreadsheet;
use std::error::Error;

#[cfg(feature = "web")]
use crate::spreadsheet::{FunctionName, Operand, ParsedRHS};
/// Convert spreadsheet to CSV format
///
/// This function exports a spreadsheet to CSV (Comma-Separated Values) format.
/// It creates a string with the spreadsheet data where:
/// - Column headers are letters (A, B, C, etc.)
/// - Values are comma-separated
/// - Special characters (commas, quotes, newlines) are properly escaped
///
/// # Arguments
/// * `sheet` - Reference to the spreadsheet to convert
///
/// # Returns
/// * `Result<String, Box<dyn Error>>` - CSV content as a string or an error
///
/// # Examples
/// ```use cop::spreadsheet::Spreadsheet;
/// use cop::downloader::to_csv;
///
/// let sheet = Spreadsheet::spreadsheet_create(5, 5).unwrap();
/// match to_csv(&sheet) {
///     Ok(csv) => println!("CSV generated: {} bytes", csv.len()),
///     Err(e) => eprintln!("Failed to generate CSV: {}", e),
/// }
/// ```
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
///
/// This function exports a spreadsheet to XLSX (Excel) format using the rust_xlsxwriter library.
/// It preserves all cell values in a format that Microsoft Excel and other spreadsheet applications can open.
///
/// # Arguments
/// * `sheet` - Reference to the spreadsheet to convert
///
/// # Returns
/// * `Result<Vec<u8>, Box<dyn Error>>` - XLSX file content as bytes or an error
///
/// # Examples
/// ```use cop::spreadsheet::Spreadsheet;
/// use cop::downloader::to_xlsx;
///
/// let sheet = Spreadsheet::spreadsheet_create(5, 5).unwrap();
/// match to_xlsx(&sheet) {
///     Ok(xlsx_data) => println!("XLSX generated: {} bytes", xlsx_data.len()),
///     Err(e) => eprintln!("Failed to generate XLSX: {}", e),
/// }
/// ```
#[cfg(feature = "web")]
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
                // Check if the cell has a formula
                match &cell.formula {
                    ParsedRHS::Function {
                        name,
                        args: (arg1, arg2),
                    } => {
                        // Convert our internal formula to Excel formula syntax
                        let excel_formula = match name {
                            FunctionName::Sum => format!(
                                "=SUM({cell_range})",
                                cell_range = convert_range_to_excel(arg1, arg2)
                            ),
                            FunctionName::Min => format!(
                                "=MIN({cell_range})",
                                cell_range = convert_range_to_excel(arg1, arg2)
                            ),
                            FunctionName::Max => format!(
                                "=MAX({cell_range})",
                                cell_range = convert_range_to_excel(arg1, arg2)
                            ),
                            FunctionName::Avg => format!(
                                "=AVERAGE({cell_range})",
                                cell_range = convert_range_to_excel(arg1, arg2)
                            ),
                            FunctionName::Stdev => format!(
                                "=STDEV({cell_range})",
                                cell_range = convert_range_to_excel(arg1, arg2)
                            ),
                            // For COPY function, just write the value since Excel doesn't have a direct equivalent
                            FunctionName::Copy => {
                                // worksheet.write_number((r - 1) as u32, (c - 1) as u16, cell.value)?;
                                continue;
                            }
                        };

                        // Use as_str() to convert String to &str
                        worksheet.write_formula(
                            (r - 1) as u32,
                            (c - 1) as u16,
                            excel_formula.as_str(),
                        )?;
                        worksheet.write_number((r - 1) as u32, (c - 1) as u16, cell.value)?;
                    }
                    ParsedRHS::Arithmetic { lhs, operator, rhs } => {
                        // Convert arithmetic to Excel formula
                        let left = operand_to_excel_ref(lhs);
                        let right = operand_to_excel_ref(rhs);
                        let excel_formula = format!("={}{}{}", left, operator, right);

                        // Use as_str() to convert String to &str
                        worksheet.write_formula(
                            (r - 1) as u32,
                            (c - 1) as u16,
                            excel_formula.as_str(),
                        )?;
                        worksheet.write_number((r - 1) as u32, (c - 1) as u16, cell.value)?;
                    }
                    ParsedRHS::SingleValue(operand) => {
                        match operand {
                            Operand::Cell(ref_row, ref_col) => {
                                // Cell reference
                                let cell_ref =
                                    format!("={}", Spreadsheet::get_cell_name(*ref_row, *ref_col));
                                // Use as_str() to convert String to &str
                                worksheet.write_formula(
                                    (r - 1) as u32,
                                    (c - 1) as u16,
                                    cell_ref.as_str(),
                                )?;
                                worksheet.write_number(
                                    (r - 1) as u32,
                                    (c - 1) as u16,
                                    cell.value,
                                )?;
                            }
                            Operand::Number(_) => {
                                // Simple number
                                worksheet.write_number(
                                    (r - 1) as u32,
                                    (c - 1) as u16,
                                    cell.value,
                                )?;
                            }
                        }
                    }
                    // Handle Sleep or None by just writing the value
                    _ => {
                        worksheet.write_number((r - 1) as u32, (c - 1) as u16, cell.value)?;
                    }
                }
            }
        }
    }

    workbook.push_worksheet(worksheet);

    // Save to memory buffer
    let buffer = workbook.save_to_buffer()?;

    Ok(buffer)
}

#[cfg(feature = "web")]
// Helper function to convert our operand to Excel reference
fn operand_to_excel_ref(operand: &Operand) -> String {
    match operand {
        Operand::Cell(row, col) => Spreadsheet::get_cell_name(*row, *col),
        Operand::Number(n) => n.to_string(),
    }
}

#[cfg(feature = "web")]
// Helper function to convert our range representation to Excel range format (A1:B2)
fn convert_range_to_excel(start: &Operand, end: &Operand) -> String {
    match (start, end) {
        (Operand::Cell(start_row, start_col), Operand::Cell(end_row, end_col)) => {
            format!(
                "{}:{}",
                Spreadsheet::get_cell_name(*start_row, *start_col),
                Spreadsheet::get_cell_name(*end_row, *end_col)
            )
        }
        // Fallback for cases that don't fit the expected pattern
        _ => String::from("A1"),
    }
}

/// Convert column number to letter (A=1, B=2, etc.)
///
/// Helper function that converts a numerical column index to a spreadsheet-style
/// column letter (A, B, C, ..., Z, AA, AB, etc.).
///
/// # Arguments
/// * `col` - Column number (1-based)
///
/// # Returns
/// * `String` - Column letter (A, B, C, etc.)
///
/// # Examples
/// ```use cop::downloader::column_to_letter;
///
/// assert_eq!(column_to_letter(1), "A");
/// assert_eq!(column_to_letter(26), "Z");
/// assert_eq!(column_to_letter(27), "AA");
/// assert_eq!(column_to_letter(52), "AZ");
/// ```
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
