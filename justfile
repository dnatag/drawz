# Build release binary
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Install drawz binary to ~/.cargo/bin
install:
    cargo install --path crates/drawz-cli
