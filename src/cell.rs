/// Module for handling individual spreadsheet cells and their dependencies.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::spreadsheet::ParsedRHS; // Using BTreeSet as an AVL-tree-like ordered collection

/// Represents a cell in a spreadsheet with its value, formula, and dependency information.
///
/// Each cell knows its position, current value, and which cells depend on it for calculations.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Cell {
    /// Row index of the cell (1-based)
    pub row: i16,
    /// Column index of the cell (1-based)
    pub col: i16,
    /// Whether the cell contains an error
    pub error: bool,
    /// Current numeric value of the cell
    pub value: i32,
    /// Formula defining how the cell's value is calculated
    pub formula: ParsedRHS,
    /// Collection of cells that depend on this cell
    pub dependents: Dependents,
}

/// Represents the collection of cells that depend on a particular cell.
///
/// This enum provides optimizations for different numbers of dependencies:
/// - None for cells with no dependents
/// - Vector for cells with few dependents (more efficient for small numbers)
/// - Set for cells with many dependents (more efficient for lookups in large collections)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Dependents {
    /// A vector of (row, column) pairs for efficient storage of small numbers of dependents
    Vector(Vec<(i16, i16)>),
    /// A sorted tree set for efficient lookups with many dependents
    Set(BTreeSet<(i16, i16)>),
    /// No dependencies
    None,
}

impl Cell {
    /// Creates a new cell at the specified row and column.
    ///
    /// The cell is initialized with a value of 0, no error, no formula, and no dependents.
    ///
    /// # Arguments
    /// * `row` - The row index (1-based)
    /// * `col` - The column index (1-based)
    ///
    /// # Returns
    /// A new Cell initialized at the specified position with default values.
    ///
    /// # Default Values
    /// - value: 0
    /// - error: false
    /// - formula: None
    /// - dependents: None
    ///
    /// # Example
    /// ```
    /// let cell = Cell::create(1, 1);
    /// assert_eq!(cell.row, 1);
    /// assert_eq!(cell.col, 1);
    /// assert_eq!(cell.value, 0);
    /// assert_eq!(cell.error, false);
    /// ```
    pub fn create(row: i16, col: i16) -> Self {
        Cell {
            row,
            col,
            value: 0,
            error: false,
            formula: ParsedRHS::None,
            dependents: Dependents::None,
        }
    }

    /// Adds a dependency to this cell.
    ///
    /// Records that the cell at (row, col) depends on this cell's value.
    /// Automatically upgrades from None to Vector to Set as needed for performance.
    ///
    /// # Arguments
    /// * `row` - Row of the dependent cell
    /// * `col` - Column of the dependent cell
    ///
    /// # Data Structure Optimization
    /// This method implements a dynamic optimization strategy for tracking dependents:
    /// - For 0 dependents: Uses `Dependents::None` (most memory efficient)
    /// - For 1-7 dependents: Uses `Dependents::Vector` (fast for small numbers)
    /// - For 8+ dependents: Upgrades to `Dependents::Set` (faster lookup for many items)
    ///
    /// # Performance Considerations
    /// - Vector is more efficient for small numbers of dependents (faster iteration)
    /// - BTreeSet is more efficient for larger numbers of dependents (faster lookup)
    /// - The threshold of 7 was chosen based on empirical performance testing
    pub fn dep_insert(&mut self, row: i16, col: i16) {
        // Set the initialised flag to 1 whenever a dependency is added
        let key = (row, col);

        match &mut self.dependents {
            Dependents::None => {
                let v = vec![key];
                self.dependents = Dependents::Vector(v);
            }
            Dependents::Vector(vec) => {
                if vec.len() > 7 {
                    // Convert to OrderedSet
                    let mut set = BTreeSet::new();
                    for item in vec.iter() {
                        set.insert(*item);
                    }
                    set.insert(key);
                    self.dependents = Dependents::Set(set);
                    // self.container = 1;
                } else {
                    vec.push(key);
                }
            }
            Dependents::Set(set) => {
                set.insert(key);
            }
        }
    }

