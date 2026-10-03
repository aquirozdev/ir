# Design decisions

## ADR-001: Direct interpretation

Proposed executable SIR runs in a fixed runtime. This reduces backend variability and makes semantics testable. Tradeoff: useful behavior is supplied by a specialized runtime, restricting scope and confounding comparisons unless helper-library baselines are included.

## ADR-002: JSON AST first

Use strict versioned JSON, with a matching schema and typed Rust structures. Avoid a custom parser initially. JSON is verbose; compact representations must later be tested for token cost and semantic accuracy rather than assumed superior.

## ADR-003: Small workspace

Start with AST, validator and CLI. Add runtime, storage, inference and benchmark crates as working slices, instead of ten empty modules. Rust is the implementation choice; single-binary distribution is a future build/release goal, not today's installation experience.

## ADR-004: Semantics before training

Validate human-written programs and a same-checkpoint pilot before fine-tuning. Training can hide representation flaws and make causal attribution harder.

## ADR-005: Conservative integration

SQLite, llama.cpp and Cedar are candidates, not integrated dependencies. Before adoption verify versions, licensing, CPU compatibility, resource overhead and semantics through primary documentation and experiments. No weight size, runner capacity or latency from earlier brainstorming is treated as a verified project fact.

## Open questions

- Which business tasks define the first domain and expose its limits?
- Which operations are essential without making inference or verification intractable?
- How are identity, optional references, deletion and migrations defined?
- How are authentication, policy evaluation and transaction isolation composed?
- What helper-library baseline fairly counts the runtime's prewritten capability?
- Who authors independent holdouts and controls the isolated runner?
- Which code license and dataset/model licenses should be adopted?
