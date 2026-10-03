# Continuous integration

GitHub Actions runs `.github/workflows/ci.yml` on push, pull request and manual dispatch. Its public example checks require no model download or inference API.

## Reproduce locally

```bash
bash scripts/ci.sh
```

This checks formatting, runs locked workspace tests, runs Clippy with warnings treated as errors, checks schema fixtures/documentation links, validates the executable example and executes `sir demo`.

Outputs under ignored `results/`:

- `tests.log`: real Cargo test output.
- `demo.json`: executable demo check results.
- `ci-report.json`: test/demo counts, commit, platform and actual Rust/Cargo versions.

Actions uploads these as `sir-runtime-<commit>` and writes a short job summary. This is regression evidence, not a model capability benchmark. Do not upload private evaluation tasks or results revealing hidden test content through this workflow.

Rust 1.85.1, dependency resolution (`Cargo.lock`) and checkout/upload actions are pinned. The hosted runner label does not freeze every OS package or CPU model, so no hardware reproducibility/performance claim follows from this CI alone.

## Known runner blocker

The initial run's GitHub check annotation states: **“The job was not started because your account is locked due to a billing issue.”** This is an account-level runner blocker, not a failed Rust test. Resolve it in GitHub's account billing/settings, then rerun the latest workflow or dispatch it manually. Local checks can run independently.

Initial blocked run: [37080347931](https://github.com/aquirozdev/ir/actions/runs/37080347931). Do not label a blocked run as passing or invent a successful hosted execution.
