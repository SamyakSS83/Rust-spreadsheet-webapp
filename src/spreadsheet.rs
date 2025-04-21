use crate::cell::{Cell, cell_create};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub enum ParsedRHS {
    Function {
        name: FunctionName,
        args: (Operand, Operand),
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

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Operand {
    Number(i32),
    Cell(i16, i16),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
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

    pub fn get_dependent_names(&self, cell: &Cell) -> Vec<(i16, i16)> {
        match &cell.dependents {
            crate::cell::Dependents::Vector(vec) => vec.clone(),
            crate::cell::Dependents::Set(set) => set.iter().cloned().collect(),
            crate::cell::Dependents::None => Vec::new(),
        }
    }

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
                    if let Some(dep_cell) = self
                        .cells
                        .get_mut((dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize)
                        .and_then(|opt| opt.as_mut())
                    {
                        crate::cell::cell_dep_remove(dep_cell, r, c);
                    }
                }
                if let Operand::Cell(dep_r, dep_c) = rhs {
                    if let Some(dep_cell) = self
                        .cells
                        .get_mut((dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize)
                        .and_then(|opt| opt.as_mut())
                    {
                        crate::cell::cell_dep_remove(dep_cell, r, c);
                    }
                }
            }
            ParsedRHS::SingleValue(Operand::Cell(dep_r, dep_c)) => {
                let dep_index = (dep_r - 1) as usize * self.cols as usize + (dep_c - 1) as usize;

                if let Some(dep_cell) = self.cells.get_mut(dep_index).and_then(|opt| opt.as_mut()) {
                    crate::cell::cell_dep_remove(dep_cell, r, c);
                }
            }
            ParsedRHS::Sleep(Operand::Cell(dep_r, dep_c)) => {
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
        (start_row, start_col): (i16, i16),
        (end_row, end_col): (i16, i16),
        is_range: bool,
    ) -> i32 {
        self.remove_old_dependents(r, c);

        if is_range {
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

    pub fn spreadsheet_undo(&mut self, status_out: &mut String) {
        let mut undo_stack = self.undo_stack.clone();
        self.undo_stack.clear();
        for _ in 0..undo_stack.len() {
            let (formula_new, row, col) = undo_stack.pop().unwrap();

            self.spreadsheet_set_cell_value(row, col, formula_new, status_out);
        }
    }

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
                        self.spreadsheet_parse_cell_name(start.trim()),
                        self.spreadsheet_parse_cell_name(end.trim()),
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
