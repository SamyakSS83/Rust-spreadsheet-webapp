.PHONY: all clean website

all:
	RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release

website:
	cargo build --release --bin website --features web

clean:
	cargo clean