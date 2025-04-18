.PHONY: all clean website

all:
	cargo build --bin spreadsheet

website:
	cargo run --release --bin website --features web

clean:
	cargo clean