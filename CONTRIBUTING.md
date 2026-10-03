# Contributing

Read the draft IR, architecture and experiment protocol before changing semantics.

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/check_repository.py
```

For format changes update AST, schema, examples and docs together. For behavior changes include independent positive, negative and boundary tests. Never weaken authorization or silently ignore unknown fields to make generated programs pass.

Keep benchmark development and final evaluation separate. Do not commit private tasks, model weights, tokens, personal data or fabricated results. PRs should describe final behavior, validation and limitations. Before a reproducibility release, generate and commit Cargo.lock, pin dependency and Action revisions, and record environment provenance.

The repository has no selected license yet; resolve licensing before accepting substantial external contributions or redistributing builds.
