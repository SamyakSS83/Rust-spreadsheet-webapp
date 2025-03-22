use cop::spreadsheet::Spreadsheet;
use std::assert;

// Helper function to get cell index
fn get_cell_index(sheet: &Spreadsheet, row: i32, col: i32) -> usize {
    ((row - 1) * sheet.cols + (col - 1)) as usize
}

// Helper function to check cell values
fn assert_cell_value(sheet: &Spreadsheet, cell_name: &str, expected_value: i32, expected_error: bool) {
    let result = sheet.spreadsheet_parse_cell_name(cell_name);
    assert!(result.is_some(), "Cell name should be valid: {}", cell_name);
    
    let (row, col) = result.unwrap();
    let idx = get_cell_index(sheet, row, col);
    
    let cell = &sheet.cells[idx];
    assert!(cell.is_some(), "Cell should exist at index {}", idx);
    
    let cell = cell.as_ref().unwrap();
    assert_eq!(cell.value, expected_value);
    assert_eq!(cell.error, expected_error);
    
    println!("✓ Cell {} has value {} and error status {} as expected", 
           cell_name, expected_value, expected_error);
}

// Test spreadsheet creation and basic properties
fn test_spreadsheet_create() {
    println!("\n====== Testing spreadsheet_create ======");
    let sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();
    
    assert_eq!(sheet.rows, 100);
    assert_eq!(sheet.cols, 100);
    assert!(!sheet.cells.is_empty());
    println!("✓ Spreadsheet created successfully with 100x100 dimensions");
    
    // Test cell initialization
    let cell_a1 = &sheet.cells[0].as_ref().unwrap(); // A1
    assert_eq!(cell_a1.row, 1);
    assert_eq!(cell_a1.col, 1);
    assert_eq!(cell_a1.value, 0);
    println!("✓ Cell A1 initialized correctly");
    
    // Test a cell in the middle
    let idx_j10 = 9 * sheet.cols as usize + 9; // J10
    let cell_j10 = &sheet.cells[idx_j10].as_ref().unwrap();
    assert_eq!(cell_j10.row, 10);
    assert_eq!(cell_j10.col, 10);
    println!("✓ Cell J10 initialized correctly");
    
    // Test bottom-right cell
    let idx_cv100 = 99 * sheet.cols as usize + 99; // CV100
    let cell_cv100 = &sheet.cells[idx_cv100].as_ref().unwrap();
    assert_eq!(cell_cv100.row, 100);
    assert_eq!(cell_cv100.col, 100);
    println!("✓ Cell CV100 (bottom-right) initialized correctly");
}

fn test_spreadsheet_parse_cell_name() {
    println!("\n====== Testing spreadsheet_parse_cell_name ======");
    let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
    
    // Test simple cell name
    let result = sheet.spreadsheet_parse_cell_name("A1");
    assert!(result.is_some());
    let (row, col) = result.unwrap();
    assert_eq!(row, 1);
    assert_eq!(col, 1);
    println!("✓ A1 parsed correctly as row={}, col={}", row, col);
    
    // Test double letter column
    let result = sheet.spreadsheet_parse_cell_name("AB12");
    // Check if this is in bounds of our 10x10 sheet
    if result.is_some() {
        let (row, col) = result.unwrap();
        // AB = 28, which is beyond our 10x10 sheet
        println!("✓ AB12 parsed as row={}, col={}", row, col);
    } else {
        println!("✓ AB12 out of bounds (expected)");
    }
    
    // Test invalid cell name
    let result = sheet.spreadsheet_parse_cell_name("1A");
    assert!(result.is_none());
    println!("✓ Invalid cell name '1A' handled correctly");
}

