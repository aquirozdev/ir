# Architecture

## Implemented foundation

`sir-ir` owns serializable declarations. `sir-validator` checks cross-declaration consistency and returns stable diagnostic codes with JSON Pointer paths. `sir-cli` loads JSON and exposes validation. The JSON Schema is an interchange contract; the Rust CLI uses strict deserialization plus semantic checks, not a JSON Schema engine.

## Intended pipeline

1. A specification enters an inference adapter with explicit limits.
2. A local model emits a constrained AST. Grammar validity does not guarantee correct symbol references, types, authorization or requested behavior.
3. Static validation resolves symbols, checks expressions, effects and policy completeness.
4. A bounded repair loop receives static diagnostics only. Evaluation holdout failures must never enter this loop.
5. An immutable validated program is installed in a deterministic runtime.
6. An independently authenticated principal calls a command through a versioned interface.

The runtime interprets SIR directly. Source generation is optional future interoperability, not required for the first experiment.

## Intended command transaction

Resolve and validate input → begin transaction → read consistent state → authorize with default deny → check preconditions → apply bounded effects → verify invariants on resulting state → commit. Any failure rolls back all effects. Authorization reads and writes must share the transaction boundary to prevent time-of-check/time-of-use errors.

SQLite is the proposed initial store. Isolation, concurrent booking races, reference integrity and persistence recovery require tests before claiming correctness. No floating-point money: choose bounded integer minor units plus currency, with explicit overflow and rounding rules. Time and IDs must be supplied through controlled interfaces; deterministic execution means identical program, state, inputs and injected values produce identical results.

## Trust and resource boundaries

SIR must not execute shell, SQL strings, arbitrary source, reflection or network calls. The model's output is untrusted. Restrict input bytes, entity counts, expression depth, relation traversals, rows scanned, effects and execution time. Absence of loops alone does not bound resource cost. The early validation CLI has no production resource-hardening claim.

The HTTP adapter must establish identity independently of user payloads. A caller-provided `actor` is not authentication. Runtime diagnostics should avoid exposing confidential state. Security checks must be tested independently of the model.

## Components to add when needed

- `sir-runtime`: expression interpreter and atomic effects.
- `sir-storage`: SQLite state adapter.
- `sir-auth`: authorization interface; evaluate Cedar integration against actual requirements.
- `sir-model`: generation and bounded semantic repairs.
- `sir-bench`: neutral behavioral protocol and resource accounting.
- `sir-http`: thin authenticated adapter after core execution is reliable.

Do not create empty crates for these components before their interfaces are needed.
