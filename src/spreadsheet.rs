/// Module for spreadsheet functionality including cell management, formula evaluation and dependency tracking.
use crate::cell::{Cell, cell_create};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

lazy_static! {
    /// Regular expression for matching function syntax, e.g., SUM(A1:B2)
    static ref FUNC_REGEX: Regex = Regex::new(r"^([A-Za-z]+)\((.*)\)$").unwrap();
    /// Regular expression for matching arithmetic expressions, e.g., A1+B2 or 10-5
    static ref ARITH_EXPR_REGEX: Regex = Regex::new(
        r"^(([+-]?[0-9]+)|([A-Za-z]+[0-9]+))([+\-*/])(([+-]?[0-9]+)|([A-Za-z]+[0-9]+))$"
    )
    .unwrap();
}

/// Represents a spreadsheet with cells, dimensions, and view settings.
///
/// The spreadsheet tracks cell values, formulas, and dependencies between cells.
/// It also maintains an undo stack for operation history.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Spreadsheet {
    /// Number of rows in the spreadsheet
    pub rows: i16,
    /// Number of columns in the spreadsheet
    pub cols: i16,
    /// Current top row of the view (for scrolling)
    pub view_row: i16,
    /// Current leftmost column of the view (for scrolling)
    pub view_col: i16,
    /// Matrix of cells stored as a flat vector
    pub cells: Vec<Option<Box<Cell>>>,
    /// Stack of previous cell states for undo functionality
    pub undo_stack: Vec<(ParsedRHS, i16, i16)>,
}

/// Represents the parsed right-hand side of a cell formula.
///
/// This enum captures the various types of expressions that can be used
/// in a spreadsheet cell, such as functions, arithmetic operations, or simple values.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub enum ParsedRHS {
    /// A function with a name and arguments
    Function {
        name: FunctionName,
        args: (Operand, Operand),
    },
    /// A sleep operation with a duration
    Sleep(Operand),
    /// An arithmetic operation with left-hand side, operator, and right-hand side
    Arithmetic {
        lhs: Operand,
        operator: char,
        rhs: Operand,
    },
    /// A single value (number or cell reference)
    SingleValue(Operand),
    /// No operation
    None,
}

/// Represents an operand in a formula, which can be a number or a cell reference.
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Operand {
    /// A numeric value
    Number(i32),
    /// A cell reference with row and column
    Cell(i16, i16),
}

/// Represents the name of a function that can be used in a formula.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum FunctionName {
    Min,
    Max,
    Avg,
    Sum,
    Stdev,
    Copy,
}

impl FunctionName {
    /// Converts a string to a FunctionName enum variant.
    pub fn from_strng(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "MIN" => Some(FunctionName::Min),
            "MAX" => Some(FunctionName::Max),
            "AVG" => Some(FunctionName::Avg),
            "SUM" => Some(FunctionName::Sum),
            "STDEV" => Some(FunctionName::Stdev),
            "COPY" => Some(FunctionName::Copy),
            _ => None,
        }
    }
    /// Checks if the function is a copy operation.
    pub fn is_copy(&self) -> bool {
        matches!(self, FunctionName::Copy)
    }
}

impl Spreadsheet {
    /// Creates a new spreadsheet with the specified number of rows and columns.
    ///
    /// This function initializes a new spreadsheet with the given dimensions and creates
    /// all cells within the specified range. Each cell is created with default values.
    ///
    /// # Arguments
    /// * `rows` - Number of rows in the spreadsheet
    /// * `cols` - Number of columns in the spreadsheet
    ///
    /// # Returns
    /// * `Some(Box<Self>)` - A boxed Spreadsheet instance if creation was successful
    /// * `None` - If creation failed (should not occur under normal circumstances)
    ///
    /// # Example
    /// ```  let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
    /// assert_eq!(sheet.rows, 10);
    /// assert_eq!(sheet.cols, 10);
    /// ```
    pub fn spreadsheet_create(rows: i16, cols: i16) -> Option<Box<Self>> {
        let mut sheet = Box::new(Spreadsheet {
            rows,
            cols,
            view_row: 0,
            view_col: 0,
            cells: Vec::with_capacity(rows as usize * cols as usize),
            undo_stack: Vec::new(),
        });

        for _ in 0..(rows as usize * cols as usize) {
            sheet.cells.push(None);
        }

        for r in 1..=rows {
            for c in 1..=cols {
                let index = ((r - 1) as usize) * (cols as usize) + ((c - 1) as usize);
                sheet.cells[index] = Some(cell_create(r, c));
            }
        }

        Some(sheet)
    }

