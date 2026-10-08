# SDK maintenance tasks (run from the SDK root).

.PHONY: test-rust

# unit tests + the example mod
test-rust:
	cd rust && cargo test
	cd rust && cargo build --release --example low_hp --target wasm32-wasip2
