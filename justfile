# Thin wrappers. The operator runs check-local, then check-remote.
# Agents do not run these recipes as crate-wide proof.

check-local:
    cargo fmt --all --check
    cargo clippy --all -- -D warnings
    cargo deny
    cargo audit

check-remote:
    nix flake check

install:
    nix build .#splora
    nix build .#splora-liquid