    /// Removes a dependency from this cell.
    ///
    /// Removes the record that the cell at (row, col) depends on this cell's value.
    ///
    /// # Arguments
    /// * `row` - Row of the no-longer-dependent cell
    /// * `col` - Column of the no-longer-dependent cell
    ///
    /// # Behavior
    /// - For `Dependents::Vector`: Uses retain to filter out the specified cell
    /// - For `Dependents::Set`: Uses the set's remove method
    /// - For `Dependents::None`: Does nothing
    ///
    /// # Note
    /// This method does not downgrade from Set to Vector or Vector to None
    /// when dependents are removed, as this would add complexity with minimal benefit.
    pub fn dep_remove(&mut self, row: i16, col: i16) {
        let key = (row, col);
        match &mut self.dependents {
            Dependents::Vector(vec) => {
                vec.retain(|k| k != &key);
            }
            Dependents::Set(set) => {
                set.remove(&key);
            }
            Dependents::None => {}
        }
    }

    /// Checks if a cell depends on this cell.
    ///
    /// # Arguments
    /// * `row` - Row of the cell to check
    /// * `col` - Column of the cell to check
    ///
    /// # Returns
    /// `true` if the specified cell depends on this cell, `false` otherwise
    ///
    /// # Performance
    /// - For `Dependents::Vector`: O(n) lookup time
    /// - For `Dependents::Set`: O(log n) lookup time
    /// - For `Dependents::None`: O(1) (always returns false)
    pub fn contains(&self, row: i16, col: i16) -> bool {
        let key = (row, col);
        match &self.dependents {
            Dependents::Vector(vec) => vec.iter().any(|k| k == &key),
            Dependents::Set(set) => set.contains(&key),
            Dependents::None => false,
        }
    }
}

// Public interface functions that match the C API

/// Creates a new cell at the specified row and column.
///
/// This function provides a C-compatible interface for creating cells,
/// returning a Box<Cell> instead of a plain Cell to match external API expectations.
///
/// # Arguments
/// * `row` - The row index (1-based)
/// * `col` - The column index (1-based)
///
/// # Returns
/// A boxed Cell for use with external API calls
///
/// # Example
/// ```
/// let cell = cell_create(1, 1);
/// assert_eq!(cell.row, 1);
/// assert_eq!(cell.col, 1);
/// ```
pub fn cell_create(row: i16, col: i16) -> Box<Cell> {
    Box::new(Cell::create(row, col))
}

/// Adds a dependency to a cell.
///
/// This function provides a C-compatible interface for the Cell::dep_insert method.
/// It records that the cell at (row, col) depends on the given cell's value.
///
/// # Arguments
/// * `cell` - The cell that is being depended on (the dependency)
/// * `row` - Row of the dependent cell (the cell that depends on `cell`)
/// * `col` - Column of the dependent cell (the cell that depends on `cell`)
///
/// # Example
/// ```
/// let mut cell_a1 = cell_create(1, 1);
/// // Record that B2 depends on A1
/// cell_dep_insert(&mut cell_a1, 2, 2);
/// ```
pub fn cell_dep_insert(cell: &mut Cell, row: i16, col: i16) {
    cell.dep_insert(row, col);
}

/// Removes a dependency from a cell.
///
/// This function provides a C-compatible interface for the Cell::dep_remove method.
/// It removes the record that the cell at (row, col) depends on the given cell's value.
///
/// # Arguments
/// * `cell` - The cell that was being depended on (the dependency)
/// * `row` - Row of the no-longer-dependent cell
/// * `col` - Column of the no-longer-dependent cell
///
/// # Example
/// ```
/// let mut cell_a1 = cell_create(1, 1);
/// // Record that B2 no longer depends on A1
/// cell_dep_remove(&mut cell_a1, 2, 2);
/// ```
pub fn cell_dep_remove(cell: &mut Cell, row: i16, col: i16) {
    cell.dep_remove(row, col);
}

/// Checks if a cell depends on the given cell.
///
/// This function provides a C-compatible interface for the Cell::contains method.
///
/// # Arguments
/// * `cell` - The cell that may be depended on
/// * `row` - Row of the cell to check
/// * `col` - Column of the cell to check
///
/// # Returns
/// `true` if the specified cell depends on the given cell, `false` otherwise
///
/// # Example
/// ```
/// let mut cell_a1 = cell_create(1, 1);
/// cell_dep_insert(&mut cell_a1, 2, 2);
/// assert!(cell_contains(&cell_a1, 2, 2));
/// assert!(!cell_contains(&cell_a1, 3, 3));
/// ```
pub fn cell_contains(cell: &Cell, row: i16, col: i16) -> bool {
    cell.contains(row, col)
}
