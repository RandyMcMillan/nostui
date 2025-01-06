alias d := doc
alias r := run
alias t := cargo-test
alias b := build
alias rr := run-release
alias cw := cargo-watch
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --choose || true

clippy:
    cargo clippy --all-targets --all-features

deny:
    cargo deny check

cargo-test:
    cargo test

run:
    cargo run --bin nostui || true

install-all:install
install:
    cargo install --path .

build:
    cargo build --bins || true

build-examples:
    cargo build --examples || true

run-release:
    cargo run --release --bin nostui || true

doc:
    cargo doc --open --offline

# Update and then commit the `Cargo.lock` file
update-cargo-dependencies:
    cargo update
    git commit Cargo.lock -m "update(cargo): \`Cargo.lock\`" -m "$(git diff Cargo.lock)"

# Future incompatibility report, run regularly
cargo-future:
    cargo check --future-incompat-report

cargo-watch:
    cargo watch -x check -x test -x build

# Format the entire Rust code
fmt:
	cargo fmt

# Run benches (unstable)
bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p nostr

# Check cargo duplicate dependencies
dup:
    cargo tree -d

# Remove artifacts that cargo has generated
clean:
	cargo clean

# Build and serve the book
book:
    cd book && just serve

# Count the lines of codes of this project
loc:
	@echo "--- Counting lines of .rs files (LOC):" && find . -type f -name "*.rs" -not -path "*/target/*" -exec cat {} \; | wc -l
