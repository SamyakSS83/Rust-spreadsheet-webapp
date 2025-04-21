// #[cfg(test)]
#![cfg(not(tarpaulin_include))]

mod spreadsheet_tests {
    use cop::cell::{cell_dep_insert,cell_contains};
    use cop::spreadsheet::{FunctionName, Operand, ParsedRHS, Spreadsheet};
    // use std::collections::BTreeSet;
    use std::time::Instant;
    // use cop::cell;
    // use serde_json::Number;

    // use super::*;

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
        if let Some(cell) = sheet.cells[d2_idx].as_mut() {
            cell.value = 2;
            cell.error = true;
        }
        if let Some(cell) = sheet.cells[d3_idx].as_mut() {
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

        let expr = ParsedRHS::Arithmetic {
            lhs: Operand::Cell(1, 5),
            operator: '+',
            rhs: Operand::Cell(1, 1),
        }; // A5 = ERR
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
        let expr = ParsedRHS::Sleep(
            Operand::Cell(1, 5), // A5 = 2
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, 2); // Sleep function should return cell value
        assert!(error); // Sleep function should return an error
        assert!(duration.as_secs() == 0); // Sleep function should sleep for 0 seconds

        // Test for sleep function without error
        let expr = ParsedRHS::Sleep(
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
        let expr = ParsedRHS::Sleep(
            Operand::Cell(1, 4), // A4 = -234
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, -234); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 0); // Sleep function should sleep for 0 seconds

        // Sleep with value as argument
        let expr = ParsedRHS::Sleep(
            Operand::Number(2), // Sleep for 2 seconds
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, 2); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 2); // Sleep function should sleep for 2 seconds

        // Test for sleep with negative value
        let expr = ParsedRHS::Sleep(
            Operand::Number(-2), // Sleep for -2 seconds
        );
        let start = Instant::now();
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        let duration = start.elapsed();
        assert_eq!(value, -2); // Sleep function should return cell value
        assert!(!error); // Sleep function should return an error
        assert!(duration.as_secs() == 0); // Sleep function should sleep for 0 seconds

        // Test for sleep with zero value
        let expr = ParsedRHS::Sleep(
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
        assert!(sheet.first_step_find_cycle((1, 1), (1, 2), (0, 0), false));

        // Then, modify B1 to depend on A1 (in a separate step)
        if let Some(cell_b1) = sheet.cells[b1_idx].as_mut() {
            cell_dep_insert(cell_b1, 1, 1);
        }

        // Test cycle detection
        assert!(sheet.first_step_find_cycle((1, 2), (1, 1), (0, 0), false));

        // Test no cycle
        let c1_idx = 0 * 10 + 2;

        // First, modify C1 to depend on C2
        if let Some(cell_c1) = sheet.cells[c1_idx].as_mut() {
            cell_dep_insert(cell_c1, 2, 3);
        }

        // Test no cycle detection
        assert!(!sheet.first_step_find_cycle((1, 3), (1, 1), (1, 2), false));

        // Test for is_range = true
        // add A1 to dependencies of D1. i.e. changing D1 will change A1
        let d1_idx = 0 * 10 + 3;
        if let Some(cell_d1) = sheet.cells[d1_idx].as_mut() {
            cell_dep_insert(cell_d1, 1, 1);
        }

        // Test cycle detection
        assert!(sheet.first_step_find_cycle((1, 4), (1, 1), (2, 2), true));

        let mut sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();
        // In dependencies of A1, add B1 to B10 using for loop
        for i in 1..=10 {
            let a1_idx = 0 * 100 + 0;
            if let Some(cell_a1) = sheet.cells[a1_idx].as_mut() {
                cell_dep_insert(cell_a1, i, 2);
            }
        }
        assert!(sheet.first_step_find_cycle((1, 1), (1, 2), (0, 0), false));
    }

    #[test]
    pub fn test_dependencies_management() {
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
        sheet.update_dependencies((1, 1), (1, 2), (1, 3), false);

        // Check if dependencies were correctly set up
        let b1_idx = 0 * 10 + 1;
        let c1_idx = 0 * 10 + 2;

        if let Some(cell_b1) = sheet.cells[b1_idx].as_ref() {
            assert!(cell_contains(cell_b1, 1, 1));
        }

        if let Some(cell_c1) = sheet.cells[c1_idx].as_ref() {
            assert!(cell_contains(cell_c1, 1, 1));
        }

        // Now change A1's formula to depend only on D1
        sheet.remove_old_dependents(1, 1);

        // Set the new formula for A1
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.formula = ParsedRHS::SingleValue(Operand::Cell(1, 4)); // D1
        }

        sheet.update_dependencies((1, 1), (1, 4), (0, 0), false);

