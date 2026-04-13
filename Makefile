install-local:
	cargo install --path . --root "$(HOME)/.local" --force

fmt:
	cargo fmt

test:
	cargo test
