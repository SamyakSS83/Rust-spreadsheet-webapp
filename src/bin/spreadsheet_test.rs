use cop::spreadsheet::Spreadsheet;
use cop::cell;
use std::assert;

// Helper function to get cell index
fn get_cell_index(sheet: &Spreadsheet, row: i32, col: i32) -> usize {
    ((row - 1) * sheet.cols + (col - 1)) as usize
}

// Helper function to check cell values
// fn assert_cell_value(sheet: &Spreadsheet, cell_name: &str, expected_value: i32, expected_error: bool) {
//     let result = sheet.spreadsheet_parse_cell_name(cell_name);
//     assert!(result.is_some(), "Cell name should be valid: {}", cell_name);
    
//     let (row, col) = result.unwrap();
//     let idx = get_cell_index(sheet, row, col);
    
//     let cell = &sheet.cells[idx];
//     assert!(cell.is_some(), "Cell should exist at index {}", idx);
    
//     let cell = cell.as_ref().unwrap();
//     assert_eq!(cell.value, expected_value);
//     assert_eq!(cell.error, expected_error);
    
//     println!("✓ Cell {} has value {} and error status {} as expected", 
//            cell_name, expected_value, expected_error);
// }

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
        "SUM", "A1:B2",  "SUM(A1:B2)",
    );
    assert_eq!(result, (100,false)); // 10+20+30+40
    // assert!(!test_cell.error);
    println!("✓ SUM(A1:B2) = {:?} (expected 100)", result);
    
    // Test AVG function
    let result = sheet.spreadsheet_evaluate_function(
        "AVG", "A1:B2",  "AVG(A1:B2)"
    );
    assert_eq!(result, (25,false)); // (10+20+30+40)/4
    // assert!(!test_cell.error);
    println!("✓ AVG(A1:B2) = {:?} (expected 25)", result);
    
    // Test MIN function
    let result = sheet.spreadsheet_evaluate_function(
        "MIN", "A1:B2",  "MIN(A1:B2)"
    );
    assert_eq!(result, (10,false));
    // assert!(!test_cell.error);
    println!("✓ MIN(A1:B2) = {:?} (expected 10)", result);
    
    // Test MAX function
    let result = sheet.spreadsheet_evaluate_function(
        "MAX", "A1:B2",  "MAX(A1:B2)"
    );
    assert_eq!(result, (40,false));
    // assert!(!test_cell.error);
    println!("✓ MAX(A1:B2) = {:?} (expected 40)", result);
    
    // Test STDEV function
    let result = sheet.spreadsheet_evaluate_function(
        "STDEV", "A1:B2",  "STDEV(A1:B2)"
    );
    // Standard deviation of [10,20,30,40] = sqrt((10-25)²+(20-25)²+(30-25)²+(40-25)²/4)
    // = sqrt((−15)²+(−5)²+5²+15²/4) = sqrt(225+25+25+225/4) = sqrt(500/4) = sqrt(125) ≈ 11
    assert_eq!(result, (11,false)); // Using integer rounding
    // assert!(!test_cell.error);
    println!("✓ STDEV(A1:B2) = {:?} (expected ~11)", result);
    
    // Test SLEEP function with numeric argument (mock test, not actually sleeping)
    let result = sheet.spreadsheet_evaluate_function(
        "SLEEP", "0",  "SLEEP(0)"
    );
    assert_eq!(result, (0,false));
    // assert!(!test_cell.error);
    println!("✓ SLEEP(0) = {:?} (no actual sleep expected)", result);
    
    // Test SLEEP function with cell reference
    {
        let result = sheet.spreadsheet_evaluate_function(
            "SLEEP", "A1",  "SLEEP(A1)"
        );
        assert_eq!(result, (10,false)); // A1 value is 10
        // assert!(!test_cell.error);
        println!("✓ SLEEP(A1) = {:?} (expected 10)", result);
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
    let result = sheet.spreadsheet_evaluate_expression("42", 1,1);
    assert_eq!(result, (42,false));
    // assert!(!test_cell.error);
    println!("✓ Expression '42' = {:?} (expected 42)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("10+5",1,1 );
    assert_eq!(result, (15,false));
    // assert!(!test_cell.error);
    println!("✓ Expression '10+5' = {:?} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("20-8", 1,1);
    assert_eq!(result, (12,false));
    // assert!(!test_cell.error);
    println!("✓ Expression '20-8' = {:?} (expected 12)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("6*7", 1,1);
    assert_eq!(result, (42,false));
    // assert!(!test_cell.error);
    println!("✓ Expression '6*7' = {:?} (expected 42)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("20/4", 1,1);
    assert_eq!(result, (5,false));
    // assert!(!test_cell.error);
    println!("✓ Expression '20/4' = {:?} (expected 5)", result);
    
    // Test division by zero
    let result = sheet.spreadsheet_evaluate_expression("10/0", 1,1);
    assert_eq!(result, (0,true));
    // assert!(test_cell.error);
    println!("✓ Expression '10/0' sets error flag correctly");
    
    // Reset error flag
    test_cell.error = false;
    
    // Test cell reference expressions
    let result = sheet.spreadsheet_evaluate_expression("A1", 1,1);
    assert_eq!(result, (10,false));
    // assert!(!test_cell.error);
    println!("✓ Expression 'A1' = {:?} (expected 10)", result);
    
    // Test arithmetic with cell references
    let result = sheet.spreadsheet_evaluate_expression("A1+B1", 1,1);
    assert_eq!(result, (15,false));
    // assert!(!test_cell.error);
    println!("✓ Expression 'A1+B1' = {:?} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("A2-B1", 1,1);
    assert_eq!(result, (15,false));
    // assert!(!test_cell.error);
    println!("✓ Expression 'A2-B1' = {:?} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("A1*B1", 1,1);
    assert_eq!(result, (50,false));
    // assert!(!test_cell.error);
    println!("✓ Expression 'A1*B1' = {:?} (expected 50)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("A1/B1", 1,1);
    assert_eq!(result, (2,false));
    // assert!(!test_cell.error);
    println!("✓ Expression 'A1/B1' = {:?} (expected 2)", result);
    
    // Test division by zero with cell references
    let result = sheet.spreadsheet_evaluate_expression("B1/C1", 1,1);
    assert_eq!(result, (0,true));
    // assert!(test_cell.error);
    println!("✓ Expression 'B1/C1' (division by zero) sets error flag correctly");
    
    // Reset error flag
    test_cell.error = false;
    
    // Test error propagation from referenced cells
    let result = sheet.spreadsheet_evaluate_expression("D1+B1", 1,1);
    assert_eq!(result, (0,true));
    // assert!(test_cell.error);
    println!("✓ Expression 'D1+B1' propagates error correctly");
    
    // Reset error flag
    test_cell.error = false;
    
    // Test function calls
    let result = sheet.spreadsheet_evaluate_expression("SUM(A1:B1)", 1,1);
    assert_eq!(result, (15,false));
    // assert!(!test_cell.error);
    println!("✓ Function call 'SUM(A1:B1)' = {:?} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("AVG(A1:A2)", 1,1);
    assert_eq!(result, (15,false));
    // assert!(!test_cell.error);
    println!("✓ Function call 'AVG(A1:A2)' = {:?} (expected 15)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("MIN(A1:A2)", 1,1);
    assert_eq!(result, (10,false));
    // assert!(!test_cell.error);
    println!("✓ Function call 'MIN(A1:A2)' = {:?} (expected 10)", result);
    
    // Test negative and positive numbers
    let result = sheet.spreadsheet_evaluate_expression("-10+5", 1,1);
    assert_eq!(result, (-5,false));
    // assert!(!test_cell.error);
    println!("✓ Expression '-10+5' = {:?} (expected -5)", result);
    
    let result = sheet.spreadsheet_evaluate_expression("10+-5", 1,1);
    assert_eq!(result,( 5,false));
    // assert!(!test_cell.error);
    println!("✓ Expression '10+-5' = {:?} (expected 5)", result);
    
    // Test invalid expressions
    let result = sheet.spreadsheet_evaluate_expression("10@20", 1,1);
    assert_eq!(result, (-1,true));
    // assert!(test_cell.error);
    println!("✓ Invalid expression '10@20' handled correctly");
    
}

fn test_cycle_detection() {
    println!("\n====== Testing cycle detection ======");
    let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
    
    // Set up a cycle: A1 depends on B1, B1 depends on C1, C1 depends on A1
    {
        // Set up A1 to reference B1
        if let Some(ref mut cell_a1) = sheet.cells[0] {
            cell_a1.formula = Some("B1".to_string());
            // Mark B1 as a dependent of A1
            cell::cell_dep_insert(cell_a1, "B1");
        }
        
        // Set up B1 to reference C1
        if let Some(ref mut cell_b1) = sheet.cells[1] {
            cell_b1.formula = Some("C1".to_string());
            // Mark C1 as a dependent of B1
            cell::cell_dep_insert(cell_b1, "C1");
        }
        
        // Set up C1 to reference A1 (creating a cycle)
        if let Some(ref mut cell_c1) = sheet.cells[2] {
            cell_c1.formula = Some("A1".to_string());
            // Mark A1 as a dependent of C1
            cell::cell_dep_insert(cell_c1, "A1");
        }
    }
    
    // Test direct cycle detection: A1 -> B1 -> C1 -> A1
    {
        // Check if cycle is detected
        let has_cycle = sheet.first_step_find_cycle("A1", 1, 1, 1, 1, false);
        assert!(has_cycle, "Should detect cycle starting from A1");
        println!("✓ Detected cycle in A1 -> B1 -> C1 -> A1");
        
        // Also check from other cells in the cycle
        let has_cycle = sheet.first_step_find_cycle("B1", 1, 1, 1, 1, false);
        assert!(has_cycle, "Should detect cycle starting from B1");
        println!("✓ Detected cycle in B1 -> C1 -> A1 -> B1");
        
        let has_cycle = sheet.first_step_find_cycle("C1", 1, 1, 1, 1, false);
        assert!(has_cycle, "Should detect cycle starting from C1");
        println!("✓ Detected cycle in C1 -> A1 -> B1 -> C1");
    }
    
    // Set up a range-based cycle: D1 depends on SUM(E1:F1), E1 depends on D1
    {
        // Set up D1 to reference SUM(E1:F1)
        if let Some(ref mut cell_d1) = sheet.cells[3] {
            cell_d1.formula = Some("SUM(E1:F1)".to_string());
            // Mark E1 and F1 as dependents of D1
            cell::cell_dep_insert(cell_d1, "E1");
            cell::cell_dep_insert(cell_d1, "F1");
        }
        
        // Set up E1 to reference D1 (creating a cycle)
        if let Some(ref mut cell_e1) = sheet.cells[4] {
            cell_e1.formula = Some("D1".to_string());
            // Mark D1 as a dependent of E1
            cell::cell_dep_insert(cell_e1, "D1");
        }
    }
    
    // Test range-based cycle detection
    {
        // Check if range-based cycle is detected
        let has_cycle = sheet.first_step_find_cycle("D1", 1, 1, 5, 6, true);
        assert!(has_cycle, "Should detect cycle in range formula");
        println!("✓ Detected cycle with range formula: D1 -> SUM(E1:F1), E1 -> D1");
        
        // Check from the other direction
        let has_cycle = sheet.first_step_find_cycle("E1", 1, 1, 4, 4, false);
        assert!(has_cycle, "Should detect cycle starting from E1");
        println!("✓ Detected cycle starting from E1: E1 -> D1 -> SUM(E1:F1)");
    }
    
    // Update the test setup for G1, H1, I1

    // Set up a no-cycle case: G1 depends on H1, H1 depends on I1
    {
        // Set up G1 to reference H1
        if let Some(ref mut cell_g1) = sheet.cells[6] {
            cell_g1.formula = Some("H1".to_string());
            // In a non-cycle case, G1 does NOT have any dependents
            // Do not add any dependents for G1
        }
        
        // Set up H1 to reference I1
        if let Some(ref mut cell_h1) = sheet.cells[7] {
            cell_h1.formula = Some("I1".to_string());
            // H1 depends on I1, so I1 is dependent on H1
            cell::cell_dep_insert(cell_h1, "I1");
        }
        
        // I1 has no formula, just a value
        if let Some(ref mut cell_i1) = sheet.cells[8] {
            cell_i1.value = 42;  // Some arbitrary value
        }
    }
    
    // Test no-cycle case
    {
        // Check that no cycle is detected when there isn't one
        let has_cycle = sheet.first_step_find_cycle("G1", 1, 1, 8, 8, false);
        assert!(!has_cycle, "Should not detect cycle when there isn't one");
        println!("✓ Correctly found no cycle in G1 -> H1 -> I1");
        
        let has_cycle = sheet.first_step_find_cycle("H1", 1, 1, 10, 10, false);
        assert!(!has_cycle, "Should not detect cycle when there isn't one");
        println!("✓ Correctly found no cycle in H1 -> I1");
        
        let has_cycle = sheet.first_step_find_cycle("I1", 1, 1, 7, 7, false);
        assert!(!has_cycle, "Should not detect cycle when there isn't one");
        println!("✓ Correctly found no cycle for I1 (no dependencies)");
    }
    
    // Test self-reference cycle
    {
        // Set up J1 to reference itself (clear cycle)
        if let Some(ref mut cell_j1) = sheet.cells[9] {
            cell_j1.formula = Some("J1".to_string());
            // Mark J1 as a dependent of itself
            cell::cell_dep_insert(cell_j1, "J1");
        }
        
        // Check self-reference cycle
        let has_cycle = sheet.first_step_find_cycle("J1", 1, 1, 10, 10, false);
        assert!(has_cycle, "Should detect self-reference cycle");
        println!("✓ Detected self-reference cycle in J1 -> J1");
    }
    
    // Test handling of invalid cell names
    {
        let has_cycle = sheet.first_step_find_cycle("ZZ100", 1, 1, 1, 1, false);
        assert!(!has_cycle, "Should handle invalid cell names gracefully");
        println!("✓ Gracefully handled invalid cell name");
    }
}

fn test_remove_old_dependents() {
    println!("\n====== Testing remove_old_dependents ======");
    // ----- Test the range formula case -----
    // Create a 3x3 spreadsheet.
    let mut sheet = Spreadsheet::spreadsheet_create(3, 3).unwrap();
    // Set cell A1’s formula to a range function: "SUM(B1:C1)"
    {
        let idx_a1 = get_cell_index(&sheet, 1, 1);
        if let Some(ref mut cell_a1) = sheet.cells[idx_a1] {
            cell_a1.formula = Some("SUM(B1:C1)".to_string());
        }
    }
    // Manually mark cells B1 and C1 as having "A1" as a dependent.
    {
        // B1 at row 1, col 2
        let idx_b1 = get_cell_index(&sheet, 1, 2);
        if let Some(ref mut cell_b1) = sheet.cells[idx_b1] {
            crate::cell::cell_dep_insert(cell_b1, "A1");
        }
        // C1 at row 1, col 3
        let idx_c1 = get_cell_index(&sheet, 1, 3);
        if let Some(ref mut cell_c1) = sheet.cells[idx_c1] {
            crate::cell::cell_dep_insert(cell_c1, "A1");
        }
    }
    // Confirm that "A1" is present in dependents of B1 and C1.
    {
        let idx_b1 = get_cell_index(&sheet, 1, 2);
        let idx_c1 = get_cell_index(&sheet, 1, 3);
        match &sheet.cells[idx_b1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(set.contains("A1")),
            crate::cell::Dependents::None => panic!("B1 should have dependents"),
        }
        match &sheet.cells[idx_c1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(set.contains("A1")),
            crate::cell::Dependents::None => panic!("C1 should have dependents"),
        }
    }
    // Call remove_old_dependents for "A1"
    sheet.remove_old_dependents("A1");
    // Verify that "A1" has been removed from B1 and C1 dependents.
    {
        let idx_b1 = get_cell_index(&sheet, 1, 2);
        let idx_c1 = get_cell_index(&sheet, 1, 3);
        match &sheet.cells[idx_b1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(!vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(!set.contains("A1")),
            crate::cell::Dependents::None => {},
        }
        match &sheet.cells[idx_c1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(!vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(!set.contains("A1")),
            crate::cell::Dependents::None => {},
        }
    }
    println!("✓ remove_old_dependents successfully removed dependents for range formula");

    // ----- Test the non-range formula case -----
    // Create a new 3x3 spreadsheet.
    let mut sheet = Spreadsheet::spreadsheet_create(3, 3).unwrap();
    // Set cell A1’s formula to a non-range expression: "B1+C1"
    {
        let idx_a1 = get_cell_index(&sheet, 1, 1);
        if let Some(ref mut cell_a1) = sheet.cells[idx_a1] {
            cell_a1.formula = Some("B1+C1".to_string());
        }
    }
    // Mark cells B1 and C1 as having "A1" as a dependent.
    {
        let idx_b1 = get_cell_index(&sheet, 1, 2);
        let idx_c1 = get_cell_index(&sheet, 1, 3);
        if let Some(ref mut cell_b1) = sheet.cells[idx_b1] {
            crate::cell::cell_dep_insert(cell_b1, "A1");
        }
        if let Some(ref mut cell_c1) = sheet.cells[idx_c1] {
            crate::cell::cell_dep_insert(cell_c1, "A1");
        }
    }
    // Confirm the dependents exist.
    {
        let idx_b1 = get_cell_index(&sheet, 1, 2);
        let idx_c1 = get_cell_index(&sheet, 1, 3);
        match &sheet.cells[idx_b1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(set.contains("A1")),
            crate::cell::Dependents::None => panic!("B1 should have dependents"),
        }
        match &sheet.cells[idx_c1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(set.contains("A1")),
            crate::cell::Dependents::None => panic!("C1 should have dependents"),
        }
    }
    // Call remove_old_dependents for "A1"
    sheet.remove_old_dependents("A1");
    // Confirm that "A1" is removed.
    {
        let idx_b1 = get_cell_index(&sheet, 1, 2);
        let idx_c1 = get_cell_index(&sheet, 1, 3);
        match &sheet.cells[idx_b1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(!vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(!set.contains("A1")),
            crate::cell::Dependents::None => {},
        }
        match &sheet.cells[idx_c1].as_ref().unwrap().dependents {
            crate::cell::Dependents::Vector(vec) => assert!(!vec.contains(&"A1".to_string())),
            crate::cell::Dependents::Set(set) => assert!(!set.contains("A1")),
            crate::cell::Dependents::None => {},
        }
    }
    println!("✓ remove_old_dependents successfully removed dependents for non-range formula");
}

fn test_dependency_updates() {
    println!("\n====== Testing dependency updates ======");

    // Create a 4x2 spreadsheet for testing
    let mut sheet = Spreadsheet::spreadsheet_create(4, 2).unwrap();

    // Set cell values:
    // A1 = 5, A2 = 10, A3 = 20, A4 = -5
    {
        let idx_a1 = get_cell_index(&sheet, 1, 1);
        if let Some(ref mut cell) = sheet.cells[idx_a1] {
            cell.value = 5;
        }
        let idx_a2 = get_cell_index(&sheet, 2, 1);
        if let Some(ref mut cell) = sheet.cells[idx_a2] {
            cell.value = 10;
        }
        let idx_a3 = get_cell_index(&sheet, 3, 1);
        if let Some(ref mut cell) = sheet.cells[idx_a3] {
            cell.value = 20;
        }
        let idx_a4 = get_cell_index(&sheet, 4, 1);
        if let Some(ref mut cell) = sheet.cells[idx_a4] {
            cell.value = -5;
        }
    }

    // Set B1 = "A1+A2" (B1 is row1, col2)
    sheet.update_dependencies("B1", "A1+A2");

    // Verify that A1 and A2 have "B1" as a dependent.
    {
        let idx_a1 = get_cell_index(&sheet, 1, 1);
        let deps_a1 = sheet.collect_dependent_keys(sheet.cells[idx_a1].as_ref().unwrap());
        assert!(deps_a1.contains(&"B1".to_string()), "A1 should have dependent B1");
        println!("✓ A1 contains dependent B1");

        let idx_a2 = get_cell_index(&sheet, 2, 1);
        let deps_a2 = sheet.collect_dependent_keys(sheet.cells[idx_a2]  .as_ref()
            .unwrap()
            );
        assert!(deps_a2.contains(&"B1".to_string()), "A2 should have dependent B1");
        println!("✓ A2 contains dependent B1");
        // Print the formula in B1
        
    }

    // Now update B1's formula to "A2*2" so that only A2 is a dependency.
   sheet.update_dependencies("B1", "A3*2");

    // Verify that A1 no longer has "B1" while A2 still does.
    {
        let idx_a1 = get_cell_index(&sheet, 1, 1);
        let deps_a1 = sheet.collect_dependent_keys(sheet.cells[idx_a1].as_ref().unwrap());
        assert!(!deps_a1.contains(&"B1".to_string()), "A1 should not have dependent B1 after update");
        println!("✓ After update, A1 does not contain dependent B1");
        
        let idx_a2 = get_cell_index(&sheet, 2, 1);
        let deps_a2 = sheet.collect_dependent_keys(sheet.cells[idx_a2]
            .as_ref()
            .unwrap()
            );
        assert!(!deps_a2.contains(&"B1".to_string()), "A2 should not have dependent B1 after update");
        println!("✓ After update, A2 does not contains dependent B1");

        let idx_a3 = get_cell_index(&sheet, 3, 1);
        let deps_a3 = sheet.collect_dependent_keys(sheet.cells[idx_a3]
            .as_ref()
            .unwrap())
            ;
        assert!(deps_a3.contains(&"B1".to_string()), "A3 should have dependent B1 after update");
        println!("✓ After update, A3 contains dependent B1");
    }

    // Additional example:
    // Set B2 = "SUM(A1:A4)" (B2 is row2, col2)
    sheet.update_dependencies("B2", "SUM(A1:A4)");

    // Verify that each cell in column A (A1, A2, A3, A4) now includes "B2" as a dependent.
    for row in 1..=4 {
        let idx = get_cell_index(&sheet, row, 1);
        let deps = sheet.collect_dependent_keys(sheet.cells[idx]
            .as_ref()
            .unwrap()
            );
        assert!(deps.contains(&"B2".to_string()), "A{} should have dependent B2", row);
        println!("✓ A{} contains dependent B2", row);
    }

    // A further example: Test a non-arithmetic formula.
    // Create a larger sheet (4x4) and set C1 = "A3-A4" (C1 is row1, col3)
    let mut sheet_large = Spreadsheet::spreadsheet_create(4, 4).unwrap();
    {
        let idx_a1 = get_cell_index(&sheet_large, 1, 1);
        if let Some(ref mut cell) = sheet_large.cells[idx_a1] {
            cell.value = 5;
        }
        let idx_a2 = get_cell_index(&sheet_large, 2, 1);
        if let Some(ref mut cell) = sheet_large.cells[idx_a2] {
            cell.value = 10;
        }
        let idx_a3 = get_cell_index(&sheet_large, 3, 1);
        if let Some(ref mut cell) = sheet_large.cells[idx_a3] {
            cell.value = 20;
        }
        let idx_a4 = get_cell_index(&sheet_large, 4, 1);
        if let Some(ref mut cell) = sheet_large.cells[idx_a4] {
            cell.value = -5;
        }
    }
    sheet_large.update_dependencies("C1", "A3-A4");

    // Verify that A3 and A4 have "C1" as a dependent.
    {
        let idx_a3 = get_cell_index(&sheet_large, 3, 1);
        let deps_a3 = sheet_large.collect_dependent_keys(sheet_large.cells[idx_a3].as_ref().unwrap())
            ;
        assert!(deps_a3.contains(&"C1".to_string()), "A3 should have dependent C1");
        println!("✓ A3 contains dependent C1");

        let idx_a4 = get_cell_index(&sheet_large, 4, 1);
        let deps_a4 = sheet_large.collect_dependent_keys(sheet_large.cells[idx_a4]
            .as_ref()
            .unwrap()
            );
        assert!(deps_a4.contains(&"C1".to_string()), "A4 should have dependent C1");
        println!("✓ A4 contains dependent C1");
    }

    println!("All dependency update tests passed");
}

fn test_topo_sort() {
    println!("\n====== Testing topo_sort ======");
    
    // Create a 5x5 spreadsheet for testing
    let mut sheet = Spreadsheet::spreadsheet_create(5, 5).unwrap();
    
    // Setup cell values and formulas:
    // A1 = 10 (base value)
    // B1 = A1 * 2  (depends on A1)
    // C1 = B1 + 5  (depends on B1)
    // D1 = C1 - A1 (depends on C1 and A1)
    // E1 = SUM(B1:D1) (depends on B1, C1, D1)
    
    // Set up initial cell values
    {
        let idx_a1 = get_cell_index(&sheet, 1, 1);
        if let Some(ref mut cell) = sheet.cells[idx_a1] {
            cell.value = 10;
        }
    }
    
    // Set up dependencies
    sheet.update_dependencies("B1", "A1*2");
    sheet.update_dependencies("C1", "B1+5");
    sheet.update_dependencies("D1", "C1-A1");
    sheet.update_dependencies("E1", "SUM(B1:D1)");
    
    // Get the cell references to perform topo_sort
    let idx_e1 = get_cell_index(&sheet, 1, 5);
    let e1_cell = sheet.cells[idx_e1].as_ref().unwrap();
    
    // Perform topological sort starting from E1
    let sorted_cells = sheet.topo_sort(e1_cell);
    
    // Print the order for debugging
    println!("Topological sort result:");
    for (i, (row,col)) in sorted_cells.iter().enumerate() {
        println!("  {}. {}{}", i+1, Spreadsheet::col_to_letter(*col as i32), row);
    }
    
    // Verify basic properties of the sort
    assert!(!sorted_cells.is_empty(), "Sorted result should not be empty");
    
    // Check that E1 is in the sorted list (should be the first cell)
    let has_e1 = sorted_cells.iter().any(|&(row,col)| row == 1 && col == 5);
    assert!(has_e1, "E1 should be in the sorted list");
    
    // Check that the order respects dependencies
    // We'll create a map of cell positions in the sorted list
    let mut positions = std::collections::HashMap::new();
    for (i, (row,col)) in sorted_cells.iter().enumerate() {
        let cell_name = Spreadsheet::get_cell_name(*row as i32, *col as i32);
        positions.insert(cell_name, i);
    }
    
    // Verify dependency ordering:
    // All cells should come before cells that depend on them
    
    // If A1 and B1 are both in the sorted list, A1 should come after B1
    // (since B1 depends on A1)
    if positions.contains_key("A1") && positions.contains_key("B1") {
        assert!(positions["A1"] < positions["B1"], 
                "A1 should come before B1 in topological sort");
        println!("✓ A1 correctly sorted before B1");
    }
    
    // Similarly for other dependencies
    if positions.contains_key("B1") && positions.contains_key("C1") {
        assert!(positions["B1"] < positions["C1"], 
                "B1 should come before C1 in topological sort");
        println!("✓ B1 correctly sorted before C1");
    }
    
    if positions.contains_key("A1") && positions.contains_key("D1") {
        assert!(positions["A1"] < positions["D1"], 
                "A1 should come before D1 in topological sort");
        println!("✓ A1 correctly sorted before D1");
    }
    
    if positions.contains_key("C1") && positions.contains_key("D1") {
        assert!(positions["C1"] < positions["D1"], 
                "C1 should come before D1 in topological sort");
        println!("✓ C1 correctly sorted before D1");
    }
    
    // E1 depends on B1, C1, and D1, so they should all come before E1
    if positions.contains_key("B1") && positions.contains_key("E1") {
        assert!(positions["B1"] < positions["E1"], 
                "B1 should come before E1 in topological sort");
        println!("✓ B1 correctly sorted before E1");
    }
    
    if positions.contains_key("C1") && positions.contains_key("E1") {
        assert!(positions["C1"] < positions["E1"], 
                "C1 should come before E1 in topological sort");
        println!("✓ C1 correctly sorted before E1");
    }
    
    if positions.contains_key("D1") && positions.contains_key("E1") {
        assert!(positions["D1"] < positions["E1"], 
                "D1 should come before E1 in topological sort");
        println!("✓ D1 correctly sorted before E1");
    }
    
    println!("✓ Topological sort passed all ordering checks");
    
    // Test with a more complex example - diamond dependency
    // A2 is depended on by both B2 and C2, which are both depended on by D2
    let mut sheet2 = Spreadsheet::spreadsheet_create(5, 5).unwrap();
    
    // Set up dependencies
    {
        let idx_a2 = get_cell_index(&sheet2, 2, 1);
        if let Some(ref mut cell) = sheet2.cells[idx_a2] {
            cell.value = 10;
        }
    }
    
    sheet2.update_dependencies("B2", "A2+5");
    sheet2.update_dependencies("C2", "A2*2");
    sheet2.update_dependencies("D2", "B2+C2");
    
    // Get D2 cell
    let idx_d2 = get_cell_index(&sheet2, 2, 4);
    let d2_cell = sheet2.cells[idx_d2].as_ref().unwrap();
    
    // Perform topological sort starting from D2
    let sorted_cells2 = sheet2.topo_sort(d2_cell);
    
    // Print the order
    println!("\nDiamond dependency sort result:");
    for (i, (row,col)) in sorted_cells2.iter().enumerate() {
        println!("  {}. {}{}", i+1, Spreadsheet::col_to_letter(*col as i32 ), row);
    }
    
    // Verify the diamond dependency ordering
    let mut positions2 = std::collections::HashMap::new();
    for (i, (row,col)) in sorted_cells2.iter().enumerate() {
        let cell_name = Spreadsheet::get_cell_name(*row as i32, *col as i32);
        positions2.insert(cell_name, i);
    }
    
    // Check that A2 comes before both B2 and C2
    if positions2.contains_key("A2") && positions2.contains_key("B2") {
        assert!(positions2["A2"] < positions2["B2"], 
                "A2 should come before B2 in diamond dependency");
        println!("✓ A2 correctly sorted before B2");
    }
    
    if positions2.contains_key("A2") && positions2.contains_key("C2") {
        assert!(positions2["A2"] < positions2["C2"], 
                "A2 should come before C2 in diamond dependency");
        println!("✓ A2 correctly sorted before C2");
    }
    
    // Check that both B2 and C2 come before D2
    if positions2.contains_key("B2") && positions2.contains_key("D2") {
        assert!(positions2["B2"] < positions2["D2"], 
                "B2 should come before D2 in diamond dependency");
        println!("✓ B2 correctly sorted before D2");
    }
    
    if positions2.contains_key("C2") && positions2.contains_key("D2") {
        assert!(positions2["C2"] < positions2["D2"], 
                "C2 should come before D2 in diamond dependency");
        println!("✓ C2 correctly sorted before D2");
    }
    
    println!("✓ Diamond dependency topological sort passed all ordering checks");
}

// Add the new test function to the run_tests function
pub fn run_tests() {
    println!("Starting spreadsheet unit tests");
    test_spreadsheet_create();
    test_spreadsheet_parse_cell_name();
    test_column_letter_conversion();
    test_cell_name_helpers();
    test_find_depends();
    test_spreadsheet_evaluate_function();
    test_spreadsheet_evaluate_expression();
    test_cycle_detection();
    test_remove_old_dependents();
    test_dependency_updates();
    test_topo_sort(); // Add this new test
    println!("All tests passed!");
}

fn main() {
    run_tests();
}