    /// Converts a column number to its corresponding letter representation.
    ///
    /// This function converts a 1-based column index to an Excel-style column name.
    /// For example, 1 becomes "A", 2 becomes "B", 27 becomes "AA", etc.
    ///
    /// # Arguments
    /// * `col` - The column number (1-based)
    ///
    /// # Returns
    /// A string representing the column letter(s)
    ///
    /// # Example
    /// ``` assert_eq!(Spreadsheet::col_to_letter(1), "A");
    /// assert_eq!(Spreadsheet::col_to_letter(26), "Z");
    /// assert_eq!(Spreadsheet::col_to_letter(27), "AA");
    /// ```
    pub fn col_to_letter(col: i16) -> String {
        let mut col = col;
        let mut result = String::new();
        while col > 0 {
            col -= 1;
            result.push(((col % 26) as u8 + b'A') as char);
            col /= 26;
        }
        result.chars().rev().collect()
    }

    /// Converts a column letter representation to its corresponding number.
    ///
    /// This function converts an Excel-style column name to a 1-based column index.
    /// For example, "A" becomes 1, "B" becomes 2, "AA" becomes 27, etc.
    ///
    /// # Arguments
    /// * `letters` - The string containing the column letters
    ///
    /// # Returns
    /// The column number (1-based)
    ///
    /// # Example
    /// ``` assert_eq!(Spreadsheet::letter_to_col("A"), 1);
    /// assert_eq!(Spreadsheet::letter_to_col("Z"), 26);
    /// assert_eq!(Spreadsheet::letter_to_col("AA"), 27);
    /// ```
    pub fn letter_to_col(letters: &str) -> i16 {
        letters
            .chars()
            .fold(0, |acc, c| acc * 26 + (c as i16 - 'A' as i16 + 1))
    }

    /// Returns the cell name for the given row and column.
    ///
    /// This function formats a row and column into a standard spreadsheet cell reference
    /// (e.g., "A1", "B2", "AA10").
    ///
    /// # Arguments
    /// * `row` - The row number (1-based)
    /// * `col` - The column number (1-based)
    ///
    /// # Returns
    /// A string containing the formatted cell name
    ///
    /// # Example
    /// ```assert_eq!(Spreadsheet::get_cell_name(1, 1), "A1");
    /// assert_eq!(Spreadsheet::get_cell_name(10, 2), "B10");
    /// ```
    pub fn get_cell_name(row: i16, col: i16) -> String {
        format!("{}{}", Self::col_to_letter(col), row)
    }

    /// Parses a cell name and returns its row and column.
    ///
    /// This function takes a cell reference (e.g., "A1", "B10") and converts it to
    /// row and column indices. It also validates that the referenced cell exists within
    /// the spreadsheet's dimensions.
    ///
    /// # Arguments
    /// * `cell_name` - The cell reference (e.g., "A1", "B10")
    ///
    /// # Returns
    /// * `Some((row, col))` - The row and column indices if the cell name is valid
    /// * `None` - If the cell name is invalid or refers to a cell outside the spreadsheet
    ///
    /// # Example
    /// ```let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
    /// assert_eq!(sheet.spreadsheet_parse_cell_name("A1"), Some((1, 1)));
    /// assert_eq!(sheet.spreadsheet_parse_cell_name("B10"), Some((10, 2)));
    /// assert_eq!(sheet.spreadsheet_parse_cell_name("K11"), None); // Outside dimensions
    /// ```
    pub fn spreadsheet_parse_cell_name(&self, cell_name: &str) -> Option<(i16, i16)> {
        let mut letters = String::new();
        let mut digits = String::new();
        let mut found_digit = false;

        for c in cell_name.chars() {
            if c.is_ascii_alphabetic() {
                if found_digit {
                    return None;
                }
                letters.push(c);
            } else if c.is_ascii_digit() {
                found_digit = true;
                digits.push(c);
            } else {
                return None;
            }
        }

        if letters.is_empty() || digits.is_empty() {
            return None;
        }

        let col = Self::letter_to_col(&letters);
        let row = digits.parse::<i16>().ok()?;

        if col > self.cols || row > self.rows || row == 0 {
            return None;
        }
        Some((row, col))
    }

    /// Checks if a string is numeric.
    ///
    /// This utility function checks if a string contains only numeric digits.
    /// It returns false for empty strings or strings with non-digit characters.
    ///
    /// # Arguments
    /// * `s` - The string to check
    ///
    /// # Returns
    /// `true` if the string contains only digits, `false` otherwise
    ///
    /// # Example
    /// ```assert!(Spreadsheet::is_numeric("123"));
    /// assert!(!Spreadsheet::is_numeric("12a"));
    /// assert!(!Spreadsheet::is_numeric(""));
    /// ```
    pub fn is_numeric(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
    }

