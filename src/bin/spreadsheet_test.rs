#![cfg(not(tarpaulin_include))]
use cop::cell::{Cell, cell_contains, cell_create, cell_dep_insert, cell_dep_remove};
use cop::spreadsheet::{FunctionName, Operand, ParsedRHS, Spreadsheet};
use std::collections::BTreeSet;
use std::time::Instant;

#[cfg(test)]

mod spreadsheet_tests {
    use cop::cell;

    use super::*;

    #[test]
    fn test_spreadsheet_creation() {
        // Test creating a spreadsheet with valid dimensions
        let sheet = Spreadsheet::spreadsheet_create(10, 10);
        assert!(sheet.is_some());
        let sheet = sheet.unwrap();
        assert_eq!(sheet.rows, 10);
        assert_eq!(sheet.cols, 10);
        assert_eq!(sheet.cells.len(), 100);

        
        // Test cells initialization
        for r in 1..=10 {
            for c in 1..=10 {
                let index = ((r - 1) as usize * 10 + (c - 1) as usize) as usize;
                assert!(sheet.cells[index].is_some());
            }
        }
        // Test creating a spreadsheet with larger dimensions
        let sheet = Spreadsheet::spreadsheet_create(999, 18278);
        assert!(sheet.is_some());
        let sheet = sheet.unwrap();
        assert_eq!(sheet.rows, 999);
        assert_eq!(sheet.cols, 18278);

        // Test cells initialization
        for r in 1..=999 {
            for c in 1..=18278 {
                let index = ((r - 1) as usize * 10 + (c - 1) as usize) as usize;
                assert!(sheet.cells[index].is_some());
            }
        }

    }

    #[test]
    fn test_column_letter_conversion() {
        // Test converting column numbers to letters
        assert_eq!(Spreadsheet::col_to_letter(1), "A");
        assert_eq!(Spreadsheet::col_to_letter(26), "Z");
        assert_eq!(Spreadsheet::col_to_letter(27), "AA");
        assert_eq!(Spreadsheet::col_to_letter(52), "AZ");
        assert_eq!(Spreadsheet::col_to_letter(53), "BA");
        assert_eq!(Spreadsheet::col_to_letter(702), "ZZ");
        assert_eq!(Spreadsheet::col_to_letter(703), "AAA");
        assert_eq!(Spreadsheet::col_to_letter(704), "AAB");
        assert_eq!(Spreadsheet::col_to_letter(728), "AAZ");
        assert_eq!(Spreadsheet::col_to_letter(729), "ABA");
        assert_eq!(Spreadsheet::col_to_letter(1404), "BAZ");
        assert_eq!(Spreadsheet::col_to_letter(18278), "ZZZ");

        // Test converting letters to column numbers
        assert_eq!(Spreadsheet::letter_to_col("A"), 1);
        assert_eq!(Spreadsheet::letter_to_col("Z"), 26);
        assert_eq!(Spreadsheet::letter_to_col("AA"), 27);
        assert_eq!(Spreadsheet::letter_to_col("AZ"), 52);
        assert_eq!(Spreadsheet::letter_to_col("BA"), 53);
        assert_eq!(Spreadsheet::letter_to_col("ZZ"), 702);
        assert_eq!(Spreadsheet::letter_to_col("AAA"), 703);
        assert_eq!(Spreadsheet::letter_to_col("AAB"), 704);
        assert_eq!(Spreadsheet::letter_to_col("AAZ"), 728);
        assert_eq!(Spreadsheet::letter_to_col("ABA"), 729);
        assert_eq!(Spreadsheet::letter_to_col("BAZ"), 1404);
        assert_eq!(Spreadsheet::letter_to_col("ZZZ"), 18278);
        
        // Test round-trip conversion
        for col in 1..=100 {
            let letter = Spreadsheet::col_to_letter(col);
            let back = Spreadsheet::letter_to_col(&letter);
            assert_eq!(col, back);
        }
    }

