make type of rows and cols consistent, somewhere it's usize and somewhere it's u32 or i32 .
we don't need to pass expr in spreadsheet_evaluate_expression. it can be removed
in spreadsheet_test.rs in many places i have typecasted &u32 to i32, in get_cell_name and col_to_letter, ideally this is not safe


how to run:

```bash
cargo build
./target/debug/cop
```

To Do:

write main for autograder
write is_valid_command
write spreadsheet_display
reduce memory
reduce time