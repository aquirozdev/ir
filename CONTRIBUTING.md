# Contributing

Read the draft IR, architecture and experiment protocol before changing semantics.

```bash
cargo fmt --all
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/check_repository.py
```

For format changes update AST, schema, examples and docs together. For behavior changes include independent positive, negative and boundary tests. Never weaken authorization or silently ignore unknown fields to make generated programs pass.

Keep benchmark development and final evaluation separate. Do not commit private tasks, model weights, tokens, personal data or fabricated results. PRs should describe final behavior, validation and limitations. Keep Cargo.lock committed and use locked builds. Run bash scripts/ci.sh for the full regression workflow. Record any unavailable checks and distinguish local passes from hosted CI execution.

The repository has no selected license yet; resolve licensing before accepting substantial external contributions or redistributing builds.
