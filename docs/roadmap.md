# Roadmap

Milestones are evidence gates, not calendar promises.

| Stage | Deliverable | Acceptance criterion | Status |
| --- | --- | --- | --- |
| 0 | Repository, draft entity AST, validator, docs, CI | Entity examples and rejection tests pass | Initialized |
| 1 | Expression and command semantics | Written operational rules and 30 hand-authored programs spanning agreed scope | Pending |
| 2 | SQLite command execution | Atomic create/update, rollback, reference integrity, authorization and concurrency tests | Pending |
| 3 | Neutral evaluation harness | Public task interface, independently authored tests, mutation checks and isolated execution | Pending |
| 4 | Local inference adapter | Pinned licensed weights, constrained output, bounded repairs and resource accounting | Pending |
| 5 | Pilot A/B/C | Reproducible paired results and decision on representation/runtime benefit | Pending |
| 6 | Procedural data and SFT | Audited specification pairs, split provenance and fresh confirmatory evaluation | Pending |
| 7 | Tiny-model and external validation | Measured CPU-only results, external tasks and explicit limitations | Pending |

## Immediate next implementation

1. Write an expense-approval behavioral specification and independent positive/negative cases.
2. Specify record identity, integer bounds, reference semantics and command failure behavior.
3. Add typed command inputs, equality/boolean expressions and update effects with static checks.
4. Build the first transaction executor and verify authorization and rollback.
5. Expand to reservations and inventory only when the first end-to-end case works.

Defer custom textual syntax, frontend generation, LLVM/MLIR, distributed storage, general plugins, IDE tooling, multi-agent orchestration and model training. Do not promise a natural-language demo until inference and execution are both implemented.
