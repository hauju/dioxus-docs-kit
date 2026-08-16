# Build the application
[group("dev")]
build:
    cargo build --workspace

# Run tests (matches CI)
[group("dev")]
test:
    cargo test --workspace

# Start the application
[group("dev")]
serve:
    dx serve

# Rebuild the kit's precompiled stylesheet (matches the CI freshness check).
# Uses the lockfile-pinned tailwind binary — bunx may silently resolve a newer
# cached version, which changes the output byte-for-byte and fails CI.
[group("dev")]
css:
    bun install --frozen-lockfile
    ./node_modules/.bin/tailwindcss -i crates/dioxus-docs-kit/tailwind.css -o crates/dioxus-docs-kit/assets/docs-kit.css --minify

# Format check (matches CI)
[group("lint")]
fmt:
    cargo fmt --all --check

# Clippy (matches CI)
[group("lint")]
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Unused dependency check (matches CI)
[group("lint")]
machete:
    cargo machete

# Run all lints and tests (matches CI)
[group("lint")]
check: fmt clippy machete test
