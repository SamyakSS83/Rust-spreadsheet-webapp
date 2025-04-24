.PHONY: all clean ext1

all:
	RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release

ext1:
	cargo build --release --bin website --features web
	./target/release/website

test:
	cargo test

coverage:
	cargo-tarpaulin 

docs: 
	cargo doc
	@echo "Documentation generated in target/doc/cop/index.html"
clean:
	cargo clean