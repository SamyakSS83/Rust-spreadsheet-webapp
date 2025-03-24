use std::collections::BTreeSet;
use crate::cell::{cell_create, Cell};

pub struct Spreadsheet {
    pub rows: i32,
    pub cols: i32,
    pub view_row: i32,
    pub view_col: i32,
    pub cells: Vec<Option<Box<Cell>>>,
}

impl Spreadsheet {
    pub fn spreadsheet_create(rows: i32, cols: i32) -> Option<Box<Self>> {
        let mut sheet = Box::new(Spreadsheet {
            rows,
            cols,
            view_row: 0,
            view_col: 0,
            cells: Vec::with_capacity((rows * cols) as usize),
        });

        // Initialize the vector with None values
        for _ in 0..(rows * cols) {
            sheet.cells.push(None);
        }

        for r in 1..=rows {
            for c in 1..=cols {
                let index = ((r - 1) * cols + (c - 1)) as usize;
                sheet.cells[index] = Some(cell_create(r, c));
            }
        }

        Some(sheet)
    }

    

    pub fn col_to_letter(col: i32) -> String {
        let mut col = col;
        let mut result = String::new();
        while col > 0 {
            col -= 1;
            result.push(((col % 26) as u8 + b'A') as char);
            col /= 26;
        }
        result.chars().rev().collect()
    }

    pub fn letter_to_col(letters: &str) -> i32 {
        letters.chars().fold(0, |acc, c| acc * 26 + (c as i32 - 'A' as i32 + 1))
    }

    pub fn get_cell_name(row: i32, col: i32) -> String {
        format!("{}{}", Self::col_to_letter(col), row)
    }

    pub fn spreadsheet_parse_cell_name(&self, cell_name: &str) -> Option<(i32, i32)> {
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
        let row = digits.parse::<i32>().ok()?;

        if col > self.cols || row > self.rows || row <= 0 {
            return None;
        }
        Some((row, col))
    }

