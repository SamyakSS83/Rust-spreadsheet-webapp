// use cop::saving::{load_spreadsheet, save_spreadsheet};
// use cop::spreadsheet::Spreadsheet;
// use std::fs;
// use std::path::Path;

// fn test_save_and_load_spreadsheet() -> std::io::Result<()> {
//     // Create a temporary filename
//     let filename = "test_spreadsheet.bin.gz";

//     // Ensure the file doesn't exist before we start
//     if Path::new(filename).exists() {
//         fs::remove_file(filename)?;
//     }

//     // Create a new spreadsheet
//     let mut sheet = Spreadsheet::spreadsheet_create(5, 5).expect("Failed to create spreadsheet");

//     // Insert some data
//     let mut status_out = String::new();

//     // Set some values and a formula
//     sheet.spreadsheet_set_cell_value("A1", "10", &mut status_out);
//     assert_eq!(status_out, "ok", "Failed to set A1 value");

//     sheet.spreadsheet_set_cell_value("B1", "20", &mut status_out);
//     assert_eq!(status_out, "ok", "Failed to set B1 value");

//     // Set a formula that adds A1 and B1
//     sheet.spreadsheet_set_cell_value("C1", "A1+B1", &mut status_out);
//     assert_eq!(status_out, "ok", "Failed to set formula in C1");

//     // Verify the formula calculation worked
//     let c1_index = (2) as usize; // C1 is at row 0, col 2 in 0-indexed
//     let c1_value = sheet.cells[c1_index].as_ref().unwrap().value;
//     assert_eq!(c1_value, 30, "Formula didn't evaluate to expected value");

//     // Save the spreadsheet to a file
//     save_spreadsheet(&sheet, filename)?;

//     // Verify the file was created
//     assert!(Path::new(filename).exists(), "File was not created");

//     // Load the spreadsheet from that file
//     let loaded_sheet = load_spreadsheet(filename)?;

//     // Check dimensions
//     assert_eq!(loaded_sheet.rows, sheet.rows, "Rows don't match");
//     assert_eq!(loaded_sheet.cols, sheet.cols, "Columns don't match");

//     // Check cell values
//     let a1_index = (0) as usize; // A1 is at row 0, col 0 in 0-indexed
//     let a1_value = loaded_sheet.cells[a1_index].as_ref().unwrap().value;
//     assert_eq!(a1_value, 10, "A1 value wasn't preserved");

//     let b1_index = (1) as usize; // B1 is at row 0, col 1 in 0-indexed
//     let b1_value = loaded_sheet.cells[b1_index].as_ref().unwrap().value;
//     assert_eq!(b1_value, 20, "B1 value wasn't preserved");

//     // Check the formula result
//     let c1_loaded_index = (2) as usize; // C1 is at row 0, col 2 in 0-indexed
//     let c1_loaded_value = loaded_sheet.cells[c1_loaded_index].as_ref().unwrap().value;
//     assert_eq!(c1_loaded_value, 30, "C1 formula result wasn't preserved");

//     // Check that the formula itself was preserved
//     let c1_formula = loaded_sheet.cells[c1_loaded_index]
//         .as_ref()
//         .unwrap()
//         .formula
//         .as_ref()
//         .unwrap();
//     assert_eq!(c1_formula, "A1+B1", "Formula wasn't preserved");

//     // Cleanup
//     fs::remove_file(filename)?;

//     Ok(())
// }

// fn main() {
//     // Run the test
//     if let Err(e) = test_save_and_load_spreadsheet() {
//         eprintln!("Test failed: {}", e);
//     } else {
//         println!("Test passed!");
//     }
// }

fn main(){
    
}
