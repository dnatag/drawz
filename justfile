# Build release binary
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Run integration tests only
test-int:
    cargo test -p drawz-core --test freeform --test table --test tree --test flow --test state --test sequence --test dag --test mermaid --test component

# Run integration tests with visible diagram output
test-print:
    cargo test -p drawz-core --test freeform --test table --test tree --test flow --test state --test sequence --test dag --test mermaid --test component -- --nocapture

# Run clippy with pedantic lints
lint:
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

# Install drawz binary to ~/.cargo/bin
install:
    cargo install --path crates/drawz-cli
