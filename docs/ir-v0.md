# Draft IR contract

Status: `0.1-draft`, unstable, entities only. Changing the accepted wire format requires updating schema, Rust structures, fixtures and documentation together.

Top-level keys are exactly `sir_version`, `name`, `entities`. Each entity contains `name` and `fields`. Each field contains `name` and `field_type`. Identifiers match `[A-Za-z_][A-Za-z0-9_]*`. Arrays of entities and fields must be nonempty. Entity names are globally unique; field names are unique within an entity.

| Field kind | Wire representation | Declaration meaning |
| --- | --- | --- |
| String | `{"kind":"string"}` | Text; no runtime length rules yet |
| Int | `{"kind":"int"}` | Integer; runtime range to be specified |
| Bool | `{"kind":"bool"}` | Boolean |
| Ref | `{"kind":"ref","entity":"Employee"}` | Reference to a declared entity |

Forward and self references are valid. Unknown entities and additional JSON object keys are rejected. No nullable types, runtime records, ID generation, uniqueness constraints, enums, commands or expressions are accepted yet. The schema cannot itself enforce cross-entity reference existence or uniqueness by name; use semantic validation.

Example: [expense entities](../examples/expenses.entities.json). This describes entities only; the integer amount is illustrative and is not a finalized money representation.

## Proposed next layer — not accepted by the current parser

Add explicit command inputs, typed expressions, preconditions, authorization and bounded transactional effects. Begin with field paths, literals, equality, boolean operators and single-record create/update operations. Specify null handling, integer overflow, reference equality and failure behavior before implementing each operator.

A later expense approval command should permit only the employee's manager, require a pending status and atomically change it to approved. Boundary tests must cover unauthorized actors, repeated approval, missing references and rollback.

Queries, state machines, money, timestamps, quantifiers and semantic patches follow only after their semantics and cost limits are defined. Generic unbounded recursion and arbitrary code execution are excluded.

## Diagnostics

Semantic diagnostics are an ordered array of `{code, path, message}`. Paths use JSON Pointer syntax. Codes are stable within this draft; messages are for humans. JSON syntax/deserialization errors currently go to stderr as text, so structured parser diagnostics are a future task.
