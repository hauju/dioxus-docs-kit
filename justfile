# Build the application
[group("dev")]
build:
    cargo build

# Run tests
[group("dev")]
test:
    cargo test

# Start the application
[group("dev")]
serve:
    dx serve
