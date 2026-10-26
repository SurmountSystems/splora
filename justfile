# check-local is the laptop cargo list.
# check-remote is nix flake check on the remote builder.
# Agents do not run these recipes as crate-wide proof.

# Bare `just` lists recipes. It does not run clippy.
[private]
default:
    @just --list

check-local:
    cargo fmt --all --check
    cargo fmt --all -- --check
    cargo check --all-features
    cargo check --no-default-features
    cargo check -F liquid
    cargo check -F electrum-discovery
    cargo check -F electrum-discovery,liquid
    cargo clippy -- -D warnings
    cargo clippy -F electrum-discovery -- -D warnings
    cargo clippy -F liquid -- -D warnings
    cargo clippy -F electrum-discovery,liquid -- -D warnings
    cargo clippy --all -- -D warnings
    cargo test --lib --all-features
    # .github/workflows/ci.yml names the bin electrs. Cargo.toml's bin is splora.
    cargo build --release --bin electrs
    cargo deny --offline --locked check --config cargo-deny.toml
    cargo audit

# Local max-jobs 0. The store queried is the ssh-ng machine in
# ~/.config/nix/machines. A laptop cache hit must not satisfy this.
# Eval stays on this machine. Nothing compiles here.
check-remote:
    nix flake check --option max-jobs 0 --store 'ssh-ng://surmount-1?remote-program=/etc/surmount/niced-nix-daemon-stdio'

install:
    nix build .#splora
    nix build .#splora-liquid
