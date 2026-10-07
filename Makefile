# SDK maintenance tasks (run from the SDK root).

.PHONY: gen-p1 gen-cs gen-abi test-rust

# wit/cuo-mod.wit -> rust/src/p1/cuo.rs, rust/src/sigcheck.rs
gen-p1:
	cargo run --offline --manifest-path tools/wit-p1-gen/Cargo.toml -- wit rust/src

# wit/cuo-mod.wit -> dotnet/Cuo.g.cs (the C# SDK's host / assets / actions / packets)
gen-cs:
	cargo run --offline --manifest-path tools/wit-p1-gen/Cargo.toml --bin wit-cs-gen -- wit dotnet

# abi/mod-abi.fbs -> rust/src/p1/generated.rs (planus-cli 1.3.0); paths made relative
gen-abi:
	cd rust/src/p1 && planus rust -o generated.rs ../../../abi/mod-abi.fbs
	sed -i -E 's#[^ `]*mod-abi\.fbs#mod-abi.fbs#g' rust/src/p1/generated.rs

# unit tests + the example mod on both targets
test-rust:
	cd rust && cargo test
	cd rust && cargo build --release --example low_hp --target wasm32-wasip1
	cd rust && cargo build --release --example low_hp --target wasm32-wasip2 --no-default-features --features p2