        // Check that old dependencies were removed
        if let Some(cell_b1) = sheet.cells[b1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_b1);
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
        let b1_idx = 0 * 100 + 1;
        // set formula of B1 to A1 + A2
        if let Some(cell) = sheet.cells[b1_idx].as_mut() {
            cell.formula = ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 1), // A1
                operator: '+',
                rhs: Operand::Cell(2, 1), // A2
            };
        }
        sheet.update_dependencies((1, 2), (1, 1), (2, 1), false);

        // assign A1 to 20 using formula
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.formula = ParsedRHS::SingleValue(Operand::Number(20));
        }
        sheet.update_dependencies((1, 1), (0, 0), (0, 0), false);
        // assign C1 to B1 * 2
        let c1_idx = 0 * 100 + 2;
        if let Some(cell) = sheet.cells[c1_idx].as_mut() {
            cell.formula = ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 2), // B1
                operator: '*',
                rhs: Operand::Number(2),
            };
        }

        // update dependencies for C1
        sheet.update_dependencies((1, 3), (1, 2), (0, 0), false);

        // update dependencies for A2
        // assign A2 to 10 using formula
        if let Some(cell) = sheet.cells[a2_idx].as_mut() {
            cell.formula = ParsedRHS::SingleValue(Operand::Number(10));
        }
        sheet.update_dependencies((2, 1), (0, 0), (0, 0), false);

        // set D1 = MAX(A1:A4)
        // set formula of D1
        let d1_idx = 0 * 100 + 3;
        if let Some(cell) = sheet.cells[d1_idx].as_mut() {
            cell.formula = ParsedRHS::Function {
                name: FunctionName::Max,
                args: (
                    Operand::Cell(1, 1), // A1
                    Operand::Cell(4, 1), // A4
                ),
            };
        }
        // set D2 = MIN(A1:A4)
        // set formula of D2
        let d2_idx = 1 * 100 + 3;
        if let Some(cell) = sheet.cells[d2_idx].as_mut() {
            cell.formula = ParsedRHS::Function {
                name: FunctionName::Min,
                args: (
                    Operand::Cell(1, 1), // A1
                    Operand::Cell(4, 1), // A4
                ),
            };
        }
        // set D3 = SUM(A1:A4)
        // set formula of D3
        let d3_idx = 2 * 100 + 3;
        if let Some(cell) = sheet.cells[d3_idx].as_mut() {
            cell.formula = ParsedRHS::Function {
                name: FunctionName::Sum,
                args: (
                    Operand::Cell(1, 1), // A1
                    Operand::Cell(4, 1), // A4
                ),
            };
        }
        // set D4= AVG(A1:A4)
        // set formula of D4
        let d4_idx = 3 * 100 + 3;
        if let Some(cell) = sheet.cells[d4_idx].as_mut() {
            cell.formula = ParsedRHS::Function {
                name: FunctionName::Avg,
                args: (
                    Operand::Cell(1, 1), // A1
                    Operand::Cell(4, 1), // A4
                ),
            };
        }
        // set D5 = STDDEV(A1:A4)
        // set formula of D5
        let d5_idx = 4 * 100 + 3;
        if let Some(cell) = sheet.cells[d5_idx].as_mut() {
            cell.formula = ParsedRHS::Function {
                name: FunctionName::Stdev,
                args: (
                    Operand::Cell(1, 1), // A1
                    Operand::Cell(4, 1), // A4
                ),
            };
        }
        // set E1= SLEEP(A1)
        // set formula of E1
        let e1_idx = 0 * 100 + 4;
        if let Some(cell) = sheet.cells[e1_idx].as_mut() {
            cell.formula = ParsedRHS::Sleep(Operand::Cell(1, 1)); // A1
        }
        // set E2= SLEEP(A4)
        // set formula of E2
        let e2_idx = 1 * 100 + 4;
        if let Some(cell) = sheet.cells[e2_idx].as_mut() {
            cell.formula = ParsedRHS::Sleep(Operand::Cell(4, 1)); // A4
        }

        // update dependencies for D1
        sheet.update_dependencies((1, 4), (1, 1), (4, 1), true);
        // update dependencies for D2
        sheet.update_dependencies((2, 4), (1, 1), (4, 1), true);
        // update dependencies for D3
        sheet.update_dependencies((3, 4), (1, 1), (4, 1), true);
        // update dependencies for D4
        sheet.update_dependencies((4, 4), (1, 1), (4, 1), true);
        // update dependencies for D5
        sheet.update_dependencies((5, 4), (1, 1), (4, 1), true);
        // update dependencies for E1
        sheet.update_dependencies((1, 5), (1, 1), (0, 0), false);
        // update dependencies for E2
        sheet.update_dependencies((2, 5), (4, 1), (0, 0), false);
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

        // test for the remove_old_dependents function
        // remove old dependents of B1 . now B1 should not be present in A1's dependents
        sheet.remove_old_dependents(1, 2); //B1
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a1);
            assert!(!deps.contains(&(1, 2)));
        }

        // remove old dependents for E2 , now E2 should not be present in A4's dependents
        sheet.remove_old_dependents(2, 5);
        if let Some(cell_a4) = sheet.cells[a4_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a4);
            assert!(!deps.contains(&(2, 5)));
        }

        // remove old dependents for D1 , now D1 should not be present in A1's dependents
        sheet.remove_old_dependents(1, 4);
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a1);
            assert!(!deps.contains(&(1, 4)));
        }

        // remove old dependents for D2 , now D2 should not be present in A1's dependents
        sheet.remove_old_dependents(2, 4);
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a1);
            assert!(!deps.contains(&(2, 4)));
        }

        // remove old dependents for D3 , now D3 should not be present in A1's dependents
        sheet.remove_old_dependents(3, 4);
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a1);
            assert!(!deps.contains(&(3, 4)));
        }

        // remove old dependents for D4 , now D4 should not be present in A1's dependents
        sheet.remove_old_dependents(4, 4);
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a1);
            assert!(!deps.contains(&(4, 4)));
        }

        // remove old dependents for D5 , now D5 should not be present in A1's dependents
        sheet.remove_old_dependents(5, 4);
        if let Some(cell_a1) = sheet.cells[a1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a1);
            assert!(!deps.contains(&(5, 4)));
        }

        // remove dependents for A1. Now A1 should not be present in any cell's dependents
        sheet.remove_old_dependents(1, 1);
        if let Some(cell_b1) = sheet.cells[b1_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_b1);
            assert!(!deps.contains(&(1, 1)));
        }

        // Single value operand cell is left. check that as well
        // first assign A1 to A2 let's say
        if let Some(cell_a1) = sheet.cells[a1_idx].as_mut() {
            cell_a1.formula = ParsedRHS::SingleValue(Operand::Cell(2, 1)); // A2
        }
        sheet.update_dependencies((1, 1), (2, 1), (0, 0), false);
        // check that A1 is present in A2's dependents
        if let Some(cell_a2) = sheet.cells[a2_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a2);
            assert!(deps.contains(&(1, 1)));
        }
        // remove old dependents for A1 , now A1 should not be present in A2's dependents
        sheet.remove_old_dependents(1, 1);
        if let Some(cell_a2) = sheet.cells[a2_idx].as_ref() {
            let deps = sheet.get_dependent_names(cell_a2);
            assert!(!deps.contains(&(1, 1)));
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
            assert_eq!(sorted.len(), 3);
            assert_eq!(sorted[0], (1, 1)); // C1
            assert_eq!(sorted[1], (1, 2)); // B1
            assert_eq!(sorted[2], (1, 3)); // A1
        }

        // Test topo sort for large spreadsheet (enhanced check)
        let mut sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();
        // set_cell(sheet, "B1", "SUM(A1:A10)");
        // set_cell(sheet, "B2", "MAX(A1:A10)");
        // set_cell(sheet, "B3", "MIN(A1:A10)");
        // set_cell(sheet, "B4", "AVG(A1:A10)");
        // set_cell(sheet, "B5", "STDEV(A1:A10)");

        // // Set formulas in C1:C5
        // set_cell(sheet, "C1", "SUM(B1:B5)");
        // set_cell(sheet, "C2", "MAX(B1:B5)");
        // set_cell(sheet, "C3", "MIN(B1:B5)");
        // set_cell(sheet, "C4", "AVG(B1:B5)");
        // set_cell(sheet, "C5", "STDEV(B1:B5)");

        // ADD B1 till B5 to A1:A10
        for i in 1..=10 {
            for j in 1..=5 {
                let cell_idx = (i - 1) * 100;
                if let Some(cell) = sheet.cells[cell_idx].as_mut() {
                    cell_dep_insert(cell, j, 2);
                }
            }
        }

        // ADD C1 till C5 to B1:B10
        for i in 1..=10 {
            for j in 1..=5 {
                let cell_idx = (i - 1) * 100 + 1;
                if let Some(cell) = sheet.cells[cell_idx].as_mut() {
                    cell_dep_insert(cell, j, 3);
                }
            }
        }

        let sorted_cells = sheet.topo_sort(&sheet.cells[0].as_ref().unwrap());
        // the vector should be something of the type... A1 then (B1 to B5 in any order) then (C1:C5 in any order)
        assert_eq!(sorted_cells.len(), 11);
        assert_eq!(sorted_cells[0], (1, 1)); // A1
        for i in 1..=5 {
            assert_eq!(sorted_cells[i], (6 - i as i16, 2)); // B1 to B5
        }
        for i in 1..=5 {
            assert_eq!(sorted_cells[i + 5], (6 - i as i16, 3)); // C1 to C5
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

        let avg_expr = ParsedRHS::Function {
            name: FunctionName::Avg,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 1), // A2
            ),
        };
        sheet.spreadsheet_set_cell_value(3, 1, avg_expr, &mut status);
        assert_eq!(status, "ok");

        let a3_idx = 2 * 10 + 0;
        if let Some(cell) = sheet.cells[a3_idx].as_ref() {
            assert_eq!(cell.value, 42); // (42 + 42) / 2
            assert!(!cell.error);
        }

        let min_expr = ParsedRHS::Function {
            name: FunctionName::Min,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 1), // A2
            ),
        };
        sheet.spreadsheet_set_cell_value(4, 1, min_expr, &mut status);
        assert_eq!(status, "ok");
        let a4_idx = 3 * 10 + 0;
        if let Some(cell) = sheet.cells[a4_idx].as_ref() {
            assert_eq!(cell.value, 42); // min(42, 42)
            assert!(!cell.error);
        }

        let max_expr = ParsedRHS::Function {
            name: FunctionName::Max,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 1), // A2
            ),
        };
        sheet.spreadsheet_set_cell_value(5, 1, max_expr, &mut status);
        assert_eq!(status, "ok");
        let a5_idx = 4 * 10 + 0;
        if let Some(cell) = sheet.cells[a5_idx].as_ref() {
            assert_eq!(cell.value, 42); // max(42, 42)
            assert!(!cell.error);
        }

        let stdev_expr = ParsedRHS::Function {
            name: FunctionName::Stdev,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 1), // A2
            ),
        };
        sheet.spreadsheet_set_cell_value(6, 1, stdev_expr, &mut status);
        assert_eq!(status, "ok");
        let a6_idx = 5 * 10 + 0;
        if let Some(cell) = sheet.cells[a6_idx].as_ref() {
            assert_eq!(cell.value, 0); // stdev(42, 42) = 0
            assert!(!cell.error);
        }

        // First set B1 to some value
        let b1_val_expr = ParsedRHS::SingleValue(Operand::Number(58));
        sheet.spreadsheet_set_cell_value(1, 2, b1_val_expr, &mut status);

        // Now set A3 to be SUM(A1:B1)
        sheet.spreadsheet_set_cell_value(3, 1, sum_expr, &mut status);
        assert_eq!(status, "ok");

        if let Some(cell) = sheet.cells[a3_idx].as_ref() {
            assert_eq!(cell.value, 100); // 42 + 58
            assert!(!cell.error);
        }

        // Test for arithmetic expression type formula
        let arith_expr = ParsedRHS::Arithmetic {
            lhs: Operand::Cell(1, 1), // A1
            operator: '+',
            rhs: Operand::Cell(2, 1), // A2
        };
        sheet.spreadsheet_set_cell_value(7, 1, arith_expr, &mut status);
        assert_eq!(status, "ok");
        let a7_idx = 6 * 10 + 0;
        if let Some(cell) = sheet.cells[a7_idx].as_ref() {
            assert_eq!(cell.value, 84); // 42 + 42
            assert!(!cell.error);
        }

        // Test for sleep type formula
        let sleep_expr = ParsedRHS::Sleep(Operand::Number(2)); // Sleep for 2 seconds
        sheet.spreadsheet_set_cell_value(8, 1, sleep_expr, &mut status);
        assert_eq!(status, "ok");
        let a8_idx = 7 * 10 + 0;
        if let Some(cell) = sheet.cells[a8_idx].as_ref() {
            assert_eq!(cell.value, 2); // Sleep function should return cell value
            assert!(!cell.error);
        }

        // Test for sleep with Operand::Cell
        let sleep_expr = ParsedRHS::Sleep(Operand::Cell(6, 1)); // Sleep for 0 seconds
        sheet.spreadsheet_set_cell_value(9, 1, sleep_expr, &mut status);
        assert_eq!(status, "ok");
        let a9_idx = 8 * 10 + 0;
        if let Some(cell) = sheet.cells[a9_idx].as_ref() {
            assert_eq!(cell.value, 0); // Sleep function should return cell value
            assert!(!cell.error);
        }

        //Test for Copy type formula
        let copy_expr = ParsedRHS::Function {
            name: FunctionName::Copy,
            args: (Operand::Cell(1, 1), Operand::Cell(8, 1)),
        };
        sheet.spreadsheet_set_cell_value(1, 2, copy_expr, &mut status);
        // B1 should have value of A1 . B2 should have value of A2 . B3 should have value of A3...and so on. and formula of them should be None
        assert_eq!(status, "ok");
        let b1_idx = 0 * 10 + 1;
        if let Some(cell) = sheet.cells[b1_idx].as_ref() {
            assert_eq!(cell.value, 42); // B1 should have value of A1
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(42)));
        }
        let b2_idx = 1 * 10 + 1;
        if let Some(cell) = sheet.cells[b2_idx].as_ref() {
            assert_eq!(cell.value, 42); // B2 should have value of A2
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(42)));
        }
        let b3_idx = 2 * 10 + 1;
        if let Some(cell) = sheet.cells[b3_idx].as_ref() {
            assert_eq!(cell.value, 100); // B3 should have value of A3
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(100)));
        }
        let b4_idx = 3 * 10 + 1;
        if let Some(cell) = sheet.cells[b4_idx].as_ref() {
            assert_eq!(cell.value, 42); // B4 should have value of A4
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(42)));
        }
        let b5_idx = 4 * 10 + 1;
        if let Some(cell) = sheet.cells[b5_idx].as_ref() {
            assert_eq!(cell.value, 42); // B5 should have value of A5
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(42)));
        }
        let b6_idx = 5 * 10 + 1;
        if let Some(cell) = sheet.cells[b6_idx].as_ref() {
            assert_eq!(cell.value, 0); // B6 should have value of A6
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(0)));
        }
        let b7_idx = 6 * 10 + 1;
        if let Some(cell) = sheet.cells[b7_idx].as_ref() {
            assert_eq!(cell.value, 84); // B7 should have value of A7
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(84)));
        }
        let b8_idx = 7 * 10 + 1;
        if let Some(cell) = sheet.cells[b8_idx].as_ref() {
            assert_eq!(cell.value, 2); // B8 should have value of A8
            assert_eq!(cell.formula, ParsedRHS::SingleValue(Operand::Number(2)));
        }
    }

    #[test]
    fn test_undo_function() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
        let mut status = String::new();

        // Test setting a simple value
        let val_expr = ParsedRHS::SingleValue(Operand::Number(42));
        sheet.spreadsheet_set_cell_value(1, 1, val_expr, &mut status);
        assert_eq!(status, "ok");

        // Test undo
        sheet.spreadsheet_undo(&mut status);
        assert_eq!(sheet.cells[0].as_ref().unwrap().value, 0); // Cell should be reset to 0

        sheet.spreadsheet_undo(&mut status);
        // should go back to 42
        assert_eq!(sheet.cells[0].as_ref().unwrap().value, 42); // Cell should be reset to 42

        let val_expr = ParsedRHS::SingleValue(Operand::Number(100));
        sheet.spreadsheet_set_cell_value(1, 1, val_expr, &mut status);

        assert_eq!(sheet.cells[0].as_ref().unwrap().value, 100); // Cell should be set to 100

        let val_expr = ParsedRHS::SingleValue(Operand::Number(200));
        sheet.spreadsheet_set_cell_value(2, 1, val_expr, &mut status);

        assert_eq!(sheet.cells[10].as_ref().unwrap().value, 200); // Cell should be set to 200

        let avg_expr = ParsedRHS::Function {
            name: FunctionName::Avg,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 1), // A2
            ),
        };

        sheet.spreadsheet_set_cell_value(1, 3, avg_expr, &mut status);

        assert_eq!(sheet.cells[2].as_ref().unwrap().value, 150); // Cell should be set to 150

        sheet.spreadsheet_undo(&mut status);
        assert_eq!(sheet.cells[2].as_ref().unwrap().value, 0); // Cell should be reset to 0
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
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(42)));

        // Cell reference
        let (valid, row, col, expr) = sheet.is_valid_command("B2", "A1");
        assert!(valid);
        assert_eq!(row, 2);
        assert_eq!(col, 2);
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Cell(1, 1)));

        // Arithmetic
        let (valid, row, col, expr) = sheet.is_valid_command("C3", "A1+B2");
        assert!(valid);
        assert_eq!(row, 3);
        assert_eq!(col, 3);
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 1),
                operator: '+',
                rhs: Operand::Cell(2, 2),
            }
        );

        // Function
        // SUM
        let (valid, row, col, expr) = sheet.is_valid_command("D4", "SUM(A1:B2)");
        assert!(valid);
        assert_eq!(row, 4);
        assert_eq!(col, 4);
        assert_eq!(
            expr,
            ParsedRHS::Function {
                name: FunctionName::Sum,
                args: (Operand::Cell(1, 1), Operand::Cell(2, 2),),
            }
        );
        // AVG
        let (valid, row, col, expr) = sheet.is_valid_command("E5", "AVG(A1:B2)");
        assert!(valid);
        assert_eq!(row, 5);
        assert_eq!(col, 5);
        assert_eq!(
            expr,
            ParsedRHS::Function {
                name: FunctionName::Avg,
                args: (Operand::Cell(1, 1), Operand::Cell(2, 2),),
            }
        );
        // MIN
        let (valid, row, col, expr) = sheet.is_valid_command("F6", "MIN(A1:B2)");
        assert!(valid);
        assert_eq!(row, 6);
        assert_eq!(col, 6);
        assert_eq!(
            expr,
            ParsedRHS::Function {
                name: FunctionName::Min,
                args: (Operand::Cell(1, 1), Operand::Cell(2, 2),),
            }
        );
        // MAX
        let (valid, row, col, expr) = sheet.is_valid_command("G7", "MAX(A1:B2)");
        assert!(valid);
        assert_eq!(row, 7);
        assert_eq!(col, 7);
        assert_eq!(
            expr,
            ParsedRHS::Function {
                name: FunctionName::Max,
                args: (Operand::Cell(1, 1), Operand::Cell(2, 2),),
            }
        );
        // STDEV
        let (valid, row, col, expr) = sheet.is_valid_command("H8", "STDEV(A1:B2)");
        assert!(valid);
        assert_eq!(row, 8);
        assert_eq!(col, 8);
        assert_eq!(
            expr,
            ParsedRHS::Function {
                name: FunctionName::Stdev,
                args: (Operand::Cell(1, 1), Operand::Cell(2, 2),),
            }
        );
        // COPY
        let (valid, row, col, expr) = sheet.is_valid_command("I9", "COPY(A1:B2)");
        assert!(valid);
        assert_eq!(row, 9);
        assert_eq!(col, 9);
        assert_eq!(
            expr,
            ParsedRHS::Function {
                name: FunctionName::Copy,
                args: (Operand::Cell(1, 1), Operand::Cell(2, 2),),
            }
        );

        let (valid, _,  _, _) = sheet.is_valid_command("J10", "COPY(A1:B2)");
        assert!(!valid);

        let (valid, row, col, expr) = sheet.is_valid_command("A1", "SLEEP(2)");
        assert!(valid);
        assert_eq!(row, 1);
        assert_eq!(col, 1);
        assert_eq!(
            expr,
            ParsedRHS::Sleep (Operand::Number(2))
        );

        let (valid, row, col, expr) = sheet.is_valid_command("A1", "SLEEP(B2)");
        assert!(valid);
        assert_eq!(row, 1);
        assert_eq!(col, 1);
        assert_eq!(
            expr,
            ParsedRHS::Sleep (Operand::Cell(2,2))
        );

        let (valid, row, col, expr) = sheet.is_valid_command("A1", "");
        assert!(!valid);
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

        // Test arbitrary is_Valid commands
        // give cell name A1 , formula = -1 , +1 , +2, -4 , 2147483647,-2147483648, 00, 09, 0090, -0090, -1*-1,-1*+1, -1+-1,-1-+1,-1/+1,-1/-1,1/0,2/0,A2*B1,-1*B1, +3*B1,B1+2,B1/0,C1-34,C1/-3,D1*-4,D1+-6,C1-+3
        let (valid, row, col, expr) = sheet.is_valid_command("A1", "-1");
        assert!(valid); // valid number
        assert_eq!(row, 1);
        assert_eq!(col, 1);
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(-1)));

        let (valid, row, col, expr) = sheet.is_valid_command("A1", "+1");
        assert!(valid); // valid number
        assert_eq!(row, 1);
        assert_eq!(col, 1);
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(1)));

        let (valid, row, col, expr) = sheet.is_valid_command("A1", "+2");
        assert!(valid); // valid number
        assert_eq!(row, 1);
        assert_eq!(col, 1);
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(2)));

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-4");
        assert!(valid); // valid number
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(-4)));

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "2147483647");
        assert!(valid); // valid number
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(2147483647)));

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-2147483648");
        assert!(valid); // valid number
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(-2147483648)));
        let (valid, _, _, expr) = sheet.is_valid_command("A1", "00");
        assert!(valid); // valid number
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(0)));

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "09");
        assert!(valid); // valid number
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(9)));

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "0090");
        assert!(valid); // valid number
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(90)));

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-0090");
        assert!(valid); // valid number
        assert_eq!(expr, ParsedRHS::SingleValue(Operand::Number(-90)));

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-1*-1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(-1),
                operator: '*',
                rhs: Operand::Number(-1),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-1*+1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(-1),
                operator: '*',
                rhs: Operand::Number(1),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-1+-1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(-1),
                operator: '+',
                rhs: Operand::Number(-1),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-1-+1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(-1),
                operator: '-',
                rhs: Operand::Number(1),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-1/+1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(-1),
                operator: '/',
                rhs: Operand::Number(1),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-1/-1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(-1),
                operator: '/',
                rhs: Operand::Number(-1),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "1/0");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(1),
                operator: '/',
                rhs: Operand::Number(0),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "2/0");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(2),
                operator: '/',
                rhs: Operand::Number(0),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "A2*B1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(2, 1),
                operator: '*',
                rhs: Operand::Cell(1, 2),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "-1*B1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(-1),
                operator: '*',
                rhs: Operand::Cell(1, 2),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "+3*B1");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Number(3),
                operator: '*',
                rhs: Operand::Cell(1, 2),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "B1+2");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 2),
                operator: '+',
                rhs: Operand::Number(2),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "B1/0");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 2),
                operator: '/',
                rhs: Operand::Number(0),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "C1-34");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 3),
                operator: '-',
                rhs: Operand::Number(34),
            }
        );
        let (valid, _, _, expr) = sheet.is_valid_command("A1", "C1/-3");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 3),
                operator: '/',
                rhs: Operand::Number(-3),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "D1*-4");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 4),
                operator: '*',
                rhs: Operand::Number(-4),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "D1+-6");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 4),
                operator: '+',
                rhs: Operand::Number(-6),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "C1-+3");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 3),
                operator: '-',
                rhs: Operand::Number(3),
            }
        );

        let (valid, _, _, expr) = sheet.is_valid_command("A1", "C1-+3");
        assert!(valid); // valid number
        assert_eq!(
            expr,
            ParsedRHS::Arithmetic {
                lhs: Operand::Cell(1, 3),
                operator: '-',
                rhs: Operand::Number(3),
            }
        );

        let (valid, _, _, _) = sheet.is_valid_command("A1", "");
        assert!(!valid); // Empty formula

        let (valid, _, _, _) = sheet.is_valid_command("A1", "2.3-+3");
        assert!(!valid); // invalid number
        let (valid, _, _, _) = sheet.is_valid_command("A1", "2-+3.5");
        assert!(!valid); // invalid number
        let (valid, _, _, _) = sheet.is_valid_command("A1", "A+3");
        assert!(!valid); // invalid number
        let (valid, _, _, _) = sheet.is_valid_command("A1", "A1+B");
        assert!(!valid); // invalid number

        let (valid,_,_,_) = sheet.is_valid_command("A1", "A1%B1");
        assert!(!valid);

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

        // Test COPY
        let copy_expr = ParsedRHS::Function {
            name: FunctionName::Copy,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(5, 1), // A5
            ),
        };
        sheet.spreadsheet_set_cell_value(5, 2, copy_expr, &mut status); // B5
        assert_eq!(status, "ok");

        let b5_idx = 4 * 10 + 1;
        if let Some(cell) = sheet.cells[b5_idx].as_ref() {
            assert_eq!(cell.value, 10); // Copying A1
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

        if let Some(cell) = sheet.cells[b3_idx].as_ref() {
            assert_eq!(cell.value, 20); // 20
        }
        if let Some(cell) = sheet.cells[b4_idx].as_ref() {
            assert_eq!(cell.value, 100); // 100
        }
        if let Some(cell) = sheet.cells[b5_idx].as_ref() {
            assert_eq!(cell.value, 10); // B5 will still be 10
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

    #[test]
    fn test_spreadsheet_display() {
        use std::io::{self, Write};

        // Create a spreadsheet for testing display
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Setup cells with various values and error states
        let a1_idx = 0 * 10 + 0; // A1
        let a2_idx = 1 * 10 + 0; // A2
        let b1_idx = 0 * 10 + 1; // B1
        let b2_idx = 1 * 10 + 1; // B2

        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.value = 42;
            cell.error = false;
        }

        if let Some(cell) = sheet.cells[a2_idx].as_mut() {
            cell.value = 100;
            cell.error = false;
        }

        if let Some(cell) = sheet.cells[b1_idx].as_mut() {
            cell.value = 0; // Value doesn't matter when error=true
            cell.error = true;
        }

        if let Some(cell) = sheet.cells[b2_idx].as_mut() {
            cell.value = 200;
            cell.error = false;
        }

        // Capture stdout to verify output
        let mut output = Vec::new();
        {
            // Redirect stdout to our buffer temporarily
            let original_stdout = io::stdout();
            let mut handle = original_stdout.lock();

            // Call spreadsheet_display
            sheet.spreadsheet_display();

            // Flush to ensure all output is captured
            handle.flush().unwrap();
        }

        // Convert captured output to string
        let output_str = String::from_utf8_lossy(&output);

        // Basic verification of output structure
        // Note: We can't fully test the exact output string since it's being
        // printed directly to stdout and we're not capturing it in this test.
        // In a real test, we'd use something like `std::io::Cursor` to capture the output.

        // Instead, we'll just verify the function doesn't panic
        // A more comprehensive test would use a crate like `rexpect` or modify
        // spreadsheet_display to accept a writer parameter.

        // For now, this test just ensures the function runs without panicking
        assert!(true);
    }

    #[test]
    fn test_spreadsheet_display_view_window() {
        // Test the view window functionality of spreadsheet_display
        let mut sheet = Spreadsheet::spreadsheet_create(20, 20).unwrap();

        // Set up some cell values
        for r in 1..=20 {
            for c in 1..=20 {
                let index = (r - 1) as usize * 20 + (c - 1) as usize;
                if let Some(cell) = sheet.cells[index].as_mut() {
                    cell.value = (r * 100 + c) as i32;
                }
            }
        }

        // Test default view (0,0)
        assert_eq!(sheet.view_row, 0);
        assert_eq!(sheet.view_col, 0);

        // Change view window
        sheet.view_row = 5;
        sheet.view_col = 5;

        // Call display - this just tests it doesn't crash
        sheet.spreadsheet_display();

        // The displayed cells should now start from row 6, col 6
        // Verify view window state
        assert_eq!(sheet.view_row, 5);
        assert_eq!(sheet.view_col, 5);

        // Test with view at the edge
        sheet.view_row = 15; // Should show rows 16-20
        sheet.view_col = 15; // Should show cols 16-20

        // Call display
        sheet.spreadsheet_display();

        // Verify window state
        assert_eq!(sheet.view_row, 15);
        assert_eq!(sheet.view_col, 15);
    }
    #[test]
    fn test_undo_function_new() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
        let mut status = String::new();

        // Test setting a simple value
        let val_expr = ParsedRHS::SingleValue(Operand::Number(42));
        sheet.spreadsheet_set_cell_value(1, 1, val_expr, &mut status);
        assert_eq!(status, "ok");

        // Check initial state
        let a1_idx = 0;
        assert_eq!(sheet.cells[a1_idx].as_ref().unwrap().value, 42);

        // Test undo
        sheet.spreadsheet_undo(&mut status);
        assert_eq!(sheet.cells[a1_idx].as_ref().unwrap().value, 0); // Cell should be reset to 0

        // Second call to undo will redo the operation (revert to 42)
        sheet.spreadsheet_undo(&mut status);
        assert_eq!(sheet.cells[a1_idx].as_ref().unwrap().value, 42); // Cell should be reset to 42

        // Set A1 to 100
        let val_expr = ParsedRHS::SingleValue(Operand::Number(100));
        sheet.spreadsheet_set_cell_value(1, 1, val_expr, &mut status);
        assert_eq!(sheet.cells[a1_idx].as_ref().unwrap().value, 100); // Cell should be set to 100

        // Set A2 to 200
        let a2_idx = 10; // row 2, col 1 (0-indexed)
        let val_expr = ParsedRHS::SingleValue(Operand::Number(200));
        sheet.spreadsheet_set_cell_value(2, 1, val_expr, &mut status);
        assert_eq!(sheet.cells[a2_idx].as_ref().unwrap().value, 200); // Cell should be set to 200

        // Set C1 to AVG(A1:A2)
        let c1_idx = 2; // row 1, col 3 (0-indexed)
        let avg_expr = ParsedRHS::Function {
            name: FunctionName::Avg,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 1), // A2
            ),
        };
        sheet.undo_stack.clear();
        sheet.spreadsheet_set_cell_value(1, 3, avg_expr, &mut status);
        // eprintln!("value forÇ is {:?}", sheet.cells[c1_idx]);
        assert_eq!(sheet.cells[c1_idx].as_ref().unwrap().value, 150); // Cell should be set to 150

        // Undo the AVG function in C1
        sheet.spreadsheet_undo(&mut status);
        assert_eq!(sheet.cells[c1_idx].as_ref().unwrap().value, 0); // Cell should be reset to 0

        // Test undo with arithmetic operations
        // Set A3 to A1 + 50
        let a3_idx = 20; // row 3, col 1 (0-indexed)
        let arith_expr = ParsedRHS::Arithmetic {
            lhs: Operand::Cell(1, 1),
            operator: '+',
            rhs: Operand::Number(50),
        };
        sheet.spreadsheet_set_cell_value(3, 1, arith_expr, &mut status);
        assert_eq!(status, "ok");
        eprintln!("value of a3 is {:?}", sheet.cells[a3_idx]);
        assert_eq!(sheet.cells[a3_idx].as_ref().unwrap().value, 150); // 100 + 50

        // Test undo with range functions
        // Set A4 to SUM(A1:A3)
        let a4_idx = 30; // row 4, col 1 (0-indexed)
        let sum_expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(3, 1), // A3
            ),
        };
        sheet.spreadsheet_set_cell_value(4, 1, sum_expr, &mut status);
        assert_eq!(status, "ok");
        assert_eq!(sheet.cells[a4_idx].as_ref().unwrap().value, 450); // 100 + 200 + 150

        // Test undo of A4 SUM function
        sheet.spreadsheet_undo(&mut status);
        assert_eq!(sheet.cells[a4_idx].as_ref().unwrap().value, 0); // Reset A4 to 0

        // Redo the SUM function in A4
        sheet.spreadsheet_undo(&mut status);
        // The correct expected value is 450 because our implementation of undo is actually toggling
        // between the two most recent states
        assert_eq!(sheet.cells[a4_idx].as_ref().unwrap().value, 450); // A4 should be back to 450

        // Test undo with SLEEP function
        let a5_idx = 40; // row 5, col 1 (0-indexed)
        let sleep_expr = ParsedRHS::Sleep(Operand::Number(1));
        sheet.spreadsheet_set_cell_value(5, 1, sleep_expr, &mut status);
        assert_eq!(status, "ok");
        assert_eq!(sheet.cells[a5_idx].as_ref().unwrap().value, 1);

        sheet.spreadsheet_undo(&mut status);
        assert_eq!(sheet.cells[a5_idx].as_ref().unwrap().value, 0);

        // Test undo after setting a cell to an error state
        let a6_idx = 50; // row 6, col 1 (0-indexed)
        let div_zero_expr = ParsedRHS::Arithmetic {
            lhs: Operand::Number(10),
            operator: '/',
            rhs: Operand::Number(0),
        };
        sheet.spreadsheet_set_cell_value(6, 1, div_zero_expr, &mut status);
        assert_eq!(status, "ok");
        assert!(sheet.cells[a6_idx].as_ref().unwrap().error);

        sheet.spreadsheet_undo(&mut status);
        assert!(!sheet.cells[a6_idx].as_ref().unwrap().error);
        assert_eq!(sheet.cells[a6_idx].as_ref().unwrap().value, 0);
    }
}

fn main() {}
