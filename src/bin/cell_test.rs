#![cfg(not(tarpaulin_include))]

fn main() {
    // When run directly, this will run the tests
    println!("=== Cell Test Suite ===");
    println!("Run with 'cargo test' to execute tests");
}

#[cfg(test)]
mod cell_tests {
    use cop::cell::{Cell, Dependents, cell_create, cell_dep_insert, cell_dep_remove};

    fn cell_contains(cell: &Cell, row: i16, col: i16) -> bool {
        cell.contains(row, col)
    }

    #[test]
    fn test_basic_cell_creation() {
        let cell = cell_create(1, 1);
        assert_eq!(cell.row, 1);
        assert_eq!(cell.col, 1);
        assert_eq!(cell.value, 0);
        assert!(!cell.error);
        assert!(matches!(cell.formula, cop::spreadsheet::ParsedRHS::None));
        assert!(matches!(cell.dependents, Dependents::None));
    }

    #[test]
    fn test_cell_value_modification() {
        let mut cell = cell_create(1, 1);
        cell.value = 100;
        assert_eq!(cell.value, 100);
    }

    #[test]
    fn test_cell_formula_assignment() {
        use cop::spreadsheet::{Operand, ParsedRHS};

        let mut cell = cell_create(1, 1);
        cell.formula = ParsedRHS::SingleValue(Operand::Number(42));

        assert!(matches!(cell.formula, ParsedRHS::SingleValue(_)));

        if let ParsedRHS::SingleValue(Operand::Number(value)) = cell.formula {
            assert_eq!(value, 42);
        } else {
            panic!("Expected ParsedRHS::SingleValue(Operand::Number)");
        }
    }

    #[test]
    fn test_error_flag() {
        let mut cell = cell_create(1, 1);
        cell.error = true;
        assert!(cell.error);
    }

    #[test]
    fn test_managing_dependents() {
        let mut cell = cell_create(1, 1);

        // Check whether no dependents exist
        assert!(!cell_contains(&cell, 1, 1)); // nothing should be there
        assert!(!cell_contains(&cell, 2, 1)); // nothing should be there
        assert!(!cell_contains(&cell, 2, 2)); // nothing should be there

        cell_dep_insert(&mut cell, 2, 1); // B1
        cell_dep_insert(&mut cell, 3, 2); // C2
        cell_dep_insert(&mut cell, 4, 3); // D3

        assert!(cell_contains(&cell, 2, 1));
        assert!(cell_contains(&cell, 3, 2));
        assert!(cell_contains(&cell, 4, 3));
        assert!(!cell_contains(&cell, 5, 4)); // E4 not added

        // Add more dependents such that it becomes set
        for i in 5..=10 {
            cell_dep_insert(&mut cell, i, i);
        }

        // Check that Set was created by verifying format
        match &cell.dependents {
            Dependents::Vector(_) => panic!("Should have converted to Set"),
            Dependents::Set(_) => {} // This is expected
            Dependents::None => panic!("Should not be None"),
        }
        // Ensure all dependencies are still accessible
        for i in 5..=10 {
            assert!(cell_contains(&cell, i, i));
        }
    }

    #[test]
    fn test_removing_dependents() {
        let mut cell = cell_create(1, 1);

        cell_dep_insert(&mut cell, 2, 1); // B1
        cell_dep_insert(&mut cell, 3, 2); // C2

        assert!(cell_contains(&cell, 3, 2));

        cell_dep_remove(&mut cell, 3, 2);
        assert!(!cell_contains(&cell, 3, 2));
        assert!(cell_contains(&cell, 2, 1)); // B1 should still be there

        // Add more cells , such that it converts to set
        for i in 4..=11 {
            cell_dep_insert(&mut cell, i, i);
        }

        // Check that Set was created by verifying format
        match &cell.dependents {
            Dependents::Vector(_) => panic!("Should have converted to Set"),
            Dependents::Set(_) => {} // This is expected
            Dependents::None => panic!("Should not be None"),
        }

        // remove some dependents
        cell_dep_remove(&mut cell, 4, 4);
        cell_dep_remove(&mut cell, 5, 5);
        cell_dep_remove(&mut cell, 6, 6);

        // check
        assert!(!cell_contains(&cell, 4, 4));
        assert!(!cell_contains(&cell, 5, 5));
        assert!(!cell_contains(&cell, 6, 6));
        assert!(cell_contains(&cell, 7, 7));
        assert!(cell_contains(&cell, 8, 8));
        assert!(cell_contains(&cell, 9, 9));
        assert!(cell_contains(&cell, 10, 10));
        assert!(cell_contains(&cell, 11, 11));
    }

    #[test]
    fn test_creating_multiple_cells() {
        let mut cell1 = cell_create(1, 1);
        let mut cell2 = cell_create(2, 3);

        assert_eq!(cell1.row, 1);
        assert_eq!(cell1.col, 1);
        assert_eq!(cell2.row, 2);
        assert_eq!(cell2.col, 3);

        cell_dep_insert(&mut cell1, 2, 1); // B1
        cell_dep_insert(&mut cell2, 1, 1); // A1
        cell_dep_insert(&mut cell2, 10, 24); // X10

        // Verify each cell has its own dependencies
        assert!(cell_contains(&cell1, 2, 1));
        assert!(!cell_contains(&cell2, 2, 1));
        assert!(!cell_contains(&cell1, 1, 1));
        assert!(cell_contains(&cell2, 1, 1));
        assert!(cell_contains(&cell2, 10, 24));
    }

    #[test]
    fn test_dependent_conversion() {
        let mut cell = cell_create(1, 1);

        // Add 9 dependencies to trigger conversion from Vector to Set
        for i in 1..=9 {
            cell_dep_insert(&mut cell, i, i);
        }

        // Check that Set was created by verifying format
        match &cell.dependents {
            Dependents::Vector(_) => panic!("Should have converted to Set"),
            Dependents::Set(_) => {} // This is expected
            Dependents::None => panic!("Should not be None"),
        }

        // Ensure all dependencies are still accessible
        for i in 1..=9 {
            assert!(cell_contains(&cell, i, i));
        }

        // Add one more and check it works
        cell_dep_insert(&mut cell, 10, 10);
        assert!(cell_contains(&cell, 10, 10));

        // Remove one and check it works
        cell_dep_remove(&mut cell, 5, 5);
        assert!(!cell_contains(&cell, 5, 5));
        assert!(cell_contains(&cell, 6, 6));
    }
}
