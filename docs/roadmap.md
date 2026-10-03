# Roadmap

Milestones are evidence gates, not calendar promises.

| Slice | Deliverable | Status |
| --- | --- | --- |
| Foundation | Versioned JSON AST, static checker, Rust workspace | Implemented |
| First execution | Typed policies/preconditions, set effects, SQLite, invariants, administrative seeding | Implemented for the expense domain |
| Regression CI | Locked tests, concurrency and rollback checks, executable demo, public artifacts | Implemented; hosted execution blocked by account billing |
| Coverage expansion | 30 hand-authored programs, create/delete semantics, reservations/inventory primitives | Pending |
| Evaluation harness | Neutral behavioral interface, independent tests, mutation checks, isolated submissions | Pending |
| Local inference | Pinned licensed model, constrained output, bounded repairs, resource accounting | Pending |
| Same-model pilot | Source/SIR/helper-library arms with paired measurements | Pending |
| Training and tiny models | Audited synthetic pairs, fresh holdouts, measured scaling | Pending |

## Next implementation

Specify deterministic creation IDs and reference-safe deletion, then implement create/delete effects with tests. Expand hand-authored examples to inventory and reservations after defining the additional semantics they require. Do not imply current set-only commands express those systems already.

Build the evaluation protocol before model training. Integrate local inference only after meaningful human-authored task coverage exists; compare the same checkpoint across representations before fine-tuning.

Defer frontend generation, LLVM/MLIR, distributed storage, general plugins, IDE tooling and multi-agent orchestration.
