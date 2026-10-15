RUSTC = rustc

main: src/main.rs
	$(RUSTC) -o $@ src/main.rs
