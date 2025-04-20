use crate::cell::{Cell, cell_create};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
// use std::time::Instant;
use lazy_static::lazy_static;
use regex::Regex;

// Pre-compile regex patterns
lazy_static! {
    static ref FUNC_REGEX: Regex = Regex::new(r"^([A-Za-z]+)\((.*)\)$").unwrap();
    static ref ARITH_EXPR_REGEX: Regex = Regex::new(
        r"^(([+-]?[0-9]+)|([A-Za-z]+[0-9]+))([+\-*/])(([+-]?[0-9]+)|([A-Za-z]+[0-9]+))$"
    )
    .unwrap();
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Spreadsheet {
    pub rows: i16,
    pub cols: i16,
    pub view_row: i16,
    pub view_col: i16,
    pub cells: Vec<Option<Box<Cell>>>,
    pub undo_stack: Vec<(ParsedRHS, i16, i16)>,
    // pub cells: Vec<Vec<Option<Cell>>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum ParsedRHS {
    Function {
        name: FunctionName,
        args: (Operand, Operand), // now each arg can be a cell (row, col) or number
    },
    Sleep(Operand),
    Arithmetic {
        lhs: Operand,
        operator: char,
        rhs: Operand,
    },
    SingleValue(Operand),
    None,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum Operand {
    Number(i32),
    Cell(i16, i16), // (row, col)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum FunctionName {
    Min,
    Max,
    Avg,
    Sum,
    Stdev,
    Cut,
    Copy,
}

impl FunctionName {
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
    pub fn is_copy(&self) -> bool {
        matches!(self, FunctionName::Copy)
    }
}

impl Spreadsheet {
    pub fn spreadsheet_create(rows: i16, cols: i16) -> Option<Box<Self>> {
        // eprintln!("Creating spreadsheet with {} rows and {} columns", rows, cols);
        let mut sheet = Box::new(Spreadsheet {
            rows,
            cols,
            view_row: 0,
            view_col: 0,
            cells: Vec::with_capacity(rows as usize * cols as usize),
            undo_stack: Vec::new(),
        });

        // Initialize the vector with None values
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

    pub fn letter_to_col(letters: &str) -> i16 {
        letters
            .chars()
            .fold(0, |acc, c| acc * 26 + (c as i16 - 'A' as i16 + 1))
    }

    pub fn get_cell_name(row: i16, col: i16) -> String {
        format!("{}{}", Self::col_to_letter(col), row)
    }

    pub fn spreadsheet_parse_cell_name(&self, cell_name: &str) -> Option<(i16, i16)> {
        let mut letters = String::new();
        let mut digits = String::new();
        let mut found_digit = false;

        for c in cell_name.chars() {
            if c.is_ascii_alphabetic() {
                // Once we've seen a digit, we shouldn't see more letters
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

    pub fn is_numeric(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
    }

    pub fn find_depends(&self, formula: &str) -> Result<(i16, i16, i16, i16, bool), &'static str> {
        let mut range_bool = false;
        let ranges = ["MIN", "MAX", "AVG", "SUM", "STDEV"];

        // Check if formula starts with a range function
        for range in &ranges {
            if formula.starts_with(range) {
                range_bool = true;

                // Find opening bracket
                if let Some(bracket_pos) = formula.find('(') {
                    let new_formula = &formula[bracket_pos + 1..];

                    // Extract range without closing bracket
                    if let Some(closing_bracket_pos) = new_formula.find(')') {
                        let only_range = &new_formula[..closing_bracket_pos];

                        // Parse the range in the format "A1:B2"
                        if let Some(colon_pos) = only_range.find(':') {
                            let start_cell = &only_range[..colon_pos];
                            let end_cell = &only_range[colon_pos + 1..];

                            // Parse start and end cells
                            if let Some((row1, col1)) = self.spreadsheet_parse_cell_name(start_cell)
                            {
                                if let Some((row2, col2)) =
                                    self.spreadsheet_parse_cell_name(end_cell)
                                {
                                    // Check if range is valid
                                    if col2 < col1 || (col1 == col2 && row2 < row1) {
                                        return Err("Invalid formula format");
                                    }

                                    // Convert to 0-based index (like in C version)
                                    let row1 = row1 - 1;
                                    let row2 = row2 - 1;

                                    return Ok((row1, row2, col1, col2, range_bool));
                                }
                            }
                        }
                    }
                }

                return Err("Invalid range format");
            }
        }

        // Not a range function, look for cell references
        let mut r1 = 0;
        let mut r2 = 0;
        let mut c1 = 0;
        let mut c2 = 0;

        // Use regex to find cell references like A1, B2, etc.
        let re = regex::Regex::new(r"([A-Za-z]+[0-9]+)").unwrap();

        for (count, cap) in re.captures_iter(formula).enumerate() {
            let ref_str = &cap[1];

            if count == 0 {
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(ref_str) {
                    r1 = row;
                    c1 = col;
                }
            } else if count == 1 {
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(ref_str) {
                    r2 = row;
                    c2 = col;
                }
            }
        }

        Ok((r1, r2, c1, c2, range_bool))
    }

    pub fn spreadsheet_evaluate_expression(
        &self,
        expr: &ParsedRHS,
        _row: i16,
        _col: i16,
    ) -> (i32, bool) {
        match expr {
            ParsedRHS::Function { name, args } => {
                let mut error = false;
                // Handle function evaluation here
                let (arg1, arg2) = args;
                // arg1 nd arg2 would be Operand cell type from there extract r1,c1 and r2,c2
                let (r1, c1) = match arg1 {
                    Operand::Cell(r, c) => (*r, *c),
                    Operand::Number(_) => (0, 0), // Placeholder
                };
                let (r2, c2) = match arg2 {
                    Operand::Cell(r, c) => (*r, *c),
                    Operand::Number(_) => (0, 0), // Placeholder
                };
                // Call the appropriate function based on the name
                // For now, just return a dummy value
                // let r1 = r1 + 1;
                // let r2 = r2 + 1;
                // println!("{},{}",r1,r2);
                // new comment

                let count = (r2 - r1 + 1) as usize * (c2 - c1 + 1) as usize;
                let mut values = Vec::with_capacity(count);

                for i in r1..=r2 {
                    for j in c1..=c2 {
                        let index = (i - 1) as usize * self.cols as usize + (j - 1) as usize;
                        if index < self.cells.len() {
                            if let Some(ref c) = self.cells[index] {
                                if c.error {
                                    // cell.error = true;
                                    error = true;
                                    return (0, error);
                                }
                                // println!("{},c.value", c.value);
                                values.push(c.value);
                            } else {
                                values.push(0);
                            }
                        }
                    }
                }

                match name {
                    FunctionName::Min => {
                        if values.is_empty() {
                            return (0, error);
                        }
                        // cell.error = false;
                        error = false;
                        return (*values.iter().min().unwrap_or(&0), error);
                    }
                    FunctionName::Max => {
                        if values.is_empty() {
                            return (0, error);
                        }
                        // cell.error = false;
                        error = false;
                        return (*values.iter().max().unwrap_or(&0), error);
                    }
                    FunctionName::Sum => {
                        // cell.error = false;
                        // println!("Sum: {:?}", values);
                        error = false;
                        return (values.iter().sum(), error);
                    }
                    FunctionName::Avg => {
                        if values.is_empty() {
                            return (0, error);
                        }
                        // cell.error = false;
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

                        // cell.error = false;
                        error = false;
                        return ((variance.sqrt().round()) as i32, error);
                    }
                    _ => {}
                }

                (0, error)
            }
            ParsedRHS::Sleep(op) => {
                // Handle sleep function here
                let val;
                let error = false;

                match op {
                    Operand::Number(n) => {
                        // Just a direct number: no error
                        val = *n;
                    }
                    Operand::Cell(r, c) => {
                        let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
                        if let Some(cell) = self.cells.get(index).and_then(|c| c.as_ref()) {
                            val = cell.value;
                            if cell.error {
                                return (val, true);
                            }
                        } else {
                            // Uninitialized cell → default to 0
                            val = 0;
                        }
                    }
                }

                // Sleep if val > 0
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

                // Combine error flags
                let mut has_error = lhs_err || rhs_err;

                // Compute result
                let result = match operator {
                    '+' => lhs_val + rhs_val,
                    '-' => lhs_val - rhs_val,
                    '*' => lhs_val * rhs_val,
                    '/' => {
                        if rhs_val == 0 {
                            has_error = true;
                            0 // or some default value
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

            ParsedRHS::SingleValue(num) => {
                // Handle single value expression here
                match num {
                    Operand::Cell(r, c) => {
                        let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
                        self.cells[index]
                            .as_ref()
                            .map_or((0, false), |cell| (cell.value, cell.error))
                    }
                    Operand::Number(x) => (*x, false),
                }
            }
            ParsedRHS::None => {
                // No expression to evaluate
                (0, false)
            }
        }
    }

    pub fn rec_find_cycle_using_stack<'a>(
        &'a self,
        (r1, r2): (i16, i16),
        (c1, c2): (i16, i16),
        range_bool: bool,
        visited: &mut BTreeSet<(i16, i16)>,
        stack: &mut Vec<&'a Cell>,
    ) -> bool {
        while let Some(my_node) = stack.pop() {
            // Generate cell name for the current node
            // let cell_name = Self::get_cell_name(my_node.row as i32, my_node.col as i32);

            // Check if we've already visited this cell
            if visited.contains(&(my_node.row, my_node.col)) {
                continue; // Skip cells we've already processed
            }

            // Mark as visited
            visited.insert((my_node.row, my_node.col));

            // Check if the cell is part of the target range
            let in_range = if range_bool {
                // For range functions (SUM, AVG, etc.)
                my_node.row >= r1 && my_node.row <= r2 && my_node.col >= c1 && my_node.col <= c2
            } else {
                // For direct cell references
                (my_node.row == r1 && my_node.col == c1) || (my_node.row == r2 && my_node.col == c2)
            };

            if in_range {
                // Cycle detected
                return true;
            } else {
                // Check all dependent cells using our helper method
                let dependent_names = self.get_dependent_names(my_node);
                // println!("Dependent names: {:?}", dependent_names);
                // println!("Visited: {:?}", visited);
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

        // No cycle found
        false
    }

    // Count the number of dependent cells
    pub fn count_dependent_cells(&self, cell: &Cell) -> usize {
        match &cell.dependents {
            crate::cell::Dependents::Vector(vec) => vec.len(),
            crate::cell::Dependents::Set(set) => set.len(),
            crate::cell::Dependents::None => 0,
        }
    }

    // Helper method for the rec_find_cycle_using_stack function - simplifies dependent collection
    pub fn get_dependent_names(&self, cell: &Cell) -> Vec<(i16, i16)> {
        match &cell.dependents {
            crate::cell::Dependents::Vector(vec) => vec.clone(),
            crate::cell::Dependents::Set(set) => set.iter().cloned().collect(),
            crate::cell::Dependents::None => Vec::new(),
        }
    }

    // Entry point for cycle detection - checks if a given cell could create a cycle
    pub fn first_step_find_cycle(
        &self,
        (r_,c_): (i16, i16),
        (r1, c1): (i16, i16),
        (r2, c2): (i16, i16),
        range_bool: bool,
    ) -> bool {
        // Parse the cell name to get row and column indices
        // let (r_, c_) = match self.spreadsheet_parse_cell_name(cell_name) {
        //     Some(coords) => coords,
        //     None => return false, // Invalid cell name
        // };

        // Get the index of the cell in the flattened vector
        let index = (r_ - 1) as usize * self.cols as usize + (c_ - 1) as usize;

        // Get the cell from the spreadsheet
        let start_node = match &self.cells[index] {
            Some(cell) => cell,
            None => return false, // Cell doesn't exist
        };

        // Create a visited set and stack for cycle detection
        let mut visited = BTreeSet::new();
        let mut stack = vec![&**start_node];

        // Call the recursive helper to find cycles
        self.rec_find_cycle_using_stack((r1, r2), (c1, c2), range_bool, &mut visited, &mut stack)
    }

    pub fn remove_old_dependents(&mut self, r: i16, c: i16) {
        // Get formula from the cell we're updating
        let formula = {
            let index = (r - 1) as usize * self.cols as usize + (c - 1) as usize;
            if let Some(curr_cell) = self.cells.get(index).and_then(|opt| opt.as_ref()) {
                curr_cell.formula.clone()
            } else {
                ParsedRHS::None
            }
        };

        match formula {
            ParsedRHS::Function { name, args } => {
                if !name.is_copy() {
                    let (arg1, arg2) = args;
                    // Get the range of cells we depend on
                    let (start_row, start_col) = match arg1 {
                        Operand::Cell(row, col) => (row, col),
                        Operand::Number(_) => (0, 0), // Placeholder
                    };
                    let (end_row, end_col) = match arg2 {
                        Operand::Cell(row, col) => (row, col),
                        Operand::Number(_) => (0, 0), // Placeholder
                    };

                    // Remove our cell as a dependent from each cell in the range
                    for dep_r in start_row..=end_row {
                        for dep_c in start_col..=end_col {
                            let dep_index =
                                (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;

                            if let Some(dep_cell) =
                                self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut())
                            {
                                // Pass r, c (our cell) to remove as a dependent
                                crate::cell::cell_dep_remove(dep_cell, r, c);
                            }
                        }
                    }
                }
            }
            // Other patterns with the same fix...
            ParsedRHS::Arithmetic {
                lhs,
                operator: _,
                rhs,
            } => {
                // Handle the left operand
                if let Operand::Cell(dep_r, dep_c) = lhs {
                    let dep_index =
                        (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;

                    if let Some(dep_cell) =
                        self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut())
                    {
                        crate::cell::cell_dep_remove(dep_cell, r, c);
                    }
                }
                // Handle the right operand
                if let Operand::Cell(dep_r, dep_c) = rhs {
                    let dep_index =
                        (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;

                    if let Some(dep_cell) =
                        self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut())
                    {
                        crate::cell::cell_dep_remove(dep_cell, r, c);
                    }
                }
            }
            // Handle single value case too
            ParsedRHS::SingleValue(Operand::Cell(dep_r, dep_c)) => {
                let dep_index = (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;

                if let Some(dep_cell) = self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut()) {
                    crate::cell::cell_dep_remove(dep_cell, r, c);
                }
            }
            _ => {}
        }
    }

    pub fn update_dependencies(
        &mut self,
        (r, c): (i16, i16),
        (start_row,start_col): (i16, i16),
        (end_row,end_col): (i16, i16),
        is_range: bool,
    ) -> i32 {
        // eprintln!("Entered update_dependencies for cell: {cell_name} with formula: {formula}");
        // First, parse the cell—this mirrors C obtaining the current cell coordinates.
        // let (r, c) = self.spreadsheet_parse_cell_name(cell_name).unwrap();

        // Remove old dependencies
        self.remove_old_dependents(r, c);
        // Add the new formula to the cell
        // if let Some(cell) = self.cells.get_mut(index).and_then(|opt| opt.as_mut()) {
        //     cell.formula = Some(formula.to_string());
        // }
        // Now, process the formula to update dependencies.

        if is_range {
            let r1 = start_row - 1;
            let r2 = end_row - 1;

            // Check validity as in the C code:
            if end_col < start_col || (start_col == end_col && r2 < r1) {
                return -1;
            }

            // Iterate over the range and update dependencies.
            for r_it in start_row..=end_row {
                for c_it in start_col..=end_col {
                    let dep_index = (r_it - 1) as usize * self.cols as usize + (c_it - 1) as usize;
                    if let Some(dep_cell) =
                        self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut())
                    {
                        crate::cell::cell_dep_insert(dep_cell, r, c);
                    }
                }
            }
        } else {
            if start_row > 0 {
                let dep_index =
                    (start_row - 1) as usize * self.cols as usize + (start_col - 1) as usize;
                if let Some(dep_cell) = self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut()) {
                    crate::cell::cell_dep_insert(dep_cell, r, c);
                }
            }
            if end_row > 0 {
                let dep_index =
                    (end_row - 1) as usize * self.cols as usize + (end_col - 1) as usize;
                if let Some(dep_cell) = self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut()) {
                    crate::cell::cell_dep_insert(dep_cell, r, c);
                }
            }
        }
        0
    }

    pub fn topo_sort(&self, starting: &Cell) -> Box<Vec<(i16, i16)>> {
        // Create an empty result vector (equivalent to Node_l *head = NULL)
        let mut sorted_nodes = Box::new(Vec::new());

        // Create a stack for DFS traversal (equivalent to Node *st_top)
        let mut stack = Box::new(Vec::new());
        stack.push(starting.clone());

        // Track visited cells using a BTreeSet (equivalent to OrderedSet *visited)
        let mut visited = Box::new(BTreeSet::new());

        // Working stack to simulate recursive DFS with explicit stack
        let mut work_stack = Box::new(Vec::new());
        work_stack.push(Box::new((starting.row, starting.col)));

        while let Some(current) = work_stack.pop() {
            let current = *current;
            // println!("Current cell: {:?}", current);
            // Generate cell name for the current node
            // let cell_name = Self::get_cell_name(current.row, current.col);

            // If we've already processed this cell, skip it
            if visited.contains(&current) {
                continue;
            }

            // Get dependents of current cell
            let index = (current.0 - 1) as usize * self.cols as usize + (current.1 - 1) as usize;
            if let Some(cell) = self.cells.get(index).and_then(|opt| opt.as_ref()) {
                let dependent_keys = self.get_dependent_names(cell);
                let mut all_dependents_visited = true;

                // Check if all dependents are visited
                for dep_key in &dependent_keys {
                    if !visited.contains(dep_key) {
                        // If we have an unvisited dependent, we need to process it first
                        // if let Some((r, c)) = self.spreadsheet_parse_cell_name(dep_key) {
                        let (r, c) = *dep_key;

                        // Push current cell back to work stack
                        work_stack.push(Box::new(current));
                        // Push dependent to work stack to process first
                        work_stack.push(Box::new((r, c)));
                        all_dependents_visited = false;
                        break;
                    }
                }

                // If all dependents are visited, we can add this cell to sorted result
                if all_dependents_visited {
                    visited.insert(current);
                    sorted_nodes.push(current);
                }
            }
        }

        // Reverse result to match original C implementation order
        sorted_nodes.reverse();
        sorted_nodes
    }

    pub fn spreadsheet_set_cell_value(
        &mut self,
        row: i16,
        col: i16,
        rhs: ParsedRHS,
        status_out: &mut String,
    ) {
        // if cell_name.is_empty() || formula.is_empty() {
        //     *status_out = "invalid args".to_string();
        //     return;
        // }

        // Parse cell name to get row and column
        // let (r_, c_) = match self.spreadsheet_parse_cell_name(cell_name) {
        //     Some(coords) => coords,
        //     None => {
        //         *status_out = "invalid args".to_string();
        //         return;
        //     }
        // };

        // Get the cell
        let index = (row - 1) as usize * self.cols as usize + (col - 1) as usize;

        // Firstly, check if the formula is copy

        if let ParsedRHS::Function {
            name: FunctionName::Copy,
            args: (Operand::Cell(start_row, start_col), Operand::Cell(end_row, end_col)),
        } = rhs
        {
            // Calculate offsets
            let dest_row = row;
            let dest_col = col;
            let row_offset = dest_row as isize - start_row as isize;
            let col_offset = dest_col as isize - start_col as isize;
            // firstly iterate through all src_cells nd put them in some vector
            // then iterate through all dest cells nd put value from vectors in dest cells
            // can't be done in single iteration ; src and dest cells can be overlapping
            let mut src_val: Vec<i32> = Vec::new();
            let mut src_err: Vec<bool> = Vec::new();
            for r in start_row..=end_row {
                for c in start_col..=end_col {
                    let src_index = ((r - 1) * self.cols + (c - 1)) as usize;
                    if let Some(src_cell) = self.cells.get(src_index).and_then(|opt| opt.as_ref()) {
                        src_val.push(src_cell.value);
                        src_err.push(src_cell.error);
                    }
                }
            }
            // now iterate through all dest cells and put value from src_val in dest cells
            let mut cnter = 0;
            for r in start_row..=end_row {
                for c in start_col..=end_col {
                    self.update_dependencies((r, c), (0, 0), (0, 0), false);
                    let dest_index = ((r as isize + row_offset - 1) * self.cols as isize
                        + (c as isize + col_offset - 1))
                        as usize;
                    println!(
                        "start row {} col {} : {:?} ",
                        r,
                        c,
                        self.cells[dest_index].as_ref().unwrap().dependents
                    );
                    if dest_index < self.cells.len() {
                        if let Some(dest_cell) =
                            self.cells.get_mut(dest_index).and_then(|opt| opt.as_mut())
                        {
                            self.undo_stack.push((
                                dest_cell.formula.clone(),
                                dest_cell.row,
                                dest_cell.col,
                            ));
                            dest_cell.value = src_val[cnter];
                            dest_cell.formula =
                                ParsedRHS::SingleValue(Operand::Number(dest_cell.value)); // Clear formula
                            dest_cell.error = src_err[cnter]; // copy error state
                            cnter += 1;
                        }
                    }
                }
            }
            *status_out = "ok".to_string();
            return;
        }

        // Go ahead if it's not COPY form
        // new
        // println!("starting find_depends");
        // // get the starting time
        // let start_time = Instant::now();
        // new
        // Find dependencies
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

        // let (r1, r2, c1, c2, range_bool) = match self.find_depends(formula) {
        //     Ok(deps) => deps,
        //     Err(_) => {
        //         *status_out = "invalid command".to_string();
        //         return;
        //     }
        // };
        // new
        // println!("find_depends took {:?}", start_time.elapsed().as_secs_f64());
        // new
        // Check for cycles
        if self.first_step_find_cycle((row, col), (r1, c1), (r2, c2), is_range) {
            *status_out = "Cycle Detected".to_string();
            return;
        }

        // new
        // println!(
        //     "cycle detection took {:?}",
        //     start_time.elapsed().as_secs_f64()
        // );
        // new

        // Update dependencies
        self.update_dependencies((row, col), (r1, c1), (r2, c2), is_range);

        // println!("cell dependency of A1 are : {:?}", self.cells[0].as_ref().unwrap().dependents);
        // println!("cell dependency of current cell are : {:?}", self.cells[index].as_ref().unwrap().dependents);

        let cell = match self.cells.get_mut(index).and_then(|opt| opt.as_mut()) {
            Some(cell) => cell,
            None => {
                *status_out = "invalid args".to_string();
                return;
            }
        };
        // new
        // println!(
        //     "update dependencies took {:?}",
        //     start_time.elapsed().as_secs_f64()
        // );
        // new
        // Update the cell's formula
        // println!(
        //     "pushing cell {}{} with formula: {:?}",
        //     cell.row, cell.col, cell.formula
        // );

        // UNDO PENDING
        self.undo_stack
            .push((cell.formula.clone(), cell.row, cell.col));

        // UNDO PENDING
        // println!("length of stack after push {}", self.undo_stack.len());
        cell.formula = rhs;
        // mutable reference not needed anymore
        // take cell as immutable
        let cell = match self.cells.get(index).and_then(|opt| opt.as_ref()) {
            Some(cell) => cell,
            None => {
                *status_out = "invalid args".to_string();
                return;
            }
        };
        // Perform topological sort

        let sorted_cells = self.topo_sort(cell);
        // println!("sorted cells {:?}", sorted_cells);
        // new
        // println!("topo sort took {:?}", start_time.elapsed().as_secs_f64());
        // new

        // println!("Updating cells in topological order:");
        // for (row, col) in &sorted_cells {
        //     println!("Cell {}{}...", Self::col_to_letter(*col as i32), row);
        // }

        // Evaluate expressions for all cells in topological order

        for (row, col) in sorted_cells.iter() {
            // Calculate index for the current cell in topological order
            let sorted_index = (*row - 1) as usize * self.cols as usize + (*col - 1) as usize;

            // Get formula from the sorted cell
            // let formula = match self.cells.get(sorted_index).and_then(|opt| opt.as_ref()) {
            //     Some(cell) => cell.formula.as_deref().unwrap_or(""),
            //     None => {
            //         continue; // Skip if cell doesn't exist
            //     }
            // };
            let formula = match self.cells.get(sorted_index).and_then(|opt| opt.as_ref()) {
                Some(cell) => &cell.formula,
                None => {
                    continue; // Skip if cell doesn't exist
                }
            };

            // Evaluate expression
            let (value, error_cell) = self.spreadsheet_evaluate_expression(formula, *row, *col);

            // Update the sorted cell's value
            if let Some(sorted_cell) = self
                .cells
                .get_mut(sorted_index)
                .and_then(|opt| opt.as_mut())
            {
                sorted_cell.value = value;
                sorted_cell.error = error_cell;
            }
        }
        // new
        // println!("evaluation took {:?}", start_time.elapsed().as_secs_f64());
        // new

        *status_out = "ok".to_string();
    }

    pub fn spreadsheet_undo(&mut self, status_out: &mut String) {
        println!("undo stack size {}", self.undo_stack.len());
        // iterate through undo_stack extract cell name --> update dependencies --> set value
        let mut undo_stack = self.undo_stack.clone();
        self.undo_stack.clear();
        for _ in 0..undo_stack.len() {
            let (formula_new, row, col) = undo_stack.pop().unwrap();

            self.spreadsheet_set_cell_value(row, col, formula_new, status_out);
        }
        // make the undo stack empty
    }

    // pub fn spreadsheet_redo(&mut self) {}

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

        // Print column headers
        print!("\t\t");
        for col in (self.view_col + 1)..=end_col {
            print!("{}\t\t", Self::col_to_letter(col));
        }
        println!();

        // Print rows
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

    pub fn is_valid_command(&self, cell_name: &str, formula: &str) -> (bool, i16, i16, ParsedRHS) {
        // initialise the return value
        let mut ret = (false, 0, 0, ParsedRHS::None);
        if cell_name.is_empty() || formula.is_empty() {
            return ret;
        }

        // Check if valid cell name
        // also update the ret val accordingly
        if let Some((row, col)) = self.spreadsheet_parse_cell_name(cell_name) {
            ret.1 = row;
            ret.2 = col;
        } else {
            return ret;
        }
        // if self.spreadsheet_parse_cell_name(cell_name).is_none() {
        //     return false;
        // }

        // Check if formula is empty (already checked above, redundant)
        if formula.is_empty() {
            return ret;
        }

        // Check for function call pattern: FUNC(...)
        if let Some(captures) = FUNC_REGEX.captures(formula) {
            let func = captures.get(1).unwrap().as_str();
            let args = captures.get(2).unwrap().as_str();

            if func.eq_ignore_ascii_case("SLEEP") {
                if args.is_empty() {
                    return ret;
                }
                // Check if args is a valid integer
                // also extract that integer
                if let Ok(value) = args.parse::<i32>() {
                    ret.0 = true;
                    ret.3 = ParsedRHS::Sleep(Operand::Number(value));
                    return ret;
                }
                // Check if args is a valid cell reference
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(args) {
                    ret.0 = true;
                    ret.3 = ParsedRHS::Sleep(Operand::Cell(row, col));
                    return ret;
                }
                return ret;
            } else {
                // Check for range functions like MIN, MAX, etc.
                if let Some(colon_pos) = args.find(':') {
                    let (start, end) = args.split_at(colon_pos);
                    let end = &end[1..]; // Skip the colon

                    if let (Some((start_row, start_col)), Some((end_row, end_col))) = (
                        self.spreadsheet_parse_cell_name(start.trim()),
                        self.spreadsheet_parse_cell_name(end.trim()),
                    ) {
                        if start_row <= end_row && start_col <= end_col {
                            // if matches!(
                            //     func.to_uppercase().as_str(),
                            //     "MIN" | "MAX" | "SUM" | "AVG" | "STDEV" | "CUT" | "COPY"
                            // ) {
                            //     ret.0 = true;
                            //     ret.3 = ParsedRHS::Function { name: FunctionName::, args: (Operand::Cell(start_row as usize,start_col as usize),Operand::Cell(end_row as usize, end_col as usize)) };
                            //     return ret;
                            // }
                            if let Some(fname) = FunctionName::from_strng(func) {
                                if func == "COPY" {
                                    let dest_row = ret.1;
                                    let dest_col = ret.2;
                                    // Calculate offsets
                                    let row_offset = dest_row - start_row;
                                    let col_offset = dest_col - start_col;

                                    // Calculate the final destination cell coordinates
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
                                                Operand::Cell(final_row, final_col),
                                            ),
                                        };
                                        return ret;
                                    } else {
                                        // Invalid destination cell
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

        // Check if the formula is a single cell reference (e.g., A1)
        if let Some((row, col)) = self.spreadsheet_parse_cell_name(formula) {
            ret.0 = true;
            ret.3 = ParsedRHS::SingleValue(Operand::Cell(row, col));
            return ret;
        }

        // Check for only some integer with optional sign on the rhs
        if let Ok(value) = formula.parse::<i32>() {
            ret.0 = true;
            ret.3 = ParsedRHS::SingleValue(Operand::Number(value));
            return ret;
        }
        // Check for arithmetic expressions with cell references or numbers
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
    pub fn is_valid_arithmetic_expression(&self, expr: &str) -> (bool, ParsedRHS) {
        // initialise the return value
        let mut ret = (false, ParsedRHS::None);
        // let mut oprnd1 = Operand::Number(0);
        // let mut oprnd2 = Operand::Number(0);
        // Use regex to separate expression into components

        if let Some(captures) = ARITH_EXPR_REGEX.captures(expr) {
            let first_operand = captures.get(1).unwrap().as_str();
            let operator = captures.get(4).unwrap().as_str();
            let second_operand = captures.get(5).unwrap().as_str();

            // Verify first operand
            let oprnd1 = if first_operand.chars().next().unwrap().is_ascii_alphabetic() {
                // It's a cell reference
                // self.spreadsheet_parse_cell_name(first_operand).is_some()
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(first_operand) {
                    Operand::Cell(row, col)
                } else {
                    // It's a number with optional sign

                    return (false, ParsedRHS::None);
                }
            } else if let Ok(value) = first_operand.parse::<i32>() {
                Operand::Number(value)
            } else {
                return (false, ParsedRHS::None);
            };

            // Verify second operand
            let oprnd2 = if second_operand.chars().next().unwrap().is_ascii_alphabetic() {
                // It's a cell reference
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(second_operand) {
                    Operand::Cell(row, col)
                } else {
                    // It's a number with optional sign

                    return (false, ParsedRHS::None);
                }
            } else if let Ok(value) = second_operand.parse::<i32>() {
                Operand::Number(value)
            } else {
                return (false, ParsedRHS::None);
            };
            // Check if the operator is valid
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