    #[test]
    fn test_cell_name_operations() {
        // Test generating cell names
        assert_eq!(Spreadsheet::get_cell_name(1, 1), "A1");
        assert_eq!(Spreadsheet::get_cell_name(10, 26), "Z10");
        assert_eq!(Spreadsheet::get_cell_name(100, 27), "AA100");
        assert_eq!(Spreadsheet::get_cell_name(100, 100), "CV100");

        // Test parsing cell names in a spreadsheet context
        let sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();

        // Valid cell names
        assert_eq!(sheet.spreadsheet_parse_cell_name("A1"), Some((1, 1)));
        assert_eq!(sheet.spreadsheet_parse_cell_name("Z10"), Some((10, 26)));
        assert_eq!(sheet.spreadsheet_parse_cell_name("AA100"), Some((100, 27)));
        assert_eq!(sheet.spreadsheet_parse_cell_name("CV100"), Some((100, 100)));

        // Invalid cell names (out of bounds)
        assert_eq!(sheet.spreadsheet_parse_cell_name("A0"), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name("A101"), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name("CW1"), None); // Column out of bounds

        // Malformed cell names
        assert_eq!(sheet.spreadsheet_parse_cell_name("1A"), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name("A1B"), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name(""), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name("A"), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name("1"), None);
    }

    #[test]
    fn test_is_numeric() {
        // Test numeric strings
        assert!(Spreadsheet::is_numeric("0"));
        assert!(Spreadsheet::is_numeric("123"));
        assert!(Spreadsheet::is_numeric("9876543210"));
        assert!(Spreadsheet::is_numeric("00"));
        assert!(Spreadsheet::is_numeric("0098"));
        assert!(Spreadsheet::is_numeric("0090"));

        // Test non-numeric strings
        assert!(!Spreadsheet::is_numeric(""));
        assert!(!Spreadsheet::is_numeric("A"));
        assert!(!Spreadsheet::is_numeric("12A"));
        assert!(!Spreadsheet::is_numeric("A12"));
        assert!(!Spreadsheet::is_numeric("-123")); // Contains non-digit
        assert!(!Spreadsheet::is_numeric("+123")); // Contains non-digit
        assert!(!Spreadsheet::is_numeric("12.3")); // Contains non-digit
        assert!(!Spreadsheet::is_numeric("-0090")); // Contains non-digit
    }

