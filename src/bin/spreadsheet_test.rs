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

        // Test creating a spreadsheet with larger dimensions
        let sheet = Spreadsheet::spreadsheet_create(100, 26);
        assert!(sheet.is_some());
        let sheet = sheet.unwrap();
        assert_eq!(sheet.rows, 100);
        assert_eq!(sheet.cols, 26);

        // Test cells initialization
        for r in 1..=10 {
            for c in 1..=10 {
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

        // Test converting letters to column numbers
        assert_eq!(Spreadsheet::letter_to_col("A"), 1);
        assert_eq!(Spreadsheet::letter_to_col("Z"), 26);
        assert_eq!(Spreadsheet::letter_to_col("AA"), 27);
        assert_eq!(Spreadsheet::letter_to_col("AZ"), 52);
        assert_eq!(Spreadsheet::letter_to_col("BA"), 53);
        assert_eq!(Spreadsheet::letter_to_col("ZZ"), 702);
        assert_eq!(Spreadsheet::letter_to_col("AAA"), 703);
    }

    #[test]
    fn test_cell_name_operations() {
        // Test generating cell names
        assert_eq!(Spreadsheet::get_cell_name(1, 1), "A1");
        assert_eq!(Spreadsheet::get_cell_name(10, 26), "Z10");
        assert_eq!(Spreadsheet::get_cell_name(100, 27), "AA100");

        // Test parsing cell names in a spreadsheet context
        let sheet = Spreadsheet::spreadsheet_create(100, 100).unwrap();

        // Valid cell names
        assert_eq!(sheet.spreadsheet_parse_cell_name("A1"), Some((1, 1)));
        assert_eq!(sheet.spreadsheet_parse_cell_name("Z10"), Some((10, 26)));
        assert_eq!(sheet.spreadsheet_parse_cell_name("AA100"), Some((100, 27)));

        // Invalid cell names (out of bounds)
        assert_eq!(sheet.spreadsheet_parse_cell_name("A0"), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name("A101"), None);
        assert_eq!(sheet.spreadsheet_parse_cell_name("CV1"), Some((1, 100))); // Beyond column limit

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

        // Test non-numeric strings
        assert!(!Spreadsheet::is_numeric(""));
        assert!(!Spreadsheet::is_numeric("A"));
        assert!(!Spreadsheet::is_numeric("12A"));
        assert!(!Spreadsheet::is_numeric("A12"));
        assert!(!Spreadsheet::is_numeric("-123")); // Contains non-digit
        assert!(!Spreadsheet::is_numeric("+123")); // Contains non-digit
        assert!(!Spreadsheet::is_numeric("12.3")); // Contains non-digit
    }

    #[test]
    fn test_find_depends() {
        let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Test range functions
        let result = sheet.find_depends("SUM(A1:B2)");
        assert!(result.is_ok());
        let (r1, r2, c1, c2, range_bool) = result.unwrap();
        assert_eq!(r1, 0); // 0-based indexing
        assert_eq!(r2, 1);
        assert_eq!(c1, 1);
        assert_eq!(c2, 2);
        assert!(range_bool);

        // Test arithmetic with cell references
        let result = sheet.find_depends("A1+B2");
        assert!(result.is_ok());
        let (r1, r2, c1, c2, range_bool) = result.unwrap();
        assert_eq!(r1, 1);
        assert_eq!(r2, 2);
        assert_eq!(c1, 1);
        assert_eq!(c2, 2);
        assert!(!range_bool);

        // Test single cell reference
        let result = sheet.find_depends("A1");
        assert!(result.is_ok());
        let (r1, _, c1, _, range_bool) = result.unwrap();
        assert_eq!(r1, 1);
        assert_eq!(c1, 1);
        assert!(!range_bool);

        // Test invalid range
        let result = sheet.find_depends("SUM(B2:A1)");
        assert!(result.is_err());

        // Test malformed formula
        let result = sheet.find_depends("SUM(A1,B2)");
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_expression() {
        let mut sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();

        // Setup some cell values
        let a1_idx = 0 * 10 + 0;
        let a2_idx = 1 * 10 + 0;
        let b1_idx = 0 * 10 + 1;
        let b2_idx = 1 * 10 + 1;

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

        // Test SingleValue - direct number
        let expr = ParsedRHS::SingleValue(Operand::Number(42));
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 1);
        assert_eq!(value, 42);
        assert!(!error);

        // Test SingleValue - cell reference
        let expr = ParsedRHS::SingleValue(Operand::Cell(1, 1)); // A1
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 2, 2);
        assert_eq!(value, 10);
        assert!(!error);

        // Test Arithmetic - addition
        let expr = ParsedRHS::Arithmetic {
            lhs: Operand::Cell(1, 1), // A1 = 10
            operator: '+',
            rhs: Operand::Cell(2, 1), // A2 = 20
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 1, 2);
        assert_eq!(value, 30);
        assert!(!error);

        // Test Function - SUM
        let expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 2), // B2
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 100); // 10 + 20 + 30 + 40
        assert!(!error);

        // Test Function - MIN
        let expr = ParsedRHS::Function {
            name: FunctionName::Min,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 2), // B2
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 10);
        assert!(!error);

        // Test Function - MAX
        let expr = ParsedRHS::Function {
            name: FunctionName::Max,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 2), // B2
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 40);
        assert!(!error);

        // Test Function - AVG
        let expr = ParsedRHS::Function {
            name: FunctionName::Avg,
            args: (
                Operand::Cell(1, 1), // A1
                Operand::Cell(2, 2), // B2
            ),
        };
        let (value, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert_eq!(value, 25); // (10 + 20 + 30 + 40) / 4
        assert!(!error);

        // Test error propagation
        if let Some(cell) = sheet.cells[a1_idx].as_mut() {
            cell.error = true;
        }

        let expr = ParsedRHS::SingleValue(Operand::Cell(1, 1)); // A1 (with error)
        let (_, error) = sheet.spreadsheet_evaluate_expression(&expr, 2, 2);
        assert!(error);

        let expr = ParsedRHS::Function {
            name: FunctionName::Sum,
            args: (
                Operand::Cell(1, 1), // A1 (with error)
                Operand::Cell(2, 2), // B2
            ),
        };
        let (_, error) = sheet.spreadsheet_evaluate_expression(&expr, 3, 3);
        assert!(error);
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
    }
}

fn main() {}
