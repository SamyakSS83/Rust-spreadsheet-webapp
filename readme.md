make type of rows and cols consistent, somewhere it's usize and somewhere it's u32 or i32 .
we don't need to pass expr in spreadsheet_evaluate_expression. it can be removed
in spreadsheet_test.rs in many places i have typecasted &u32 to i32, in get_cell_name and col_to_letter, ideally this is not safe


how to run:

```bash
cargo build
./target/debug/spreadsheet
```

```bash
cargo install flamegraph

sudo sh -c 'echo 0 > /proc/sys/kernel/perf_event_paranoid'

cargo flamegraph --bin spreadsheet -- 999 18278 < ../cop290_autograder/hidden_tc2/chain/large_dep_chain.cmds

```

To Do:

write main for autograder
write is_valid_command
write spreadsheet_display
reduce memory
reduce time