    #[test]
    fn test_evaluate_expression() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Setup some cell values
        let a1_idx = 0 * 10 + 0;
        let a2_idx = 1 * 10 + 0;
        let b1_idx = 0 * 10 + 1;
        let b2_idx = 1 * 10 + 1;
        let c1_idx = 0 * 10 + 2;
        let d1_idx = 0 * 10 + 3;
        let d2_idx = 0 * 10 + 4;
        let d3_idx = 0 * 10 + 5;

        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.value = 10;
        }
        if let Some(cell) = sheet.cells[a2_idx].as_mut() {
            cell.value = 20;
        }
        if let Some(cell) = sheet.cells[b1_idx].as_mut() {
            cell.value = 30;
        }
        if let Some(cell) = sheet.cells[b2_idx].as_mut() {
            cell.value = 40;
        }
        if let Some(cell) = sheet.cells[c1_idx].as_mut() {
            cell.value = 123;
        }
        if let Some(cell) = sheet.cells[d1_idx].as_mut() {
            cell.value = -234;
        }
        if let Some(cell) = sheet.cells[d2_idx].as_mut(){
            cell.value = 2;
            cell.error = true;
        }
        if let Some(cell) = sheet.cells[d3_idx].as_mut(){
            cell.value = 2;
            cell.error = false;
        }


        // Test various expressions similar to the C tests
        
        // Test with numeric literals
        let expr = ParsedRHS::SingleValue(Operand::Number(42));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, 42);
        assert!(!error);

        // Test with positive and negative numbers
        let expr = ParsedRHS::SingleValue(Operand::Number(-1));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, -1);
        assert!(!error);

        let expr = ParsedRHS::SingleValue(Operand::Number(1));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, 1);
        assert!(!error);

        let expr = ParsedRHS::SingleValue(Operand::Number(09));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, 9);
        assert!(!error);


        let expr = ParsedRHS::SingleValue(Operand::Number(-09));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, -9);
        assert!(!error);

        let expr = ParsedRHS::SingleValue(Operand::Number(-2_147_483_648));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, -2147483648);
        assert!(!error);

        let expr = ParsedRHS::SingleValue(Operand::Number(2_147_483_647));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, 2147483647);
        assert!(!error);

        // Test with cell references
        let expr = ParsedRHS::SingleValue(Operand::Cell(1, 1)); // A1
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 2, 2);
        assert_eq!(value, 10);
        assert!(!error);

        let expr = ParsedRHS::SingleValue(Operand::Cell(1, 4)); // A4
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 2, 2);
        assert_eq!(value, -234);
        assert!(!error);

        let expr = ParsedRHS::SingleValue(Operand::Cell(1, 5)); // A4
        let (_, error) = sheet.spreadsheet_evaluate_expression(&expr, 2, 2);
        assert!(error);

        // Test for none formula i.e. default cells
        let expr = ParsedRHS::None;
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 2, 2);
        assert_eq!(value, 0);
        assert!(!error);

        // Test basic arithmetic
        let expr = ParsedRHS::Arithmetic {
            lhs: Operand::Cell(1, 1), // A1 = 10
            operator: '+',
            rhs: Operand::Cell(2, 1), // A2 = 20
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 2);
        assert_eq!(value, 30);
        assert!(!error);

        let expr = ParsedRHS::Arithmetic { lhs: Operand::Cell(1,5), operator: '+', rhs: Operand::Cell(1, 1) }; // A5 = ERR
        let (_, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 2);
        assert!(error);

        // Test arithmetic with constants
        let expr = ParsedRHS::Arithmetic {
            lhs: Operand::Number(3),
            operator: '*',
            rhs: Operand::Cell(1, 2), // B1 = 30
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 2);
        assert_eq!(value, 90);
        assert!(!error);

        let expr = ParsedRHS::Arithmetic {
            lhs: Operand::Cell(1, 3), // C1 = 123
            operator: '-',
            rhs: Operand::Number(34),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 2);
        assert_eq!(value, 89);
        assert!(!error);

        // Test division and error propagation
        let expr = ParsedRHS::Arithmetic {
            lhs: Operand::Number(3),
            operator: '/',
            rhs: Operand::Cell(1, 1), // A1 = 10
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 2);
        assert_eq!(value, 0); // Integer division
        assert!(!error);

        let expr = ParsedRHS::Arithmetic {
            lhs: Operand::Number(10),
            operator: '/',
            rhs: Operand::Number(0),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 2);
        assert_eq!(value, 0);
        assert!(error); // Division by zero error

        // Test propagation of errors
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.error = true;
        }

        let expr = ParsedRHS::SingleValue(Operand::Cell(1, 1)); // A1 (with error)
        let (_, error) = sheet.spreadsheet_evaluate_expression(&expr, 2, 2);
        assert!(error);

        // Test with range functions
        // First, reset A1's error flag
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.error = false;
        }

        // Test SUM function
        let expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(2, 2), // B2 = 40
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 100); // 10 + 20 + 30 + 40
        assert!(!error);

        // Test MIN function
        let expr = ParsedRHS::Function {
            name: FunctionName::Min,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(2, 2), // B2 = 40
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 10);
        assert!(!error);

        // Test MAX function
        let expr = ParsedRHS::Function {
            name: FunctionName::Max,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(2, 2), // B2 = 40
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 40);
        assert!(!error);

        // Test AVG function
        let expr = ParsedRHS::Function {
            name: FunctionName::Avg,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(2, 2), // B2 = 40
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 25); // (10 + 20 + 30 + 40) / 4
        assert!(!error);

        // Range can be single cell as well
        let expr = ParsedRHS::Function {
            name: FunctionName::Avg,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(1, 1), // A1 = 10
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 10); // (10) / 1
        assert!(!error);

        //Test STDDEV function 
        let expr = ParsedRHS::Function {
            name: FunctionName::Stdev,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(2, 2), // B2 = 40
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 11); // Standard deviation of the values
        assert!(!error);

        let expr = ParsedRHS::Function {
            name: FunctionName::Stdev,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(1, 1), // A1 = 10
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 0); // Standard deviation returns 0 for a single value
        assert!(!error);

        // Test COPY function 
        let expr = ParsedRHS::Function {
            name: FunctionName::Copy,
            args: (
                Operand::Cell(1, 1), // A1 = 10
                Operand::Cell(2, 2), // B2 = 40
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 0); // Copy is not implemented in this function. it should simply return 0,false
        assert!(!error); // Copy function should return a default error

        // Test error propagation with functions
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.error = true;
        }

        let expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1 (with error)
                Operand::Cell(2, 2), // B2
            ),
        };
        let (_, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert!(error);

        // Test for sleep function with error
        let expr = ParsedRHS::Sleep 
        (
            Operand::Cell(1, 5), // A5 = 2
        )
        ;
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, 2); // Sleep function should return cell value
        assert!(error); // Sleep function should return an error
        assert!(duration.as_secs() == 0); // Sleep function should sleep for 0 seconds

        // Test for sleep function without error
        let expr = ParsedRHS::Sleep 
        (
            Operand::Cell(1, 6), // A6 = 2
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, 2); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 2); // Sleep function should sleep for 2 seconds

        // test for sleep with cell with negative value
        //A4 is negative value
        let expr = ParsedRHS::Sleep 
        (
            Operand::Cell(1, 4), // A4 = -234
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, -234); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 0); // Sleep function should sleep for 0 seconds

        // Sleep with value as argument
        let expr = ParsedRHS::Sleep 
        (
            Operand::Number(2), // Sleep for 2 seconds
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, 2); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 2); // Sleep function should sleep for 2 seconds

        // Test for sleep with negative value
        let expr = ParsedRHS::Sleep 
        (
            Operand::Number(-2), // Sleep for -2 seconds
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, -2); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 0); // Sleep function should sleep for 0 seconds

        // Test for sleep with zero value
        let expr = ParsedRHS::Sleep 
        (
            Operand::Number(0), // Sleep for 0 seconds
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, 0); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 0); // Sleep function should sleep for 0 seconds


    }

    #[test]
    fn test_cycle_detection() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Set up a cyclic dependency: A1 depends on B1, B1 depends on A1
        let a1_idx = 0 * 10 + 0;
        let b1_idx = 0 * 10 + 1;

        // First, modify A1 to depend on B1
        if let Some(cell_a1) = sheet.cells[a1_idx].as_mut() {
            cell_dep_insert(cell_a1, 1, 2);
        }

        // Then, modify B1 to depend on A1 (in a separate step)
        if let Some(cell_b1) = sheet.cells[b1_idx].as_mut() {
            cell_dep_insert(cell_b1, 1, 1);
        }

        // Test cycle detection
        assert!(sheet.first_step_find_cycle(1, 1, 1, 0, 2, 0, false));
        assert!(sheet.first_step_find_cycle(1, 2, 1, 0, 1, 0, false));

        // Test no cycle
        let c1_idx = 0 * 10 + 2;
        let c2_idx = 1 * 10 + 2;

        // First, modify C1 to depend on C2
        if let Some(cell_c1) = sheet.cells[c1_idx].as_mut() {
            cell_dep_insert(cell_c1, 2, 3);
        }

        // Test no cycle detection
        // assert!(!sheet.first_step_find_cycle(1, 3, 2, 0, 3, 0, false));
    }

    #[test]
    fn test_dependencies_management() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Setup A1's formula to depend on B1 and C1
        let a1_idx = 0 * 10 + 0;
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.formula = ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 2), // B1
                operator: '+',
                rhs: Operand::Cell(1, 3), // C1
            };
        }

        // Now update the dependencies
        sheet.update_dependencies(1, 1, 1, 2, 1, 3, false);

        // Check if dependencies were correctly set up
        let b1_idx = 0 * 10 + 1;
        let c1_idx = 0 * 10 + 2;

        if let Some(cell_b1) = sheet.cells[b1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_b1);
            assert!(deps.contains(&(1, 1)));
        }

        if let Some(cell_c1) = sheet.cells[c1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_c1);
            assert!(deps.contains(&(1, 1)));
        }

        // Now change A1's formula to depend only on D1
        sheet.remove_old_dependents(1, 1);

        // Set the new formula for A1
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.formula = ParsedRHS::SingleValue(Operand::Cell(1, 4)); // D1
        }

        sheet.update_dependencies(1, 1, 1, 4, 0, 0, false);

        // Check that old dependencies were removed
        if let Some(cell_b1) = sheet.cells[b1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_b1);
            println!("B1 dependencies: {:?}", deps);
            assert!(!deps.contains(&(1, 1)));
        }

        if let Some(cell_c1) = sheet.cells[c1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_c1);
            assert!(!deps.contains(&(1, 1)));
        }

        // Check that new dependency was added
        let d1_idx = 0 * 10 + 3;
        if let Some(cell_d1) = sheet.cells[d1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_d1);
            assert!(deps.contains(&(1, 1)));
        }

        // Test for larger spreadsheet
        let mut sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();
        // set A1 to 5 
        let a1_idx = 0 * 100 + 0;
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.value = 5;
        }
        // set A2 to 10 
        let a2_idx = 1 * 100 + 0;
        if let Some(cell) = sheet.cells[a2_idx].as_mut() {
            cell.value = 10;
        }

        //set A3 to 20
        let a3_idx = 2 * 100 + 0;
        if let Some(cell) = sheet.cells[a3_idx].as_mut() {
            cell.value = 20;
        }
         // set A4 to -5
        let a4_idx = 3 * 100 + 0;
        if let Some(cell) = sheet.cells[a4_idx].as_mut() {
            cell.value = -5;
        }

        //  set B1 to be equal to A1 + A2 using set_cell_value
        sheet.update_dependencies(1, 2, 1, 1, 2, 1, false);

        // assign A1 to 20 using formula
        sheet.update_dependencies(1, 1, 0, 0, 0, 0, false);
         // assign C1 to B1 * 2
        
        // update dependencies for C1
        sheet.update_dependencies(1, 3, 1, 2, 0, 0, false);

        // update dependencies for A2
        sheet.update_dependencies(2, 1, 0, 0, 0, 0, false);

        // set D1 = MAX(A1:A4)
        // set D2 = MIN(A1:A4)
        // set D3 = SUM(A1:A4)
        // set D4= AVG(A1:A4)
        // set D5 = STDDEV(A1:A4)
        // set E1= SLEEP(A1)
        // set E2= SLEEP(A4)

        // update dependencies for D1
        sheet.update_dependencies(1, 4, 1, 1, 4, 1, true);
        // update dependencies for D2
        sheet.update_dependencies(2, 4, 1, 1, 4, 1, true);
        // update dependencies for D3
        sheet.update_dependencies(3, 4, 1, 1, 4, 1, true);
        // update dependencies for D4
        sheet.update_dependencies(4, 4, 1, 1, 4, 1, true);
        // update dependencies for D5
        sheet.update_dependencies(5, 4, 1, 1, 4, 1, true);
        // update dependencies for E1
        sheet.update_dependencies(1, 5, 1, 1, 0, 0, false);
        // update dependencies for E2
        sheet.update_dependencies(2, 5, 4, 1, 0, 0, false);
        // Check that dependencies were correctly set up
        // B1 D1-D5 E1 should be present in dependents of A1

        let a1_idx = 0 * 10 + 0;
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a1);
            assert!(deps.contains(&(1, 2)));
            assert!(deps.contains(&(1, 4)));
            assert!(deps.contains(&(2, 4)));
            assert!(deps.contains(&(3, 4)));
            assert!(deps.contains(&(4, 4)));
            assert!(deps.contains(&(5, 4)));
            assert!(deps.contains(&(1, 5)));
        }

        // dependencies of B1 should contain C1
        let b1_idx = 0 * 10 + 1;
        if let Some(cell_b1) = sheet.cells[b1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_b1);
            assert!(deps.contains(&(1, 3)));
        }

        
    }

    #[test]
    fn test_topological_sort() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
        // Setup:
        // A1 depends on B1
        // B1 depends on C1
        // C1 has no dependencies
        let a1_idx = 0 * 10 + 0;
        let b1_idx = 0 * 10 + 1;
        let c1_idx = 0 * 10 + 2;

        if let Some(cell_a1) = sheet.cells[a1_idx].as_mut() {
            cell_dep_insert(cell_a1, 1, 2); // A1 points to B1
        }

        if let Some(cell_b1) = sheet.cells[b1_idx].as_mut() {
            cell_dep_insert(cell_b1, 1, 3); // B1 points to C1
        }

        // Perform topological sort starting from A1
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let sorted = sheet.topo_sort(cell_a1);

            // Check that the order is correct (should be C1, B1, A1)
            println!("Sorted order: {:?}", sorted);
            assert_eq!(sorted.len(), 3);
            assert_eq!(sorted[0], (1, 1)); // C1
            assert_eq!(sorted[1], (1, 2)); // B1
            assert_eq!(sorted[2], (1, 3)); // A1
        }
    }

    #[test]
    fn test_set_cell_value() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
        let mut status = String::new();

        // Test setting a simple value
        let val_expr = ParsedRHS::SingleValue(Operand::Number(42));
        sheet.spreadsheet_set_cell_value(1, 1, val_expr, &mut status);
        assert_eq!(status, "ok");

        let a1_idx = 0 * 10 + 0;
        if let Some(cell) = sheet.cells[a1_idx].as_ref() {
            assert_eq!(cell.value, 42);
            assert!(!cell.error);
        }

        // Test setting a cell reference
        let ref_expr = ParsedRHS::SingleValue(Operand::Cell(1, 1)); // Reference to A1
        sheet.spreadsheet_set_cell_value(2, 1, ref_expr, &mut status);
        assert_eq!(status, "ok");

        let a2_idx = 1 * 10 + 0;
        if let Some(cell) = sheet.cells[a2_idx].as_ref() {
            assert_eq!(cell.value, 42); // Should get value from A1
            assert!(!cell.error);
        }

        // Test cycle detection
        let cycle_expr = ParsedRHS::SingleValue(Operand::Cell(2, 1)); // Reference to A2, creating cycle
        sheet.spreadsheet_set_cell_value(1, 1, cycle_expr.clone(), &mut status);
        assert_eq!(status, "Cycle Detected");

        // Test formula with function
        let sum_expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(1, 2), // B1
            ),
        };

        // First set B1 to some value
        let b1_val_expr = ParsedRHS::SingleValue(Operand::Number(58));
        sheet.spreadsheet_set_cell_value(1, 2, b1_val_expr, &mut status);

        // Now set A3 to be SUM(A1:B1)
        sheet.spreadsheet_set_cell_value(3, 1, sum_expr, &mut status);
        assert_eq!(status, "ok");

        let a3_idx = 2 * 10 + 0;
        if let Some(cell) = sheet.cells[a3_idx].as_ref() {
            assert_eq!(cell.value, 100); // 42 + 58
            assert!(!cell.error);
        }
    }

    #[test]
    fn test_command_validation() {
        let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Test valid commands

        // Number
        let (valid, row, col, expr) = sheet.is_valid_command("A1", "42");
        assert!(valid);
        assert_eq!(row, 1);
        assert_eq!(col, 1);

        // Cell reference
        let (valid, row, col, expr) = sheet.is_valid_command("B2", "A1");
        assert!(valid);
        assert_eq!(row, 2);
        assert_eq!(col, 2);

        // Arithmetic
        let (valid, row, col, expr) = sheet.is_valid_command("C3", "A1+B2");
        assert!(valid);
        assert_eq!(row, 3);
        assert_eq!(col, 3);

        // Function
        let (valid, row, col, expr) = sheet.is_valid_command("D4", "SUM(A1:B2)");
        assert!(valid);
        assert_eq!(row, 4);
        assert_eq!(col, 4);

        // Test invalid commands

        // Invalid cell name
        let (valid, _, _, _) = sheet.is_valid_command("X1", "42");
        assert!(!valid); // X1 is beyond the column limit

        let (valid, _, _, _) = sheet.is_valid_command("1A", "42");
        assert!(!valid); // Malformed cell name

        // Invalid formula
        let (valid, _, _, _) = sheet.is_valid_command("A1", "A1++B1");
        assert!(!valid); // Malformed formula

        let (valid, _, _, _) = sheet.is_valid_command("A1", "SUM(A1)");
        assert!(!valid); // Missing second argument

        // Out-of-bounds reference
        let (valid, _, _, _) = sheet.is_valid_command("A1", "Z100");
        assert!(!valid); // Z100 is beyond the sheet bounds
        
        // Test additional invalid commands
        let (valid, _, _, _) = sheet.is_valid_command("B1", "-A11");
        assert!(!valid); // A11 out of bounds
        
        let (valid, _, _, _) = sheet.is_valid_command("A1", "MAZ(B1:C5)");
        assert!(!valid); // Invalid function
        
        let (valid, _, _, _) = sheet.is_valid_command("A1", "SLEEP()");
        assert!(!valid); // Missing arguments
        
        let (valid, _, _, _) = sheet.is_valid_command("A1", "MAX()");
        assert!(!valid); // Missing arguments
        
        let (valid, _, _, _) = sheet.is_valid_command("", "");
        assert!(!valid); // Empty cell and formula
    }

    #[test]
    fn test_arithmetic_expression_validation() {
        let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Test valid expressions
        let (valid, expr) = sheet.is_valid_arithmetic_expression("A1+B2");
        assert!(valid);

        let (valid, expr) = sheet.is_valid_arithmetic_expression("42-A1");
        assert!(valid);

        let (valid, expr) = sheet.is_valid_arithmetic_expression("A1*10");
        assert!(valid);

        let (valid, expr) = sheet.is_valid_arithmetic_expression("B2/2");
        assert!(valid);

        // Test invalid expressions
        let (valid, _) = sheet.is_valid_arithmetic_expression("A1++B2");
        assert!(!valid); // Invalid operator

        let (valid, _) = sheet.is_valid_arithmetic_expression("A1+");
        assert!(!valid); // Missing second operand

        let (valid, _) = sheet.is_valid_arithmetic_expression("+B2");
        assert!(!valid); // Missing first operand

        let (valid, _) = sheet.is_valid_arithmetic_expression("A1 + B2");
        assert!(!valid); // Spaces not allowed

        let (valid, _) = sheet.is_valid_arithmetic_expression("Z100+A1");
        assert!(!valid); // Out of bounds cell
        
        let (valid, _) = sheet.is_valid_arithmetic_expression("--5");
        assert!(!valid); // Invalid format
        
        let (valid, _) = sheet.is_valid_arithmetic_expression("-4+++5");
        assert!(!valid); // Invalid format
    }
    
    #[test]
    fn test_range_functions() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
        let mut status = String::new();
        
        // Setup test data in A1-A5
        let vals = [10, 20, 30, 40, 50];
        for i in 0..5 {
            let row = i + 1;
            let expr = ParsedRHS::SingleValue(Operand::Number(vals[i]));
            sheet.spreadsheet_set_cell_value(row as i16, 1, expr, &mut status);
            assert_eq!(status, "ok");
        }
        
        // Test SUM
        let sum_expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(5, 1), // A5
            ),
        };
        sheet.spreadsheet_set_cell_value(1, 2, sum_expr, &mut status); // B1
        assert_eq!(status, "ok");
        
        let b1_idx = 0 * 10 + 1;
        if let Some(cell) = sheet.cells[b1_idx].as_ref() {
            assert_eq!(cell.value, 150); // 10+20+30+40+50
            assert!(!cell.error);
        }
        
        // Test AVG
        let avg_expr = ParsedRHS::Function {
            name: FunctionName::Avg,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(5, 1), // A5
            ),
        };
        sheet.spreadsheet_set_cell_value(2, 2, avg_expr, &mut status); // B2
        assert_eq!(status, "ok");
        
        let b2_idx = 1 * 10 + 1;
        if let Some(cell) = sheet.cells[b2_idx].as_ref() {
            assert_eq!(cell.value, 30); // (10+20+30+40+50)/5
            assert!(!cell.error);
        }
        
        // Test MIN
        let min_expr = ParsedRHS::Function {
            name: FunctionName::Min,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(5, 1), // A5
            ),
        };
        sheet.spreadsheet_set_cell_value(3, 2, min_expr, &mut status); // B3
        assert_eq!(status, "ok");
        
        let b3_idx = 2 * 10 + 1;
        if let Some(cell) = sheet.cells[b3_idx].as_ref() {
            assert_eq!(cell.value, 10);
            assert!(!cell.error);
        }
        
        // Test MAX
        let max_expr = ParsedRHS::Function {
            name: FunctionName::Max,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(5, 1), // A5
            ),
        };
        sheet.spreadsheet_set_cell_value(4, 2, max_expr, &mut status); // B4
        assert_eq!(status, "ok");
        
        let b4_idx = 3 * 10 + 1;
        if let Some(cell) = sheet.cells[b4_idx].as_ref() {
            assert_eq!(cell.value, 50);
            assert!(!cell.error);
        }
        
        // Update a value and check if dependencies update
        let new_val_expr = ParsedRHS::SingleValue(Operand::Number(100));
        sheet.spreadsheet_set_cell_value(1, 1, new_val_expr, &mut status); // A1 = 100
        assert_eq!(status, "ok");
        
        // Check that SUM, AVG, MIN, MAX all updated
        if let Some(cell) = sheet.cells[b1_idx].as_ref() {
            assert_eq!(cell.value, 240); // 100+20+30+40+50
        }
        
        if let Some(cell) = sheet.cells[b2_idx].as_ref() {
            assert_eq!(cell.value, 48); // (100+20+30+40+50)/5
        }
    }
    
    #[test]
    fn test_error_propagation() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
        let mut status = String::new();
        
        // Set up a cell with division by zero error
        let div_zero_expr = ParsedRHS::Arithmetic {
            lhs: Operand::Number(10),
            operator: '/',
            rhs: Operand::Number(0),
        };
        sheet.spreadsheet_set_cell_value(1, 1, div_zero_expr, &mut status); // A1
        assert_eq!(status, "ok");
        
        // Verify error state
        let a1_idx = 0 * 10 + 0;
        if let Some(cell) = sheet.cells[a1_idx].as_ref() {
            assert!(cell.error);
        }
        
        // Reference the error cell
        let ref_expr = ParsedRHS::SingleValue(Operand::Cell(1, 1)); // A1
        sheet.spreadsheet_set_cell_value(2, 1, ref_expr, &mut status); // A2
        assert_eq!(status, "ok");
        
        // Check error propagation
        let a2_idx = 1 * 10 + 0;
        if let Some(cell) = sheet.cells[a2_idx].as_ref() {
            assert!(cell.error);
        }
        
        // Use in arithmetic
        let arith_expr = ParsedRHS::Arithmetic {
            lhs: Operand::Cell(1, 1), // A1 (error)
            operator: '+',
            rhs: Operand::Number(5),
        };
        sheet.spreadsheet_set_cell_value(3, 1, arith_expr, &mut status); // A3
        assert_eq!(status, "ok");
        
        // Check error propagation
        let a3_idx = 2 * 10 + 0;
        if let Some(cell) = sheet.cells[a3_idx].as_ref() {
            assert!(cell.error);
        }
        
        // Use in function
        let func_expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1 (error)
                Operand::Cell(1, 2), // B1
            ),
        };
        sheet.spreadsheet_set_cell_value(4, 1, func_expr, &mut status); // A4
        assert_eq!(status, "ok");
        
        // Check error propagation
        let a4_idx = 3 * 10 + 0;
        if let Some(cell) = sheet.cells[a4_idx].as_ref() {
            assert!(cell.error);
        }
    }
}

fn main() {}