    pub fn isNumeric(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit())
    }

    pub fn find_depends(&self, formula: &str) -> Result<(i32, i32, i32, i32, bool), &'static str> {
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
                            if let Some((row1, col1)) = self.spreadsheet_parse_cell_name(start_cell) {
                                if let Some((row2, col2)) = self.spreadsheet_parse_cell_name(end_cell) {
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
        let mut r1 = -1;
        let mut r2 = -1;
        let mut c1 = -1;
        let mut c2 = -1;
        
        // Use regex to find cell references like A1, B2, etc.
        let re = regex::Regex::new(r"([A-Za-z]+[0-9]+)").unwrap();
        let mut count = 0;
        
        for cap in re.captures_iter(formula) {
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
            
            count += 1;
        }
        
        Ok((r1, r2, c1, c2, range_bool))
    }
    
    pub fn spreadsheet_evaluate_function(&self, func: &str, args: &str, cell: &mut Cell, expr: &str) -> i32 {
        // SLEEP function
        if func.eq_ignore_ascii_case("SLEEP") {
            let mut val = 0;
            
            // Case 1: args is a numeric value
            if Self::isNumeric(args) {
                val = args.parse::<i32>().unwrap_or(0);
                cell.error = false;
            } else {
                // Case 2: args is a cell reference
                let mut sign = 1;
                let args_str = if args.starts_with('-') {
                    sign = -1;
                    &args[1..]
                } else if args.starts_with('+') {
                    &args[1..]
                } else {
                    args
                };
                
                if let Some((row, col)) = self.spreadsheet_parse_cell_name(args_str) {
                    let index = ((row - 1) * self.cols + (col - 1)) as usize;
                    if index < self.cells.len() {
                        if let Some(ref op_cell) = self.cells[index] {
                            val = sign * op_cell.value;
                            if op_cell.error {
                                cell.error = true;
                                return val;
                            } else {
                                cell.error = false;
                            }
                        } else {
                            // Cell not found or not initialized, use 0
                            val = 0;
                        }
                    }
                }
            }
            
            // Sleep if value is positive
            if val > 0 {
                std::thread::sleep(std::time::Duration::from_secs(val as u64));
            }
            
            return val;
        }
        
        // Evaluate range functions
        let count: i32;
        let result = self.find_depends(expr);
        
        if let Ok((r1, r2, c1, c2, _range_bool)) = result {
            // Convert from 0-based to 1-based index for consistency with C code
            let r1 = r1 + 1;
            let r2 = r2 + 1;
            
            count = (r2 - r1 + 1) * (c2 - c1 + 1);
            let mut values = Vec::with_capacity(count as usize);
            
            for i in r1..=r2 {
                for j in c1..=c2 {
                    let index = ((i - 1) * self.cols + (j - 1)) as usize;
                    if index < self.cells.len() {
                        if let Some(ref c) = self.cells[index] {
                            if c.error {
                                cell.error = true;
                                return 0;
                            }
                            values.push(c.value);
                        } else {
                            values.push(0);
                        }
                    }
                }
            }
            
            if func.eq_ignore_ascii_case("MIN") {
                if values.is_empty() {
                    return 0;
                }
                cell.error = false;
                return *values.iter().min().unwrap_or(&0);
            } else if func.eq_ignore_ascii_case("MAX") {
                if values.is_empty() {
                    return 0;
                }
                cell.error = false;
                return *values.iter().max().unwrap_or(&0);
            } else if func.eq_ignore_ascii_case("SUM") {
                cell.error = false;
                return values.iter().sum();
            } else if func.eq_ignore_ascii_case("AVG") {
                if values.is_empty() {
                    return 0;
                }
                cell.error = false;
                let sum: i32 = values.iter().sum();
                return sum / values.len() as i32;
            } else if func.eq_ignore_ascii_case("STDEV") {
                if values.len() < 2 {
                    return 0;
                }
                
                let mean = values.iter().sum::<i32>() as f64 / values.len() as f64;
                let variance = values.iter()
                    .map(|&x| {
                        let diff = x as f64 - mean;
                        diff * diff
                    })
                    .sum::<f64>() / values.len() as f64;
                
                cell.error = false;
                return (variance.sqrt().round()) as i32;
            }
        }
        
        // Unknown function
        0
    }

    pub fn spreadsheet_evaluate_expression(&self, expr: &str, cell: &mut Cell) -> i32 {
        if expr.is_empty() {
            return 0;
        }

        // Check for function call pattern: FUNC(...)
        let func_regex = regex::Regex::new(r"^([A-Za-z]+)\((.*)\)$").unwrap();
        if let Some(captures) = func_regex.captures(expr) {
            let func = captures.get(1).unwrap().as_str();
            let args = captures.get(2).unwrap().as_str();
            return self.spreadsheet_evaluate_function(func, args, cell, expr);
        }

        // Check if the expr is a cell reference (e.g. A1)
        let cell_regex = regex::Regex::new(r"^[A-Za-z]+[0-9]+$").unwrap();
        if cell_regex.is_match(expr) {
            // It's a cell reference
            if let Some((row, col)) = self.spreadsheet_parse_cell_name(expr) {
                let index = ((row - 1) * self.cols + (col - 1)) as usize;
                if index < self.cells.len() {
                    if let Some(ref c) = self.cells[index] {
                        cell.error = c.error;
                        return c.value;
                    }
                }
            }
            return 0;
        }

        // Check for arithmetic operations
        // Parse the first number or cell reference
        let mut i = 0;
        let expr_chars: Vec<char> = expr.chars().collect();
        let len = expr_chars.len();
        
        let mut num1 = 0;
        let mut sign1 = 1;
        
        // Handle sign for first operand
        if i < len && expr_chars[i] == '+' {
            i += 1;
        } else if i < len && expr_chars[i] == '-' {
            sign1 = -1;
            i += 1;
        }
        
        // Parse the first operand (either a number or cell reference)
        if i < len && expr_chars[i].is_ascii_digit() {
            // It's a number
            let mut j = i;
            while j < len && expr_chars[j].is_ascii_digit() {
                num1 = num1 * 10 + (expr_chars[j] as u8 - b'0') as i32;
                j += 1;
            }
            i = j;
        } else if i < len && expr_chars[i].is_ascii_alphabetic() {
            // It's a cell reference
            let mut j = i;
            while j < len && expr_chars[j].is_ascii_alphabetic() {
                j += 1;
            }
            
            let mut k = j;
            while k < len && expr_chars[k].is_ascii_digit() {
                k += 1;
            }
            
            let cell_name = &expr[i..k];
            i = k;
            
            if let Some((row, col)) = self.spreadsheet_parse_cell_name(cell_name) {
                let index = ((row - 1) * self.cols + (col - 1)) as usize;
                if index < self.cells.len() {
                    if let Some(ref c) = self.cells[index] {
                        if c.error {
                            cell.error = true;
                            return 0;
                        }
                        num1 = c.value;
                    }
                }
            }
        }
        
        num1 *= sign1;
        
        // If no more characters, just return the first number
        if i >= len {
            cell.error = false;
            return num1;
        }
        
        // Get the operation
        let operation = expr_chars[i];
        i += 1;
        
        // Parse the second operand
        let mut num2 = 0;
        let mut sign2 = 1;
        
        // Handle sign for second operand
        if i < len && expr_chars[i] == '+' {
            i += 1;
        } else if i < len && expr_chars[i] == '-' {
            sign2 = -1;
            i += 1;
        }
        
        // Parse the second operand (either a number or cell reference)
        if i < len && expr_chars[i].is_ascii_digit() {
            // It's a number
            let mut j = i;
            while j < len && expr_chars[j].is_ascii_digit() {
                num2 = num2 * 10 + (expr_chars[j] as u8 - b'0') as i32;
                j += 1;
            }
        } else if i < len && expr_chars[i].is_ascii_alphabetic() {
            // It's a cell reference
            let mut j = i;
            while j < len && expr_chars[j].is_ascii_alphabetic() {
                j += 1;
            }
            
            let mut k = j;
            while k < len && expr_chars[k].is_ascii_digit() {
                k += 1;
            }
            
            let cell_name = &expr[i..k];
            
            if let Some((row, col)) = self.spreadsheet_parse_cell_name(cell_name) {
                let index = ((row - 1) * self.cols + (col - 1)) as usize;
                if index < self.cells.len() {
                    if let Some(ref c) = self.cells[index] {
                        if c.error {
                            cell.error = true;
                            return 0;
                        }
                        num2 = c.value;
                    }
                }
            }
        }
        
        num2 *= sign2;
        
        // Perform the operation
        cell.error = false;
        match operation {
            '+' => num1 + num2,
            '-' => num1 - num2,
            '*' => num1 * num2,
            '/' => {
                if num2 == 0 {
                    cell.error = true;
                    0
                } else {
                    num1 / num2
                }
            },
            _ => {
                cell.error = true;
                -1
            }
        }
    }

    pub fn rec_find_cycle_using_stack<'a>(&'a self, r1: i32, r2: i32, c1: i32, c2: i32, 
                                       range_bool: bool, visited: &mut BTreeSet<String>, 
                                       stack: &mut Vec<&'a Box<Cell>>) -> bool {
        while !stack.is_empty() {
            let my_node = stack.pop().unwrap();
            
            // Generate cell name for the current node
            let cell_name = Self::get_cell_name(my_node.row, my_node.col);
            
            // Check if we've already visited this cell
            if visited.contains(&cell_name) {
                continue;  // Skip cells we've already processed
            }
            
            // Mark as visited
            visited.insert(cell_name);
            
            // Check if the cell is part of the target range
            let in_range = if range_bool {
                // For range functions (SUM, AVG, etc.)
                my_node.row >= r1 && my_node.row <= r2 && 
                my_node.col >= c1 && my_node.col <= c2
            } else {
                // For direct cell references
                (my_node.row == r1 && my_node.col == c1) || 
                (my_node.row == r2 && my_node.col == c2)
            };
            
            if in_range {
                // Cycle detected
                return true;
            } else {
                // Check all dependent cells using our helper method
                let dependent_names = self.get_dependent_names(my_node);
                
                for dependent_name in dependent_names {
                    if !visited.contains(dependent_name) {
                        if let Some((r, c)) = self.spreadsheet_parse_cell_name(dependent_name) {
                            let index = ((r - 1) * self.cols + (c - 1)) as usize;
                            if index < self.cells.len() {
                                if let Some(ref neighbor_node) = self.cells[index] {
                                    stack.push(neighbor_node);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // No cycle found
        false
    }

    // Helper method to check for cycles before updating a cell
    pub fn check_for_cycle<'a>(&'a self, r1: i32, r2: i32, c1: i32, c2: i32, 
                           range_bool: bool, curr_cell: &'a Box<Cell>) -> bool {
        let mut visited = BTreeSet::new();
        let mut stack = Vec::new();
        
        // Start DFS from the current cell
        stack.push(curr_cell);
        
        self.rec_find_cycle_using_stack(r1, r2, c1, c2, range_bool, &mut visited, &mut stack)
    }

    // Collects keys from dependents regardless of the storage type
    pub fn collect_dependent_keys(&self, cell: &Box<Cell>) -> Vec<String> {
        match &cell.dependents {
            crate::cell::Dependents::Vector(vec) => {
                // For Vector, simply clone all elements
                vec.iter().cloned().collect()
            },
            crate::cell::Dependents::Set(set) => {
                // For BTreeSet, also simply clone all elements (already ordered)
                set.iter().cloned().collect()
            },
            crate::cell::Dependents::None => {
                // No dependents
                Vec::new()
            }
        }
    }
    
    // Count the number of dependent cells
    pub fn count_dependent_cells(&self, cell: &Box<Cell>) -> usize {
        match &cell.dependents {
            crate::cell::Dependents::Vector(vec) => vec.len(),
            crate::cell::Dependents::Set(set) => set.len(),
            crate::cell::Dependents::None => 0,
        }
    }
    
    // Helper method for the rec_find_cycle_using_stack function - simplifies dependent collection
    pub fn get_dependent_names<'a>(&self, cell: &'a Box<Cell>) -> Vec<&'a String> {
        match &cell.dependents {
            crate::cell::Dependents::Vector(vec) => vec.iter().collect(),
            crate::cell::Dependents::Set(set) => set.iter().collect(),
            crate::cell::Dependents::None => Vec::new(),
        }
    }

    // Entry point for cycle detection - checks if a given cell could create a cycle
    pub fn first_step_find_cycle(&self, cell_name: &str, r1: i32, r2: i32, c1: i32, c2: i32, range_bool: bool) -> bool {
        // Parse the cell name to get row and column indices
        let (r_, c_) = match self.spreadsheet_parse_cell_name(cell_name) {
            Some(coords) => coords,
            None => return false, // Invalid cell name
        };
        
        // Get the index of the cell in the flattened vector
        let index = ((r_ - 1) * self.cols + (c_ - 1)) as usize;
        
        // Get the cell from the spreadsheet
        let start_node = match &self.cells[index] {
            Some(cell) => cell,
            None => return false, // Cell doesn't exist
        };
        
        // Create a visited set and stack for cycle detection
        let mut visited = BTreeSet::new();
        let mut stack = Vec::new();
        
        // Start DFS from the current cell
        stack.push(start_node);
        
        // Call the recursive helper to find cycles
        self.rec_find_cycle_using_stack(r1, r2, c1, c2, range_bool, &mut visited, &mut stack)
    }

    pub fn remove_old_dependents(&mut self, cell_name: &str) {
        // Parse the provided cell name to locate the current cell.
        let formula_and_deps = if let Some((r, c)) = self.spreadsheet_parse_cell_name(cell_name) {
            let index = ((r - 1) * self.cols + (c - 1)) as usize;
            if let Some(curr_cell) = self.cells.get(index).and_then(|opt| opt.as_ref()) {
                // If there's no formula, nothing to update.
                curr_cell.formula.clone()
            } else {
                None
            }
        } else {
            None
        };
        
        // Process formula if it exists
        if let Some(formula) = formula_and_deps {
            let ranges = ["MIN", "MAX", "AVG", "SUM", "STDEV"];
            let mut processed_range = false;
            // Check if the formula starts with one of the range functions.
            for range in &ranges {
                if formula.starts_with(range) {
                    if let (Some(open_paren_idx), Some(close_paren_idx)) =
                        (formula.find('('), formula.find(')'))
                    {
                        let only_range = &formula[open_paren_idx + 1..close_paren_idx];
                        
                        if let Some(colon_pos) = only_range.find(':') {
                            let start_cell_str = only_range[..colon_pos].trim();
                            let end_cell_str = only_range[colon_pos + 1..].trim();
                            
                            if let (Some((start_row, start_col)),
                                    Some((end_row, end_col))) =
                                (
                                    self.spreadsheet_parse_cell_name(start_cell_str),
                                    self.spreadsheet_parse_cell_name(end_cell_str)
                                )
                            {
                                // Convert to 0-based indices and iterate over the range.
                                for r in (start_row - 1)..=(end_row - 1) {
                                    for c in start_col..=end_col {
                                        let dep_index = (r * self.cols + (c - 1)) as usize;
                                        
                                        if let Some(dep_cell) =
                                            self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut())
                                        {
                                            crate::cell::cell_dep_remove(dep_cell, cell_name);
                                        }
                                    }
                                }
                                processed_range = true;
                            }
                        }
                    }
                    break;
                }
            }
            
            // Non-range formula: remove dependency from the two cell references found.
            if !processed_range {
                if let Ok((dep_r1, dep_r2, dep_c1, dep_c2, _)) = self.find_depends(&formula) {
                    if dep_r1 > 0 {
                        let dep_index = (((dep_r1 - 1) * self.cols + (dep_c1 - 1)) as usize);
                        
                        if let Some(dep_cell) = self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut()) {
                            crate::cell::cell_dep_remove(dep_cell, cell_name);
                        }
                    }
                    
                    if dep_r2 > 0 {
                        let dep_index = (((dep_r2 - 1) * self.cols + (dep_c2 - 1)) as usize);
                        
                        if let Some(dep_cell) = self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut()) {
                            crate::cell::cell_dep_remove(dep_cell, cell_name);
                        }
                    }
                }
            }
        }
    }

    pub fn update_dependencies(&mut self, cell_name: &str, formula: &str) -> i32 {
        // First, parse the cell—this mirrors C obtaining the current cell coordinates.
        let (r, c) = self.spreadsheet_parse_cell_name(cell_name).unwrap();

        // Remove old dependencies
        self.remove_old_dependents(cell_name);
        // Add the new formula to the cell
        let index = ((r - 1) * self.cols + (c - 1)) as usize;
        if let Some(cell) = self.cells.get_mut(index).and_then(|opt| opt.as_mut()) {
            cell.formula = Some(formula.to_string());
        }
        // Now, process the formula to update dependencies.

        let ranges = ["MIN", "MAX", "AVG", "SUM", "STDEV"];
        let mut processed_range = false;
        // If the formula starts with one of the range functions...
        for range in &ranges {
            if formula.starts_with(range) {
                if let Some(open_paren_idx) = formula.find('(') {
                    let rest = &formula[open_paren_idx + 1..];
                    if let Some(close_paren_idx) = rest.find(')') {
                        let only_range = &rest[..close_paren_idx];
                        // Expect a format like "A1:B3"
                        let parts: Vec<&str> = only_range.split(':').collect();
                        if parts.len() == 2 {
                            if let (Some((start_row, start_col)), Some((end_row, end_col))) =
                                (self.spreadsheet_parse_cell_name(parts[0].trim()),
                                 self.spreadsheet_parse_cell_name(parts[1].trim()))
                            {
                                // Convert to 0-based indices for rows (columns remain 1-based in our indexing)
                                let r1 = start_row - 1;
                                let r2 = end_row - 1;
                                
                                // Check validity as in the C code:
                                if end_col < start_col || (start_col == end_col && r2 < r1) {
                                    return -1;
                                }

                                // Iterate over the range and update dependencies.
                                for r in r1..=r2 {
                                    for c in start_col..=end_col {
                                        let dep_index = (r * self.cols + (c - 1)) as usize;
                                        if let Some(dep_cell) = self.cells
                                            .get_mut(dep_index)
                                            .and_then(|opt| opt.as_mut())
                                        {
                                            crate::cell::cell_dep_insert(dep_cell, cell_name);
                                        }
                                    }
                                }
                                processed_range = true;
                            }
                        }
                    }
                }
                break;
            }
        }
        // Non-range formula: add dependency only for the two cell references found.
        if !processed_range {
            if let Ok((dep_r1, dep_r2, dep_c1, dep_c2, _)) = self.find_depends(formula) {
                if dep_r1 > 0 {
                    let dep_index = (((dep_r1 - 1) * self.cols + (dep_c1 - 1)) as usize);
                    if let Some(dep_cell) = self.cells
                        .get_mut(dep_index)
                        .and_then(|opt| opt.as_mut())
                    {
                        crate::cell::cell_dep_insert(dep_cell, cell_name);
                    }
                }
                if dep_r2 > 0 {
                    let dep_index = (((dep_r2 - 1) * self.cols + (dep_c2 - 1)) as usize);
                    if let Some(dep_cell) = self.cells
                        .get_mut(dep_index)
                        .and_then(|opt| opt.as_mut())
                    {
                        crate::cell::cell_dep_insert(dep_cell, cell_name);
                    }
                }
            }
        }
        0
    }

}


