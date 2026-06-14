# Build release binary
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Run integration tests only
test-int:
    cargo test -p drawz-core --test edge_cases --test happy_path

# Run happy-path tests with visible diagram output
test-print:
    cargo test -p drawz-core --test happy_path -- --nocapture

# Run clippy with pedantic lints
lint:
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

# Install drawz binary to ~/.cargo/bin
install:
    cargo install --path crates/drawz-cli
