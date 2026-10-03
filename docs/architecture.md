# Architecture

## Working pipeline

`sir-ir` defines strict typed declarations. `sir-validator` resolves entity/binding/field references and checks expressions, policies and effect types. `sir-runtime` interprets validated commands over SQLite. `sir-cli` validates programs, initializes state, invokes commands, inspects state and runs an executable demo.

There is no model adapter yet. The implemented input is a hand-authored SIR JSON program.

## Command transaction

Resolve command and trusted principal type → begin SQLite immediate transaction → load bounded consistent state → validate input values and references → authorize → check preconditions → stage ordered effects → check reference/type integrity and all invariants → write changed records → commit.

Failures drop the uncommitted transaction. A regression test injects a SQLite failure on a command's second record write and verifies that the first write is rolled back. Another uses two connections/threads to approve one expense concurrently; exactly one succeeds. Invariants and authorization reads share the transaction boundary with writes.

The runtime stages a bounded full state snapshot, not a production query planner. SQL uses fixed statements and bound values; the IR cannot supply SQL strings. The database records its installed program and rejects incompatible programs. No automatic migrations exist.

## Identity and administration

`Runtime::execute` takes a trusted `Principal {entity,id}` from its host and input values separately. Inputs cannot override the `actor` binding. Principal existence is checked in transactional state. The local CLI's actor arguments are trusted administrative input, not authentication.

A remote host must authenticate separately, protect admin seeding and snapshot access, and handle error disclosure. This release has no HTTP adapter or production authentication. Policies enforce their explicit predicates; type checking cannot prove those predicates express the specification correctly.

## Determinism and bounds

No implicit time, randomness or IDs are generated. Identical program/state/input/principal produce identical semantic outcomes. Lock contention and storage failure are environmental failures, not deterministic successful outcomes.

Programs, records, field counts, paths, expressions, strings and evaluation visits are bounded. No shell, arbitrary source, reflection, custom SQL or network calls are available in SIR. The bounded interpreter is not an OS sandbox; untrusted source-code baselines will still need isolated processes and resource controls.

## Next slices

1. Create/delete effects with identity, referential integrity and deletion rules specified first.
2. Neutral behavioral evaluation protocol and more hand-authored domains.
3. Local generation adapter with grammar constraints and bounded static repairs.
4. Same-checkpoint SIR/source/helper-library pilot before training.

Cedar and llama.cpp remain integration candidates. SQLite is now integrated through pinned rusqlite with bundled SQLite. A text syntax, source backend and HTTP adapter are deferred.
