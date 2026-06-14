# Build release binary
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Run clippy with pedantic lints
lint:
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

# Install drawz binary to ~/.cargo/bin
install:
    cargo install --path crates/drawz-cli
