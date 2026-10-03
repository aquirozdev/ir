# SIR — Semantic Intermediate Representation

**Can a specialized representation and deterministic runtime reduce the model size needed to construct correct software?**

SIR is an early research project for declarative business applications. Its first working slice interprets typed JSON commands directly over SQLite. The intended full pipeline is specification → small local model → typed SIR → deterministic runtime.

## Run the working demo

Install Rust using [rustup](https://rustup.rs/). This repository pins Rust 1.85.1 and commits its dependency lockfile.

```bash
cargo run --locked -p sir-cli -- demo
```

The demo executes actual SIR policies, preconditions and effects. It verifies that an employee cannot approve their own expense, their manager can approve it, a second approval fails, and a negative amount fails its invariant without changing state. The JSON output contains four checks derived from these executions.

For the same checks used by GitHub Actions:

```bash
bash scripts/ci.sh
```

## Execute your own command against a persistent database

```bash
cargo run --locked -p sir-cli -- validate examples/expenses.sir.json
cargo run --locked -p sir-cli -- init examples/expenses.sir.json expenses.db examples/expenses.seed.json
cargo run --locked -p sir-cli -- exec examples/expenses.sir.json expenses.db ApproveExpense Employee manager examples/approve.inputs.json
cargo run --locked -p sir-cli -- inspect examples/expenses.sir.json expenses.db
```

Use a fresh database for initialization. Repeating the approval produces `PRECONDITION_FAILED`. Running it with principal `Employee employee` on a pending expense produces `UNAUTHORIZED`.

The CLI is a **local administrative interface**. Principal arguments come from its trusted caller; they are not remote authentication. A future HTTP adapter must establish identity independently and authorize reads separately. `init` and `inspect` are administrative operations, not public application commands.

## Implemented

- Versioned strict JSON AST, matching schema and semantic type checker.
- String, signed 64-bit integer, boolean and typed entity references.
- Required boolean command policies, preconditions and bounded `set` effects.
- Entity-wide invariants evaluated after all staged effects.
- SQLite immediate transactions, reference integrity, atomic seeding and persistent state.
- Structured runtime errors, declared resource limits and program/database compatibility checks.
- Regression tests for authorization, typing, persistence, concurrency, resource bounds and rollback after a SQLite write error.
- Push, pull-request and manual CI with test logs, demo JSON and provenance artifacts.

The initial implementation stages a bounded state snapshot in memory before committing changed records. It is suitable for small experiments, not large production databases.

## Not implemented yet

Natural-language inference, `sir build`, create/delete effects, general queries, enum/optional/money/time types, migrations, HTTP serving, model training and an ML benchmark. There are **no model accuracy or frontier-comparison results**. Current tests measure runtime regression behavior only.

## Documentation

| Document | Purpose |
| --- | --- |
| [Architecture](docs/architecture.md) | Runtime and trust boundaries |
| [Draft IR](docs/ir-v0.md) | Accepted wire format and execution semantics |
| [Expense specification](examples/expenses.spec.md) | Behavior covered by the first working slice |
| [Experiment protocol](docs/experiment.md) | Baselines, fairness, holdouts and decision gates |
| [Roadmap](docs/roadmap.md) | Completed slices and next evidence gates |
| [CI](docs/ci.md) | Checks, artifacts and known runner blocker |
| [Decisions](docs/decisions.md) | Design choices and open questions |
| [Research notes](docs/research.md) | Primary integration references |
| [Contributing](CONTRIBUTING.md) | Development and review expectations |

The first model experiment will compare identical weights generating SIR, conventional source and source using an equivalent helper library. Runtime assistance must be counted when attributing capability gains to representation.

No license has been selected yet. Choose one before distributing the project as reusable open-source software.
