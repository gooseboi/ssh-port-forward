build_static $RUSTFLAGS='-C target-feature=+crt-static':
	cargo build --release