    /// Evaluates a parsed right-hand side expression and returns its value and error status.
    ///
    /// This complex function evaluates different types of expressions:
    /// - Functions like SUM, MIN, MAX, AVG, STDEV
    /// - Sleep operations
    /// - Arithmetic expressions
    /// - Single value references
    ///
    /// It handles cell references, numeric values, and produces appropriate error states.
    ///
    /// # Arguments
    /// * `expr` - The parsed expression to evaluate
    /// * `_row` - Row of the cell containing the expression (for context)
    /// * `_col` - Column of the cell containing the expression (for context)
    ///
    /// # Returns
    /// A tuple containing:
    /// * The calculated value
    /// * Whether an error occurred during evaluation
    ///
    /// # Error Handling
    /// Returns an error state (second tuple element = true) for:
    /// - Division by zero
    /// - References to cells in error state
    /// - Invalid operations
    pub fn spreadsheet_evaluate_expression(
        &self,
        expr: &ParsedRHS,
        _row: i16,
        _col: i16,
    ) -> (i32, bool) {
        match expr {
            ParsedRHS::Function { name, args } => {
                let mut error = false;
                let (arg1, arg2) = args;
                let (r1, c1) = match arg1 {
                    Operand::Cell(r, c) => (*r, *c),
                    Operand::Number(_) => (0, 0),
                };
                let (r2, c2) = match arg2 {
                    Operand::Cell(r, c) => (*r, *c),
                    Operand::Number(_) => (0, 0),
                };

                let mut values =
                    Vec::with_capacity((r2 - r1 + 1) as usize * (c2 - c1 + 1) as usize);

                for i in r1..=r2 {
                    for j in c1..=c2 {
                        let index = (i - 1) as usize * self.cols as usize + (j - 1) as usize;
                        if index < self.cells.len() {
                            if let Some(ref c) = self.cells[index] {
                                if c.error {
                                    error = true;
                                    return (0, error);
                                }
                                values.push(c.value);
                            }
                        }
                    }
                }

                match name {
                    FunctionName::Min => {
                        error = false;
                        return (*values.iter().min().unwrap_or(&0), error);
                    }
                    FunctionName::Max => {
                        error = false;
                        return (*values.iter().max().unwrap_or(&0), error);
                    }
                    FunctionName::Sum => {
                        error = false;
                        return (values.iter().sum(), error);
                    }
                    FunctionName::Avg => {
                        error = false;
                        let sum: i32 = values.iter().sum();
                        return (sum / values.len() as i32, error);
                    }
                    FunctionName::Stdev => {
                        if values.len() < 2 {
                            return (0, error);
                        }

                        let mean = values.iter().sum::<i32>() as f64 / values.len() as f64;
                        let variance = values
                            .iter()
                            .map(|&x| {
                                let diff = x as f64 - mean;
                                diff * diff
                            })
                            .sum::<f64>()
                            / values.len() as f64;

                        error = false;
                        return ((variance.sqrt().round()) as i32, error);
                    }
                    _ => {}
                }

                (0, error)
            }
            ParsedRHS::Sleep(op) => {
                let mut val = 0;
                let error = false;

                match op {
                    Operand::Number(n) => {
                        val = *n;
                    }
                    Operand::Cell(r, c) => {
                        let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
                        if let Some(cell) = self.cells.get(index).and_then(|c| c.as_ref()) {
                            val = cell.value;
                            if cell.error {
                                return (val, true);
                            }
                        }
                    }
                }

                if val > 0 {
                    std::thread::sleep(std::time::Duration::from_secs(val as u64));
                }

                (val, error)
            }
            ParsedRHS::Arithmetic { lhs, operator, rhs } => {
                let (lhs_val, lhs_err) = match lhs {
                    Operand::Number(n) => (*n, false),
                    Operand::Cell(r, c) => {
                        let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
                        self.cells[index]
                            .as_ref()
                            .map_or((0, true), |cell| (cell.value, cell.error))
                    }
                };

                let (rhs_val, rhs_err) = match rhs {
                    Operand::Number(n) => (*n, false),
                    Operand::Cell(r, c) => {
                        let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
                        self.cells[index]
                            .as_ref()
                            .map_or((0, true), |cell| (cell.value, cell.error))
                    }
                };

                let mut has_error = lhs_err || rhs_err;

                let result = match operator {
                    '+' => lhs_val + rhs_val,
                    '-' => lhs_val - rhs_val,
                    '*' => lhs_val * rhs_val,
                    '/' => {
                        if rhs_val == 0 {
                            has_error = true;
                            0
                        } else {
                            lhs_val / rhs_val
                        }
                    }
                    _ => {
                        has_error = true;
                        0
                    }
                };

                (result, has_error)
            }

            ParsedRHS::SingleValue(num) => match num {
                Operand::Cell(r, c) => {
                    let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
                    self.cells[index]
                        .as_ref()
                        .map_or((0, false), |cell| (cell.value, cell.error))
                }
                Operand::Number(x) => (*x, false),
            },
            ParsedRHS::None => (0, false),
        }
    }

