// use cop::cell::{Cell, cell_create, cell_dep_insert, cell_dep_remove};
// // use crate::cell::{Cell, cell_create, cell_destroy, cell_dep_insert, cell_dep_remove};

// fn cell_contains(cell: &Cell, key: &str) -> bool {
//     cell.contains(key)
// }

// fn main() {
//     println!("=== Cell Test Suite ===\n");

//     println!("Test 1: Basic cell creation");
//     let mut cell = cell_create(1, 1);
//     assert_eq!(cell.row, 1);
//     assert_eq!(cell.col, 1);
//     assert_eq!(cell.value, 0);
//     assert!(!cell.error);
//     assert_eq!(cell.formula, None);
//     assert_eq!(cell.dependents_initialised, 0);
//     println!(
//         "Cell created at position ({},{}) - PASS\n",
//         cell.row, cell.col
//     );

//     println!("Test 2: Cell value modification");
//     cell.value = 100;
//     assert_eq!(cell.value, 100);
//     println!("Cell value set to {} - PASS\n", cell.value);

//     println!("Test 3: Cell formula assignment");
//     let formula = "=B1+C2".to_string();
//     cell.formula = Some(formula);
//     assert_eq!(cell.formula, Some("=B1+C2".to_string()));
//     println!(
//         "Cell formula set to \"{}\" - PASS\n",
//         cell.formula.as_ref().unwrap()
//     );

//     println!("Test 4: Error flag");
//     cell.error = true;
//     assert!(cell.error);
//     println!("Cell error flag set - PASS\n");

//     println!("Test 5: Managing dependents");
//     cell_dep_insert(&mut cell, "B1");
//     cell_dep_insert(&mut cell, "C2");
//     cell_dep_insert(&mut cell, "D3");

//     assert!(cell_contains(&cell, "B1"));
//     assert!(cell_contains(&cell, "C2"));
//     assert!(cell_contains(&cell, "D3"));
//     assert!(!cell_contains(&cell, "E4"));

//     println!("Test 6: Removing dependents");
//     cell_dep_remove(&mut cell, "C2");
//     assert!(!cell_contains(&cell, "C2"));

//     println!("Test 7: Creating multiple cells");
//     let mut cell2 = cell_create(2, 3);
//     assert_eq!(cell2.row, 2);
//     assert_eq!(cell2.col, 3);
//     println!("Cell 2 created at position ({},{})", cell2.row, cell2.col);

//     cell_dep_insert(&mut cell2, "A1");
//     cell_dep_insert(&mut cell2, "X10");

//     // Verify each cell has its own dependencies
//     assert!(cell_contains(&cell, "B1"));
//     assert!(!cell_contains(&cell2, "B1"));
//     assert!(!cell_contains(&cell, "A1"));
//     assert!(cell_contains(&cell2, "A1"));
//     println!("Each cell maintains its own dependencies - PASS\n");

//     println!("Test 8: Memory management");
//     let cell_size = std::mem::size_of::<Cell>();
//     println!("Size of Cell struct: {} bytes", cell_size);

//     println!("All tests completed.");
// }
fn main() {}