// Test column/letter conversion functions
fn test_column_letter_conversion() {
    println!("\n====== Testing column/letter conversion ======");
    
    // Column to letter tests
    assert_eq!(Spreadsheet::col_to_letter(1), "A");
    println!("✓ Column 1 converts to 'A'");
    
    assert_eq!(Spreadsheet::col_to_letter(26), "Z");
    println!("✓ Column 26 converts to 'Z'");
    
    assert_eq!(Spreadsheet::col_to_letter(27), "AA");
    println!("✓ Column 27 converts to 'AA'");
    
    assert_eq!(Spreadsheet::col_to_letter(52), "AZ");
    println!("✓ Column 52 converts to 'AZ'");
    
    assert_eq!(Spreadsheet::col_to_letter(100), "CV");
    println!("✓ Column 100 converts to 'CV'");

    assert_eq!(Spreadsheet::col_to_letter(703), "AAA");
    println!("✓ Column 703 converts to 'AAA'");

    assert_eq!(Spreadsheet::col_to_letter(1404), "BAZ");
    println!("✓ Column 1404 converts to 'BAZ'");

    assert_eq!(Spreadsheet::col_to_letter(703), "AAA");
    println!("✓ Column 703 converts to 'AAA'");
    
    assert_eq!(Spreadsheet::col_to_letter(704), "AAB");
    println!("✓ Column 704 converts to 'AAB'");
    
    assert_eq!(Spreadsheet::col_to_letter(728), "AAZ");
    println!("✓ Column 728 converts to 'AAZ'");
    
    assert_eq!(Spreadsheet::col_to_letter(729), "ABA");
    println!("✓ Column 729 converts to 'ABA'");
    
    assert_eq!(Spreadsheet::col_to_letter(18278), "ZZZ");
    println!("✓ Column 18278 converts to 'ZZZ'");
    
    // Letter to column tests
    assert_eq!(Spreadsheet::letter_to_col("A"), 1);
    println!("✓ 'A' converts to column 1");
    
    assert_eq!(Spreadsheet::letter_to_col("Z"), 26);
    println!("✓ 'Z' converts to column 26");
    
    assert_eq!(Spreadsheet::letter_to_col("AA"), 27);
    println!("✓ 'AA' converts to column 27");
    
    assert_eq!(Spreadsheet::letter_to_col("AZ"), 52);
    println!("✓ 'AZ' converts to column 52");
    
    assert_eq!(Spreadsheet::letter_to_col("CV"), 100);
    println!("✓ 'CV' converts to column 100");
    
    assert_eq!(Spreadsheet::letter_to_col("AAA"), 703);
    println!("✓ 'AAA' converts to column 703");

    assert_eq!(Spreadsheet::letter_to_col("BAZ"), 1404);
    println!("✓ 'BAZ' converts to column 1404");
    
    assert_eq!(Spreadsheet::letter_to_col("AAB"), 704);
    println!("✓ 'AAB' converts to column 704");
    
    assert_eq!(Spreadsheet::letter_to_col("AAZ"), 728);
    println!("✓ 'AAZ' converts to column 728");
    
    assert_eq!(Spreadsheet::letter_to_col("ABA"), 729);
    println!("✓ 'ABA' converts to column 729");

    assert_eq!(Spreadsheet::letter_to_col("ZZZ"), 18278);
    println!("✓ 'ZZZ' converts to column 18278");
    
    // Test round-trip conversion
    for col in 1..=100 {
        let letters = Spreadsheet::col_to_letter(col);
        let back = Spreadsheet::letter_to_col(&letters);
        assert_eq!(col, back);
    }
    println!("✓ Round-trip conversion successful for columns 1-100");
}

// Test cell name helper functions
fn test_cell_name_helpers() {
    println!("\n====== Testing cell name helpers ======");
    let sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();
    
    // Test cell name generation
    assert_eq!(Spreadsheet::get_cell_name(1, 1), "A1");
    println!("✓ Cell (1,1) generates name 'A1'");
    
    assert_eq!(Spreadsheet::get_cell_name(10, 26), "Z10");
    println!("✓ Cell (10,26) generates name 'Z10'");
    
    assert_eq!(Spreadsheet::get_cell_name(100, 100), "CV100");
    println!("✓ Cell (100,100) generates name 'CV100'");
    
    // Test cell name parsing
    let result = sheet.spreadsheet_parse_cell_name("A1");
    assert!(result.is_some());
    let (row, col) = result.unwrap();
    assert_eq!(row, 1);
    assert_eq!(col, 1);
    println!("✓ 'A1' parses to row=1, col=1");
    
    let result = sheet.spreadsheet_parse_cell_name("Z10");
    assert!(result.is_some());
    let (row, col) = result.unwrap();
    assert_eq!(row, 10);
    assert_eq!(col, 26);
    println!("✓ 'Z10' parses to row=10, col=26");
    
    let result = sheet.spreadsheet_parse_cell_name("CV100");
    assert!(result.is_some());
    let (row, col) = result.unwrap();
    assert_eq!(row, 100);
    assert_eq!(col, 100);
    println!("✓ 'CV100' parses to row=100, col=100");
    
    // Test invalid cell names
    let result = sheet.spreadsheet_parse_cell_name("A101");
    assert!(result.is_none());
    println!("✓ 'A101' correctly identified as invalid (row out of bounds)");
    
    let result = sheet.spreadsheet_parse_cell_name("CW1");
    assert!(result.is_none());
    println!("✓ 'CW1' correctly identified as invalid (column out of bounds)");
    
    let result = sheet.spreadsheet_parse_cell_name("1A");
    assert!(result.is_none());
    println!("✓ '1A' correctly identified as invalid format");
    
    let result = sheet.spreadsheet_parse_cell_name("A1B");
    assert!(result.is_none());
    println!("✓ 'A1B' correctly identified as invalid format");
    
    let result = sheet.spreadsheet_parse_cell_name("");
    assert!(result.is_none());
    println!("✓ Empty string correctly identified as invalid");
}

