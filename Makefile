.PHONY: all clean website

all:
	RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release --bin spreadsheet

website:
	cargo run --release --bin website --features web

clean:
	cargo clean