# Thin wrappers. The operator runs check-local, then check-remote.
# Agents do not run these recipes as crate-wide proof.

# Bare `just` lists recipes. It does not run clippy.
[private]
default:
    @just --list

check-local:
    cargo fmt --all --check
    cargo clippy --all -- -D warnings
    cargo deny --offline --locked check --config cargo-deny.toml
    cargo audit

check-remote:
    nix flake check

install:
    nix build .#splora
    nix build .#splora-liquid