    /// Recursively finds cycles in the dependency graph using a stack.
    ///
    /// This function implements cycle detection in the cell dependency graph to prevent
    /// circular references. It uses a stack-based approach rather than recursion for
    /// better performance with deep dependency chains.
    ///
    /// # Arguments
    /// * `(r1, r2)` - Row range to check
    /// * `(c1, c2)` - Column range to check
    /// * `range_bool` - Whether checking a range (true) or specific cells (false)
    /// * `visited` - Set of already visited cells to prevent re-processing
    /// * `stack` - Stack of cells to process
    ///
    /// # Returns
    /// `true` if a cycle is detected, `false` otherwise
    pub fn rec_find_cycle_using_stack<'a>(
        &'a self,
        (r1, r2): (i16, i16),
        (c1, c2): (i16, i16),
        range_bool: bool,
        visited: &mut BTreeSet<(i16, i16)>,
        stack: &mut Vec<&'a Cell>,
    ) -> bool {
        while let Some(my_node) = stack.pop() {
            if visited.contains(&(my_node.row, my_node.col)) {
                continue;
            }
            visited.insert((my_node.row, my_node.col));
            let in_range = if range_bool {
                my_node.row >= r1 && my_node.row <= r2 && my_node.col >= c1 && my_node.col <= c2
            } else {
                (my_node.row == r1 && my_node.col == c1) || (my_node.row == r2 && my_node.col == c2)
            };

            if in_range {
                return true;
            } else {
                let dependent_names = self.get_dependent_names(my_node);
                for dependent_name in &dependent_names {
                    if !visited.contains(dependent_name) {
                        let r = dependent_name.0;
                        let c = dependent_name.1;
                        let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
                        if index < self.cells.len() {
                            if let Some(ref neighbor_node) = self.cells[index] {
                                stack.push(neighbor_node);
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Returns the names of cells that depend on the given cell.
    ///
    /// This function extracts the list of dependent cells from a cell's dependency tracker,
    /// handling the different internal representations (Vector, Set, None) transparently.
    ///
    /// # Arguments
    /// * `cell` - The cell to get dependents for
    ///
    /// # Returns
    /// A vector of (row, column) pairs representing the cells that depend on the given cell
    pub fn get_dependent_names(&self, cell: &Cell) -> Vec<(i16, i16)> {
        match &cell.dependents {
            crate::cell::Dependents::Vector(vec) => vec.clone(),
            crate::cell::Dependents::Set(set) => set.iter().cloned().collect(),
            crate::cell::Dependents::None => Vec::new(),
        }
    }

    /// Initiates the cycle detection process for a given cell and range.
    ///
    /// This function sets up the cycle detection by creating initial state and delegating
    /// to the stack-based cycle detection algorithm. It's used to check if adding a
    /// dependency would create a circular reference.
    ///
    /// # Arguments
    /// * `(r_, c_)` - The cell initiating the dependency
    /// * `(r1, c1)` - Start of range or first specific cell
    /// * `(r2, c2)` - End of range or second specific cell
    /// * `range_bool` - Whether checking a range (true) or specific cells (false)
    ///
    /// # Returns
    /// `true` if a cycle would be created, `false` otherwise
    pub fn first_step_find_cycle(
        &self,
        (r_, c_): (i16, i16),
        (r1, c1): (i16, i16),
        (r2, c2): (i16, i16),
        range_bool: bool,
    ) -> bool {
        let index = (r_ - 1) as usize * self.cols as usize + (c_ - 1) as usize;

        let start_node = self.cells[index].as_ref().unwrap();

        let mut visited = BTreeSet::new();
        let mut stack = vec![&**start_node];

        self.rec_find_cycle_using_stack((r1, r2), (c1, c2), range_bool, &mut visited, &mut stack)
    }

    /// Removes old dependencies for a cell.
    ///
    /// This function cleans up existing dependencies before assigning a new formula to a cell.
    /// It examines the cell's current formula and removes the cell from the dependents lists
    /// of all cells it currently depends on.
    ///
    /// # Arguments
    /// * `r` - Row of the cell having dependencies removed
    /// * `c` - Column of the cell having dependencies removed
    pub fn remove_old_dependents(&mut self, r: i16, c: i16) {
        let formula = {
            let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
            let curr_cell = self.cells[index].as_ref().unwrap();
            curr_cell.formula.clone()
        };

        match formula {
            ParsedRHS::Function { name, args } => {
                if !name.is_copy() {
                    let (arg1, arg2) = args;
                    let (start_row, start_col) = match arg1 {
                        Operand::Cell(row, col) => (row, col),
                        Operand::Number(_) => (0, 0),
                    };
                    let (end_row, end_col) = match arg2 {
                        Operand::Cell(row, col) => (row, col),
                        Operand::Number(_) => (0, 0), // Placeholder
                    };

                    for dep_r in start_row..=end_row {
                        for dep_c in start_col..=end_col {
                            let dep_index =
                                (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;

                            if let Some(dep_cell) =
                                self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut())
                            {
                                crate::cell::cell_dep_remove(dep_cell, r, c);
                            }
                        }
                    }
                }
            }
            ParsedRHS::Arithmetic {
                lhs,
                operator: _,
                rhs,
            } => {
                if let Operand::Cell(dep_r, dep_c) = lhs {
                    let dep_cell = self.cells
                        [(dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize]
                        .as_mut()
                        .unwrap();
                    crate::cell::cell_dep_remove(dep_cell, r, c);
                }
                if let Operand::Cell(dep_r, dep_c) = rhs {
                    let dep_cell = self.cells
                        [(dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize]
                        .as_mut()
                        .unwrap();
                    crate::cell::cell_dep_remove(dep_cell, r, c);
                }
            }
            ParsedRHS::SingleValue(Operand::Cell(dep_r, dep_c)) => {
                let dep_index = (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;
                let dep_cell = self.cells[dep_index].as_mut().unwrap();
                crate::cell::cell_dep_remove(dep_cell, r, c);
            }
            ParsedRHS::Sleep(Operand::Cell(dep_r, dep_c)) => {
                let dep_index = (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;

                let dep_cell = self.cells[dep_index].as_mut().unwrap();
                crate::cell::cell_dep_remove(dep_cell, r, c);
            }
            _ => {}
        }
    }

    /// Updates the dependencies for a cell.
    ///
    /// This function establishes new dependencies after a cell's formula changes.
    /// It first removes old dependencies, then adds the cell as a dependent to all
    /// cells it now depends on based on its new formula.
    ///
    /// # Arguments
    /// * `(r, c)` - The cell being updated
    /// * `(start_row, start_col)` - Start of range or first specific cell
    /// * `(end_row, end_col)` - End of range or second specific cell
    /// * `is_range` - Whether dealing with a range (true) or specific cells (false)
    ///
    /// # Returns
    /// Always returns 0 (legacy return value maintained for compatibility)
    pub fn update_dependencies(
        &mut self,
        (r, c): (i16, i16),
        (start_row, start_col): (i16, i16),
        (end_row, end_col): (i16, i16),
        is_range: bool,
    ) -> i32 {
        self.remove_old_dependents(r, c);

        if is_range {
            for r_it in start_row..=end_row {
                for c_it in start_col..=end_col {
                    let dep_index = (r_it - 1) as usize * self.cols as usize + (c_it - 1) as usize;
                    let dep_cell = self.cells[dep_index].as_mut().unwrap();
                    crate::cell::cell_dep_insert(dep_cell, r, c);
                }
            }
        } else {
            if start_row > 0 {
                let dep_index =
                    (start_row - 1) as usize * self.cols as usize + (start_col - 1) as usize;
                let dep_cell = self.cells[dep_index].as_mut().unwrap();
                crate::cell::cell_dep_insert(dep_cell, r, c);
            }
            if end_row > 0 {
                let dep_index =
                    (end_row - 1) as usize * self.cols as usize + (end_col - 1) as usize;

                let dep_cell = self.cells[dep_index].as_mut().unwrap();
                crate::cell::cell_dep_insert(dep_cell, r, c);
            }
        }
        0
    }

    /// Performs a topological sort on the dependency graph starting from a given cell.
    ///
    /// This function sorts cells in dependency order, ensuring that cells are evaluated
    /// only after all their dependencies have been evaluated. This is crucial for
    /// correctly propagating value changes through the dependency chain.
    ///
    /// # Arguments
    /// * `starting` - The cell to start the topological sort from
    ///
    /// # Returns
    /// A boxed vector of (row, column) pairs in topological order
    pub fn topo_sort(&self, starting: &Cell) -> Box<Vec<(i16, i16)>> {
        let mut sorted_nodes = Box::new(Vec::new());
        let mut stack = Box::new(Vec::new());
        stack.push(starting.clone());

        let mut visited = Box::new(BTreeSet::new());

        let mut work_stack = Box::new(Vec::new());
        work_stack.push(Box::new((starting.row, starting.col)));

        while let Some(current) = work_stack.pop() {
            let current = *current;
            if visited.contains(&current) {
                continue;
            }

            let index = (current.0 - 1) as usize * self.cols as usize + (current.1 - 1) as usize;
            if let Some(cell) = self.cells.get(index).and_then(|opt| opt.as_ref()) {
                let dependent_keys = self.get_dependent_names(cell);
                let mut all_dependents_visited = true;

                for dep_key in &dependent_keys {
                    if !visited.contains(dep_key) {
                        let (r, c) = *dep_key;
                        work_stack.push(Box::new(current));
                        work_stack.push(Box::new((r, c)));
                        all_dependents_visited = false;
                        break;
                    }
                }

                if all_dependents_visited {
                    visited.insert(current);
                    sorted_nodes.push(current);
                }
            }
        }
        sorted_nodes.reverse();
        sorted_nodes
    }

    /// Sets the value of a cell and updates dependencies and dependent cells.
    ///
    /// This is the main function for updating a cell's formula. It:
    /// 1. Handles special cases like COPY function
    /// 2. Checks for circular references
    /// 3. Updates dependencies
    /// 4. Evaluates the new formula
    /// 5. Propagates changes to dependent cells
    /// 6. Updates the undo stack
    ///
    /// # Arguments
    /// * `row` - Row of the cell to update
    /// * `col` - Column of the cell to update
    /// * `rhs` - The new formula for the cell
    /// * `status_out` - Output parameter for operation status message
    ///
    /// # Side Effects
    /// - Updates the cell's value and error state
    /// - Updates dependent cells' values
    /// - Adds to the undo stack
    /// - Modifies `status_out` to indicate success or failure
    pub fn spreadsheet_set_cell_value(
        &mut self,
        row: i16,
        col: i16,
        rhs: ParsedRHS,
        status_out: &mut String,
    ) {
        let index = (row - 1) as usize * self.cols as usize + (col - 1) as usize;

        if let ParsedRHS::Function {
            name: FunctionName::Copy,
            args: (Operand::Cell(start_row, start_col), Operand::Cell(end_row, end_col)),
        } = rhs
        {
            let dest_row = row;
            let dest_col = col;
            let row_offset = dest_row as isize - start_row as isize;
            let col_offset = dest_col as isize - start_col as isize;

            let mut src_val: Vec<i32> = Vec::new();
            let mut src_err: Vec<bool> = Vec::new();
            for r in start_row..=end_row {
                for c in start_col..=end_col {
                    let src_index = ((r - 1) * self.cols + (c - 1)) as usize;
                    let src_cell = self.cells[src_index].as_ref().unwrap();
                    src_val.push(src_cell.value);
                    src_err.push(src_cell.error);
                }
            }
            let mut cnter = 0;
            for r in start_row..=end_row {
                for c in start_col..=end_col {
                    self.update_dependencies((r, c), (0, 0), (0, 0), false);
                    let dest_index = ((r as isize + row_offset - 1) * self.cols as isize
                        + (c as isize + col_offset - 1))
                        as usize;
                    if dest_index < self.cells.len() {
                        let dest_cell = self.cells[dest_index].as_mut().unwrap();
                        self.undo_stack.push((
                            dest_cell.formula.clone(),
                            dest_cell.row,
                            dest_cell.col,
                        ));
                        dest_cell.value = src_val[cnter];
                        dest_cell.formula =
                            ParsedRHS::SingleValue(Operand::Number(dest_cell.value));
                        dest_cell.error = src_err[cnter];
                        cnter += 1;
                    }
                }
            }
            *status_out = "ok".to_string();
            return;
        }

        let mut r1 = 0;
        let mut r2 = 0;
        let mut c1 = 0;
        let mut c2 = 0;
        let mut is_range = false;
        match &rhs {
            ParsedRHS::Function {
                args: (Operand::Cell(w, x), Operand::Cell(y, z)),
                ..
            } => {
                r1 = *w;
                r2 = *y;
                c1 = *x;
                c2 = *z;
                is_range = true;
            }
            ParsedRHS::Arithmetic { lhs, rhs, .. } => {
                if let Operand::Cell(w, x) = lhs {
                    r1 = *w;
                    c1 = *x;
                }
                if let Operand::Cell(y, z) = rhs {
                    r2 = *y;
                    c2 = *z;
                }
            }
            ParsedRHS::Sleep(Operand::Cell(r, c)) => {
                r1 = *r;
                c1 = *c;
            }
            ParsedRHS::SingleValue(Operand::Cell(r, c)) => {
                r1 = *r;
                c1 = *c;
            }
            _ => {}
        };

        if self.first_step_find_cycle((row, col), (r1, c1), (r2, c2), is_range) {
            *status_out = "Cycle Detected".to_string();
            return;
        }

        self.update_dependencies((row, col), (r1, c1), (r2, c2), is_range);
        let cell = self.cells[index].as_mut().unwrap();
        self.undo_stack
            .push((cell.formula.clone(), cell.row, cell.col));

        cell.formula = rhs;
        let cell = self.cells[index].as_ref().unwrap();

        let sorted_cells = self.topo_sort(cell);

        for (row, col) in sorted_cells.iter() {
            let sorted_index = (*row - 1) as usize * self.cols as usize + (*col - 1) as usize;

            let formula = &self.cells[sorted_index].as_ref().unwrap().formula;

            let (value, error_cell) = self.spreadsheet_evaluate_expression(formula, *row, *col);

            let sorted_cell = self.cells[sorted_index].as_mut().unwrap();
            sorted_cell.value = value;
            sorted_cell.error = error_cell;
        }

        *status_out = "ok".to_string();
    }

    /// Undoes the last operation by restoring the previous cell states.
    ///
    /// This function reverts the spreadsheet to its previous state by popping operations
    /// from the undo stack and applying them in reverse. It handles the entire undo operation
    /// as a single atomic action.
    ///
    /// # Arguments
    /// * `status_out` - Output parameter for operation status message
    ///
    /// # Side Effects
    /// - Modifies multiple cells back to their previous states
    /// - Clears the current undo stack
    /// - Rebuilds dependencies based on the restored formulas
    pub fn spreadsheet_undo(&mut self, status_out: &mut String) {
        let mut undo_stack = self.undo_stack.clone();
        self.undo_stack.clear();
        for _ in 0..undo_stack.len() {
            let (formula_new, row, col) = undo_stack.pop().unwrap();

            self.spreadsheet_set_cell_value(row, col, formula_new, status_out);
        }
    }

    /// Displays the current state of the spreadsheet.
    ///
    /// This function prints a formatted view of the spreadsheet to the console,
    /// showing a window of cells based on the current view_row and view_col settings.
    /// It displays at most 10 rows and 10 columns at a time.
    ///
    /// # Format
    /// - Column headers are shown as letters (A, B, C, ...)
    /// - Row headers are shown as numbers (1, 2, 3, ...)
    /// - Cell values are displayed in the grid
    /// - Cells with errors show "ERR" instead of their value
    pub fn spreadsheet_display(&self) {
        let end_row = if self.view_row + 10 < self.rows {
            self.view_row + 10
        } else {
            self.rows
        };

        let end_col = if self.view_col + 10 < self.cols {
            self.view_col + 10
        } else {
            self.cols
        };

        print!("\t\t");
        for col in (self.view_col + 1)..=end_col {
            print!("{}\t\t", Self::col_to_letter(col));
        }
        println!();
        for row in (self.view_row + 1)..=end_row {
            print!("{}\t\t", row);
            for col in (self.view_col + 1)..=end_col {
                let index = (row - 1) as usize * self.cols as usize + (col - 1) as usize;
                if let Some(cell) = self.cells.get(index).and_then(|opt| opt.as_ref()) {
                    if cell.error {
                        print!("ERR\t\t");
                    } else {
                        print!("{:<16}", cell.value);
                    }
                } else {
                    print!("0\t\t");
                }
            }
            println!();
        }
    }

    /// Checks if a command is valid and returns the parsed result.
    ///
    /// This function parses and validates a cell update command, such as "A1=B1+C1" or
    /// "D5=SUM(A1:A10)". It ensures the cell reference is valid and the formula can be parsed.
    ///
    /// # Arguments
    /// * `cell_name` - The name of the cell to update (e.g., "A1")
    /// * `formula` - The formula to assign to the cell
    ///
    /// # Returns
    /// A tuple containing:
    /// * Whether the command is valid
    /// * The row of the target cell
    /// * The column of the target cell
    /// * The parsed formula
    ///
    /// # Example
    /// ```let sheet = Spreadsheet::spreadsheet_create(10, 10).unwrap();
    /// let (valid, row, col, formula) = sheet.is_valid_command("A1", "10");
    /// assert!(valid);
    /// assert_eq!(row, 1);
    /// assert_eq!(col, 1);
    /// // formula will be ParsedRHS::SingleValue(Operand::Number(10))
    /// ```
    pub fn is_valid_command(&self, cell_name: &str, formula: &str) -> (bool, i16, i16, ParsedRHS) {
        let mut ret = (false, 0, 0, ParsedRHS::None);
        if cell_name.is_empty() || formula.is_empty() {
            return ret;
        }

        if let Some((row, col)) = self.spreadsheet_parse_cell_name(cell_name) {
            ret.1 = row;
            ret.2 = col;
        } else {
            return ret;
        }
        if formula.is_empty() {
            return ret;
        }
        if let Some(captures) = FUNC_REGEX.captures(formula) {
            let func = captures.get(1).unwrap().as_str();
            let args = captures.get(2).unwrap().as_str();

            if func.eq_ignore_ascii_case("SLEEP") {
                if args.is_empty() {
                    return ret;
                }
                if let Ok(value) = args.parse::<i32>() {
                    ret.0 = true;
                    ret.3 = ParsedRHS::Sleep(Operand::Number(value));
                    return ret;
                }
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(args) {
                    ret.0 = true;
                    ret.3 = ParsedRHS::Sleep(Operand::Cell(row, col));
                    return ret;
                }
                return ret;
            } else {
                if let Some(colon_pos) = args.find(':') {
                    let (start, end) = args.split_at(colon_pos);
                    let end = &end[1..];

                    if let (Some((start_row, start_col)), Some((end_row, end_col))) = (
                        self.spreadsheet_parse_cell_name(start),
                        self.spreadsheet_parse_cell_name(end),
                    ) {
                        if start_row <= end_row && start_col <= end_col {
                            if let Some(fname) = FunctionName::from_strng(func) {
                                if func == "COPY" {
                                    let dest_row = ret.1;
                                    let dest_col = ret.2;
                                    let row_offset = dest_row - start_row;
                                    let col_offset = dest_col - start_col;
                                    let final_row = end_row + row_offset;
                                    let final_col = end_col + col_offset;

                                    if final_row > 0
                                        && final_row <= self.rows
                                        && final_col > 0
                                        && final_col <= self.cols
                                    {
                                        ret.0 = true;
                                        ret.3 = ParsedRHS::Function {
                                            name: fname,
                                            args: (
                                                Operand::Cell(start_row, start_col),
                                                Operand::Cell(end_row, end_col),
                                            ),
                                        };
                                        return ret;
                                    } else {
                                        ret.0 = false;
                                        return ret;
                                    }
                                }
                                ret.0 = true;
                                ret.3 = ParsedRHS::Function {
                                    name: fname,
                                    args: (
                                        Operand::Cell(start_row, start_col),
                                        Operand::Cell(end_row, end_col),
                                    ),
                                };
                                return ret;
                            }
                        }
                    }
                }
                return ret;
            }
        }

        if let Some((row, col)) = self.spreadsheet_parse_cell_name(formula) {
            ret.0 = true;
            ret.3 = ParsedRHS::SingleValue(Operand::Cell(row, col));
            return ret;
        }

        if let Ok(value) = formula.parse::<i32>() {
            ret.0 = true;
            ret.3 = ParsedRHS::SingleValue(Operand::Number(value));
            return ret;
        }
        let (b, x) = self.is_valid_arithmetic_expression(formula);
        if b {
            ret.0 = true;
            ret.3 = x;
            ret
        } else {
            ret.0 = false;
            ret
        }
    }

    /// Checks if an arithmetic expression is valid and returns the parsed result.
    ///
    /// This function parses expressions like "A1+B2", "10-5", "C3*D4", or "E5/F6".
    /// It validates the operands and operator, and constructs a ParsedRHS if valid.
    ///
    /// # Arguments
    /// * `expr` - The arithmetic expression to parse
    ///
    /// # Returns
    /// A tuple containing:
    /// * Whether the expression is valid
    /// * The parsed expression as a ParsedRHS
    ///
    /// # Recognized Formats
    /// - Cell reference + Cell reference: "A1+B2"
    /// - Cell reference + Number: "A1+10"
    /// - Number + Cell reference: "10+A1"
    /// - Number + Number: "10+20"
    /// - Operators: +, -, *, /
    pub fn is_valid_arithmetic_expression(&self, expr: &str) -> (bool, ParsedRHS) {
        let mut ret = (false, ParsedRHS::None);
        if let Some(captures) = ARITH_EXPR_REGEX.captures(expr) {
            let first_operand = captures.get(1).unwrap().as_str();
            let operator = captures.get(4).unwrap().as_str();
            let second_operand = captures.get(5).unwrap().as_str();

            let oprnd1 = if first_operand.chars().next().unwrap().is_ascii_alphabetic() {
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(first_operand) {
                    Operand::Cell(row, col)
                } else {
                    return (false, ParsedRHS::None);
                }
            } else if let Ok(value) = first_operand.parse::<i32>() {
                Operand::Number(value)
            } else {
                return (false, ParsedRHS::None);
            };

            let oprnd2 = if second_operand.chars().next().unwrap().is_ascii_alphabetic() {
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(second_operand) {
                    Operand::Cell(row, col)
                } else {
                    return (false, ParsedRHS::None);
                }
            } else if let Ok(value) = second_operand.parse::<i32>() {
                Operand::Number(value)
            } else {
                return (false, ParsedRHS::None);
            };
            if let Some(op_char) = operator.chars().next() {
                if "+-*/".contains(op_char) {
                    ret.0 = true;
                    ret.1 = ParsedRHS::Arithmetic {
                        lhs: oprnd1,
                        operator: op_char,
                        rhs: oprnd2,
                    };
                }
            } else {
                return (false, ParsedRHS::None);
            }

            ret
        } else {
            ret
        }
    }
}
