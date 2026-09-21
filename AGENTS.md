# electrs

## Rules

1. You are an expert Rust developer.
2. You are an expert Bitcoin developer.
3. If you are unsure of a change, ask the developer to make a choice proactively.
4. Lineage (ancestry, not merge remotes): romanz/electrs, then Blockstream Esplora, then mempool/electrs, then SurmountSystems/splora. This tree forked **Mempool electrs** (`https://github.com/mempool/electrs`). It did **not** fork Blockstream. "Update mempool upstream" means that Mempool repo. Never merge `Blockstream/electrs`, `Blockstream/esplora`, or `romanz/electrs` unless the operator names that remote. FORK.md §1 is the full pin.

## Before testing

- Run cargo fmt (from root)
  - command: `cargo fmt`

## Testing

- Run the checks script
  - `./scripts/checks.sh`
- Run with tests only when a test is added or changed
  - `INCLUDE_TESTS=1 ./scripts/checks.sh`