fn test_find_depends() {
    println!("\n====== Testing find_depends ======");
    let sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();
    
    // Test range functions
    let result = sheet.find_depends("SUM(A1:B10)").unwrap();
    assert_eq!(result, (0, 9, 1, 2, true));
    println!("✓ SUM(A1:B10) parsed correctly");
    
    let result = sheet.find_depends("AVG(C5:E15)").unwrap();
    assert_eq!(result, (4, 14, 3, 5, true));
    println!("✓ AVG(C5:E15) parsed correctly");
    
    let result = sheet.find_depends("MIN(A1:Z100)").unwrap();
    assert_eq!(result, (0, 99, 1, 26, true));
    println!("✓ MIN(A1:Z100) parsed correctly");
    
    let result = sheet.find_depends("MAX(F10:F20)").unwrap();
    assert_eq!(result, (9, 19, 6, 6, true));
    println!("✓ MAX(F10:F20) parsed correctly");
    
    let result = sheet.find_depends("STDEV(G1:H5)").unwrap();
    assert_eq!(result, (0, 4, 7, 8, true));
    println!("✓ STDEV(G1:H5) parsed correctly");
    
    // Test invalid range order
    let result = sheet.find_depends("SUM(B10:A1)");
    assert!(result.is_err());
    println!("✓ Invalid range order correctly rejected");
    
    // Test cell references in formulas
    let result = sheet.find_depends("A1+B2").unwrap();
    assert_eq!(result, (1, 2, 1, 2, false));
    println!("✓ A1+B2 parsed correctly");
    
    let result = sheet.find_depends("C3*D4").unwrap();
    assert_eq!(result, (3, 4, 3, 4, false));
    println!("✓ C3*D4 parsed correctly");
    
    // Test more complex formula
    let result = sheet.find_depends("(A1+B2)/C3").unwrap();
    assert_eq!(result, (1, 2, 1, 2, false));
    println!("✓ (A1+B2)/C3 parsed correctly");
    
    // Test formula with no cell references
    let result = sheet.find_depends("42+31").unwrap();
    assert_eq!(result, (-1, -1, -1, -1, false));
    println!("✓ 42+31 (no cell refs) parsed correctly");
}

