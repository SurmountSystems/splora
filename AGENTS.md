# electrs

## Rules

1. You are an expert Rust developer.
2. You are an expert Bitcoin developer.
3. If you are unsure of a change, ask the developer to make a choice proactively.
4. Lineage (ancestry, not merge remotes): romanz/electrs, then Blockstream Esplora, then mempool/electrs, then SurmountSystems/splora. This tree forked **Mempool electrs** (`https://github.com/mempool/electrs`). It did **not** fork Blockstream. "Update mempool upstream" means that Mempool repo. Never merge `Blockstream/electrs`, `Blockstream/esplora`, or `romanz/electrs` unless the operator names that remote. FORK.md §1 is the full pin.
5. Two slices run as two L2 coordinators at the same time. Separate coordinators. One does not own both. Each coordinator stays listed until its own report is written. Do not exit a coordinator after handing work to a specialist and leave one row on the list. Web and native are the usual pair.
6. "Finished" means the approved plan's owed outcome, not a passing test on one slice. Before saying finished, read the session plan and name what landed and what that plan still owes. A green `cargo test` is evidence for that test. It is not the explorer.
7. Do not leave `trunk serve` or a headless browser running. Do not start them unless the operator asks for that serve in the same message. If a check starts one, stop it in the same command. A killed serve stays killed.

## Before testing

- Run cargo fmt (from root)
  - command: `cargo fmt`

## Testing

- Run the checks script
  - `./scripts/checks.sh`
- Run with tests only when a test is added or changed
  - `INCLUDE_TESTS=1 ./scripts/checks.sh`
