#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p results
python3 - <<'PYTHON'
from pathlib import Path
for name in ('tests.log', 'demo.json', 'ci-report.json'):
    Path('results', name).unlink(missing_ok=True)
PYTHON
cargo fmt --all -- --check
cargo test --workspace --locked 2>&1 | tee results/tests.log
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/check_repository.py
cargo run --locked --quiet -p sir-cli -- validate examples/expenses.sir.json
cargo run --locked --quiet -p sir-cli -- demo > results/demo.json
python3 scripts/ci_report.py