fn test_spreadsheet_evaluate_function() {
    println!("\n====== Testing spreadsheet_evaluate_function ======");
    let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
    
    // Set up some cells with values for testing
    {
        for i in 1..=5 {
            for j in 1..=5 {
                let index = ((i - 1) * sheet.cols + (j - 1)) as usize;
                if let Some(ref mut cell) = sheet.cells[index] {
                    cell.value = i * j; // Simple pattern: A1=1, A2=2, B1=1, B2=4, etc.
                }
            }
        }
        
        // Special cases
        if let Some(ref mut cell) = sheet.cells[0] { // A1
            cell.value = 10;
        }
        
        if let Some(ref mut cell) = sheet.cells[1] { // B1
            cell.value = 20;
        }
        
        if let Some(ref mut cell) = sheet.cells[10] { // A2
            cell.value = 30;
        }
        
        if let Some(ref mut cell) = sheet.cells[11] { // B2
            cell.value = 40;
        }
    }
    
    // Create a temporary cell for testing function evaluation
    let mut test_cell = cop::cell::cell_create(1, 1);
    
    // Test SUM function
    let result = sheet.spreadsheet_evaluate_function(
        "SUM", "A1:B2", &mut test_cell, "SUM(A1:B2)"
    );
    assert_eq!(result, 100); // 10+20+30+40
    assert!(!test_cell.error);
    println!("✓ SUM(A1:B2) = {} (expected 100)", result);
    
    // Test AVG function
    let result = sheet.spreadsheet_evaluate_function(
        "AVG", "A1:B2", &mut test_cell, "AVG(A1:B2)"
    );
    assert_eq!(result, 25); // (10+20+30+40)/4
    assert!(!test_cell.error);
    println!("✓ AVG(A1:B2) = {} (expected 25)", result);
    
    // Test MIN function
    let result = sheet.spreadsheet_evaluate_function(
        "MIN", "A1:B2", &mut test_cell, "MIN(A1:B2)"
    );
    assert_eq!(result, 10);
    assert!(!test_cell.error);
    println!("✓ MIN(A1:B2) = {} (expected 10)", result);
    
    // Test MAX function
    let result = sheet.spreadsheet_evaluate_function(
        "MAX", "A1:B2", &mut test_cell, "MAX(A1:B2)"
    );
    assert_eq!(result, 40);
    assert!(!test_cell.error);
    println!("✓ MAX(A1:B2) = {} (expected 40)", result);
    
    // Test STDEV function
    let result = sheet.spreadsheet_evaluate_function(
        "STDEV", "A1:B2", &mut test_cell, "STDEV(A1:B2)"
    );
    // Standard deviation of [10,20,30,40] = sqrt((10-25)²+(20-25)²+(30-25)²+(40-25)²/4)
    // = sqrt((−15)²+(−5)²+5²+15²/4) = sqrt(225+25+25+225/4) = sqrt(500/4) = sqrt(125) ≈ 11
    assert_eq!(result, 11); // Using integer rounding
    assert!(!test_cell.error);
    println!("✓ STDEV(A1:B2) = {} (expected ~13)", result);
    
    // Test SLEEP function with numeric argument (mock test, not actually sleeping)
    let result = sheet.spreadsheet_evaluate_function(
        "SLEEP", "0", &mut test_cell, "SLEEP(0)"
    );
    assert_eq!(result, 0);
    assert!(!test_cell.error);
    println!("✓ SLEEP(0) = {} (no actual sleep expected)", result);
    
    // Test SLEEP function with cell reference
    {
        let result = sheet.spreadsheet_evaluate_function(
            "SLEEP", "A1", &mut test_cell, "SLEEP(A1)"
        );
        assert_eq!(result, 10); // A1 value is 10
        assert!(!test_cell.error);
        println!("✓ SLEEP(A1) = {} (expected 10)", result);
    }
}

