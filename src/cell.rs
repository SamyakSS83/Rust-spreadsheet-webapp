use std::collections::BTreeSet; // Using BTreeSet as an AVL-tree-like ordered collection

pub struct Cell {
    pub row: i32,
    pub col: i32,
    pub value: i32,
    pub error: bool,
    pub formula: Option<String>,
    pub container: i32, // 0 for Vector, 1 for OrderedSet
    pub dependents_initialised: i32,
    dependents: Dependents,
}

enum Dependents {
    Vector(Vec<String>),
    Set(BTreeSet<String>),
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
            container: 0,
            dependents_initialised: 0,
            dependents: Dependents::None,
        }
    }

    pub fn dep_insert(&mut self, key: &str) {
        if self.dependents_initialised == 0 {
            self.dependents_initialised = 1;
            self.dependents = Dependents::Vector(Vec::new());
        }

        if self.container == 0 {
            // Using Vector
            if let Dependents::Vector(vec) = &mut self.dependents {
                if vec.len() > 7 {
                    // Convert to OrderedSet
                    let mut set = BTreeSet::new();
                    for item in vec.iter() {
                        set.insert(item.clone());
                    }
                    set.insert(key.to_string());
                    self.dependents = Dependents::Set(set);
                    self.container = 1;
                } else {
                    vec.push(key.to_string());
                }
            }
        } else {
            // Using OrderedSet
            if let Dependents::Set(set) = &mut self.dependents {
                set.insert(key.to_string());
            }
        }
    }

    pub fn dep_remove(&mut self, key: &str) {
        match &mut self.dependents {
            Dependents::Vector(vec) => {
                vec.retain(|k| k != key);
            },
            Dependents::Set(set) => {
                set.remove(key);
            },
            Dependents::None => {}
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        if self.dependents_initialised == 0 {
            return false;
        }

        match &self.dependents {
            Dependents::Vector(vec) => vec.iter().any(|k| k == key),
            Dependents::Set(set) => set.contains(key),
            Dependents::None => false,
        }
    }
}

// Public interface functions that match the C API
pub fn cell_create(row: i32, col: i32) -> Box<Cell> {
    Box::new(Cell::create(row, col))
}

pub fn cell_destroy(_cell: Box<Cell>) {
    // No explicit destruction needed in Rust due to RAII
}

pub fn cell_dep_insert(cell: &mut Cell, key: &str) {
    cell.dep_insert(key);
}

pub fn cell_dep_remove(cell: &mut Cell, key: &str) {
    cell.dep_remove(key);
}
