use serde::{Deserialize, Serialize};
use std::collections::BTreeSet; // Using BTreeSet as an AVL-tree-like ordered collection

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Cell {
    pub row: i32,
    pub col: i32,
    pub error: bool,
    pub value: i32,
    pub formula: Option<String>,
    pub dependents_initialised: i32,
    pub dependents: Dependents,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Dependents {
    Vector(Vec<(i32, i32)>),
    Set(BTreeSet<(i32, i32)>),
    None,
}

impl Cell {
    pub fn create(row: i32, col: i32) -> Self {
        Cell {
            row,
            col,
            value: 0,
            error: false,
            formula: None,
            dependents_initialised: 0,
            dependents: Dependents::None,
        }
    }

    pub fn dep_insert(&mut self, row: i32, col: i32) {
        // Set the initialised flag to 1 whenever a dependency is added
        self.dependents_initialised = 1;
        let key = (row, col);

        match &mut self.dependents {
            Dependents::None => {
                let mut v = Vec::new();
                v.push(key);
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

    pub fn dep_remove(&mut self, row: i32, col: i32) {
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

    pub fn contains(&self, row: i32, col: i32) -> bool {
        if self.dependents_initialised == 0 {
            return false;
        }

        let key = (row, col);
        match &self.dependents {
            Dependents::Vector(vec) => vec.iter().any(|k| k == &key),
            Dependents::Set(set) => set.contains(&key),
            Dependents::None => false,
        }
    }
}

// Public interface functions that match the C API
pub fn cell_create(row: i32, col: i32) -> Box<Cell> {
    Box::new(Cell::create(row, col))
}

pub fn cell_dep_insert(cell: &mut Cell, row: i32, col: i32) {
    cell.dep_insert(row, col);
}

pub fn cell_dep_remove(cell: &mut Cell, row: i32, col: i32) {
    cell.dep_remove(row, col);
}

pub fn cell_contains(cell: &Cell, row: i32, col: i32) -> bool {
    cell.contains(row, col)
}