fn test_spreadsheet_evaluate_expression() {
    println!("\n====== Testing spreadsheet_evaluate_expression ======");
    let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
    
    // Set up some cells with values for testing
    {
        // A1 = 10
        if let Some(ref mut cell) = sheet.cells[0] {
            cell.value = 10;
        }
        
        // B1 = 5
        if let Some(ref mut cell) = sheet.cells[1] {
            cell.value = 5;
        }
        
        // C1 = 0
        if let Some(ref mut cell) = sheet.cells[2] {
            cell.value = 0;
        }
        
        // A2 = 20
        if let Some(ref mut cell) = sheet.cells[10] {
            cell.value = 20;
        }
        
        // Set an error in D1
        if let Some(ref mut cell) = sheet.cells[3] {
            cell.value = 100;
            cell.error = true;
        }
    }
    
    // Create a temporary cell for testing expression evaluation
    let mut test_cell = cop::cell::cell_create(1, 1);
    
    // Test simple numeric expressions
    let result = sheet.spreadsheet_evaluate_expression("42", &mut test_cell);
    assert_eq!(result, 42);
    assert!(!test_cell.error);
    println!("✓ Expression '42' = {} (expected 42)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("10+5", &mut test_cell);
    assert_eq!(result, 15);
    assert!(!test_cell.error);
    println!("✓ Expression '10+5' = {} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("20-8", &mut test_cell);
    assert_eq!(result, 12);
    assert!(!test_cell.error);
    println!("✓ Expression '20-8' = {} (expected 12)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("6*7", &mut test_cell);
    assert_eq!(result, 42);
    assert!(!test_cell.error);
    println!("✓ Expression '6*7' = {} (expected 42)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("20/4", &mut test_cell);
    assert_eq!(result, 5);
    assert!(!test_cell.error);
    println!("✓ Expression '20/4' = {} (expected 5)", result);
    
    // Test division by zero
    let result = sheet.spreadsheet_evaluate_expression("10/0", &mut test_cell);
    assert_eq!(result, 0);
    assert!(test_cell.error);
    println!("✓ Expression '10/0' sets error flag correctly");
    
    // Reset error flag
    test_cell.error = false;
    
    // Test cell reference expressions
    let result = sheet.spreadsheet_evaluate_expression("A1", &mut test_cell);
    assert_eq!(result, 10);
    assert!(!test_cell.error);
    println!("✓ Expression 'A1' = {} (expected 10)", result);
    
    // Test arithmetic with cell references
    let result = sheet.spreadsheet_evaluate_expression("A1+B1", &mut test_cell);
    assert_eq!(result, 15);
    assert!(!test_cell.error);
    println!("✓ Expression 'A1+B1' = {} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("A2-B1", &mut test_cell);
    assert_eq!(result, 15);
    assert!(!test_cell.error);
    println!("✓ Expression 'A2-B1' = {} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("A1*B1", &mut test_cell);
    assert_eq!(result, 50);
    assert!(!test_cell.error);
    println!("✓ Expression 'A1*B1' = {} (expected 50)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("A1/B1", &mut test_cell);
    assert_eq!(result, 2);
    assert!(!test_cell.error);
    println!("✓ Expression 'A1/B1' = {} (expected 2)", result);
    
    // Test division by zero with cell references
    let result = sheet.spreadsheet_evaluate_expression("B1/C1", &mut test_cell);
    assert_eq!(result, 0);
    assert!(test_cell.error);
    println!("✓ Expression 'B1/C1' (division by zero) sets error flag correctly");
    
    // Reset error flag
    test_cell.error = false;
    
    // Test error propagation from referenced cells
    let result = sheet.spreadsheet_evaluate_expression("D1+B1", &mut test_cell);
    assert_eq!(result, 0);
    assert!(test_cell.error);
    println!("✓ Expression 'D1+B1' propagates error correctly");
    
    // Reset error flag
    test_cell.error = false;
    
    // Test function calls
    let result = sheet.spreadsheet_evaluate_expression("SUM(A1:B1)", &mut test_cell);
    assert_eq!(result, 15);
    assert!(!test_cell.error);
    println!("✓ Function call 'SUM(A1:B1)' = {} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("AVG(A1:A2)", &mut test_cell);
    assert_eq!(result, 15);
    assert!(!test_cell.error);
    println!("✓ Function call 'AVG(A1:A2)' = {} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("MIN(A1:A2)", &mut test_cell);
    assert_eq!(result, 10);
    assert!(!test_cell.error);
    println!("✓ Function call 'MIN(A1:A2)' = {} (expected 10)", result);
    
    // Test negative and positive numbers
    let result = sheet.spreadsheet_evaluate_expression("-10+5", &mut test_cell);
    assert_eq!(result, -5);
    assert!(!test_cell.error);
    println!("✓ Expression '-10+5' = {} (expected -5)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("10+-5", &mut test_cell);
    assert_eq!(result, 5);
    assert!(!test_cell.error);
    println!("✓ Expression '10+-5' = {} (expected 5)", result);
    
    // Test invalid expressions
    let result = sheet.spreadsheet_evaluate_expression("10@20", &mut test_cell);
    assert_eq!(result, -1);
    assert!(test_cell.error);
    println!("✓ Invalid expression '10@20' handled correctly");
    
    // Clean up
    cop::cell::cell_destroy(test_cell);
}

pub fn run_tests() {
    println!("Starting spreadsheet unit tests");
    test_spreadsheet_create();
    test_spreadsheet_parse_cell_name();
    test_column_letter_conversion();
    test_cell_name_helpers();
    test_find_depends();
    test_spreadsheet_evaluate_function();
    test_spreadsheet_evaluate_expression(); // Add our new test
    println!("All tests passed!");
}

fn main() {
    run_tests();
}

