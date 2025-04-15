.PHONY: all clean

all:
	cargo build --release
	mv ./target/release/cop ./target/release/spreadsheet

clean:
	cargo clean