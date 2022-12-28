dev:
	cargo watch -x run
dev-detailed:
	RUST_BACKTRACE=1 cargo watch -x run