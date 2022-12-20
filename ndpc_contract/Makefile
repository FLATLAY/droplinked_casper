prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cd contract && cargo build --release --target wasm32-unknown-unknown
	wasm-strip contract/target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true
	cp contract/target/wasm32-unknown-unknown/release/contract.wasm deploy/contract.wasm
build-session:
	cd session && cargo build --release --target wasm32-unknown-unknown
	wasm-strip session/target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true
	cp session/target/wasm32-unknown-unknown/release/contract.wasm deploy/session.wasm

build-all: build-contract build-session

test: build-contract
	mkdir -p tests/wasm
	cp contract/target/wasm32-unknown-unknown/release/contract.wasm tests/wasm

clippy:
	cd contract && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contract && cargo fmt -- --check

lint: clippy
	cd contract && cargo fmt

clean:
	cd contract && cargo clean
	rm -rf tests/wasm
