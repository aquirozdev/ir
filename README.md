# SIR — Semantic Intermediate Representation

**Can a specialized representation and deterministic runtime reduce the model size needed to construct correct software?**

SIR is an early research project for declarative business applications: entities, relationships, commands, permissions, workflows and invariants. The intended pipeline is specification → small local model → typed SIR → deterministic runtime.

There are no model results yet. Frontier parity, CPU latency, RAM usage and accuracy targets are hypotheses, not measured capabilities. This project does not currently build applications from natural language.

## Current status

Implemented in this initialization:

- Rust workspace: `sir-ir`, `sir-validator`, `sir-cli`.
- Draft JSON entity AST and matching JSON Schema.
- Static checks for versions, identifiers, duplicate names, empty declarations and unresolved references.
- `sir validate` with structured diagnostics and nonzero exit codes.
- Example entity model, regression tests and GitHub Actions CI.

Planned: executable commands, expression typing, transactional SQLite runtime, authorization, model integration and behavioral evaluation. Commands, HTTP serving and inference are **not implemented**.

## Quick start

Install Rust using [rustup](https://rustup.rs/). The repository pins its development toolchain.

```bash
cargo test --workspace
cargo run -p sir-cli -- validate examples/expenses.entities.json
```

A valid document prints `[]` and exits 0. Static validation failures exit 1; usage, file and JSON parsing errors exit 2. Validation checks declarations only; it does not test expense approval behavior.

## Documentation

| Document | Purpose |
| --- | --- |
| [Architecture](docs/architecture.md) | Components, runtime boundaries and intended execution |
| [Draft IR](docs/ir-v0.md) | Implemented wire format and proposed semantics |
| [Experiment protocol](docs/experiment.md) | Baselines, fairness, holdouts, metrics and decision gates |
| [Roadmap](docs/roadmap.md) | Ordered milestones and acceptance criteria |
| [Decisions](docs/decisions.md) | Design choices and unresolved questions |
| [Research notes](docs/research.md) | Primary references to investigate before integration |
| [Contributing](CONTRIBUTING.md) | Development and review expectations |

## Research scope

The initial domain is structured business software. Arbitrary algorithms, codecs, compilers, operating systems and general terminal work are outside this first representation. Runtime functionality is part of the system's capability and must be counted when comparing systems.

The first experiment compares the **same model** producing SIR, conventional code and conventional code using an equivalent helper library. A strong runtime advantage alone would not establish a representation advantage.

## Project layout

```text
crates/       AST, semantic checks and CLI
schemas/      Draft interchange contract
examples/     Hand-authored public inputs
benchmark/    Evaluation design; no hidden test set published here
models/       Acquisition and provenance requirements; no weights
scripts/      Repository consistency checks
.github/      CI and contribution templates
```

No license has been selected yet. Choose one before distributing the project as reusable open-source software.
