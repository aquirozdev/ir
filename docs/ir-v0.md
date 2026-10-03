# Executable draft IR contract

Status: `0.2-draft`, unstable. The previous entity-only `0.1-draft` wire format is no longer accepted. Schema, AST, fixtures and static checks must change together.

## Declarations

Top-level keys are exactly `sir_version`, `name`, `entities`, `commands`, `invariants`. All are required; commands/invariants may be empty for an entity-only declaration. Entities contain `name`, `fields`; fields contain `name`, `field_type`.

Identifiers are ASCII `[A-Za-z_][A-Za-z0-9_]*` of at most 64 bytes. Entity, command and invariant names are unique within their own namespace; field and input names are unique within their enclosing declaration. `actor` and `record` are reserved input names.

| Field kind | Wire representation | Runtime value |
| --- | --- | --- |
| String | `{"kind":"string"}` | UTF-8 string, at most 4096 bytes |
| Int | `{"kind":"int"}` | Signed 64-bit JSON integer; floats, booleans and out-of-range integers fail |
| Bool | `{"kind":"bool"}` | JSON boolean |
| Ref | `{"kind":"ref","entity":"Employee"}` | Identifier string resolved within the declared entity |

All fields are required and non-null. References are typed by entity, so the same ID in two entities is not the same reference. Forward and self references are permitted. Record IDs are storage metadata, not declared fields, and cannot be assigned through a command.

## Expressions

- `path`: `segments` begins with `actor`, a command input, or invariant binding `record`. Reference fields can be traversed; scalars cannot.
- `literal`: tagged `string`, `int` or `bool` value. There are no reference literals.
- `eq`, `ne`: `left` and `right` must have identical types, including reference entity type.
- `gt`, `gte`, `lt`, `lte`: compare signed integers without arithmetic or overflow.
- `and`, `or`: nonempty boolean `args`, evaluated left to right with short circuit.
- `not`: negates a boolean `arg`.

JSON Schema validates syntax and some local bounds. Semantic validation additionally checks names, references, types, depth and UTF-8 byte length. Schema `maxLength` counts characters and therefore does not replace the runtime byte limit. The CLI uses strict Serde parsing plus the semantic checker, not a general schema engine. Unknown JSON object keys and unsupported operations fail parsing.

## Commands and invariants

A command has exactly `name`, `actor` (entity name), `inputs`, `policy`, `requires`, `effects`. Its policy is a required boolean expression; there is no implicit permit. A deliberately explicit `true` policy permits any existing principal of the declared type. Static validation checks policy type, not whether it matches the author's intended permissions.

The only effect is `{"op":"set","target":[...],"value":expression}`. Target must be a declared field rooted in a reference input, and the value's type must match it. Direct `actor`-rooted assignments are rejected. Aliasing is possible: this restriction does not itself guarantee principal records cannot be reached through input references; command policies must protect every intended write.

Inputs and the principal are bound to immutable values/reference identities. All preconditions see the original transaction snapshot. Effects execute in declared order, and later paths/expressions see earlier staged changes. Invariants run after **all** effects, once per matching entity record using only `record`. Intermediate invariant violations are permitted if final state is valid. Any error rejects the command without committing effects.

## Records and storage

Administrative seed format is an array of `{entity, id, fields}`. `fields` is an exact object matching the entity's declarations. Seed references may target any record in the same seed, including forward/self references. Nonempty existing databases cannot be reseeded.

SQLite stores entity, ID and JSON fields in a fixed parameterized schema. A canonical serialized program is attached to the database; opening it with a different program fails `PROGRAM_MISMATCH`. Migration is not supported. Authorization, state reads, invariant checks and writes share an immediate transaction. Missing records and failed expression evaluation never imply authorization.

## Limits and diagnostics

- Program: 1 MiB serialized; entities, fields, commands, invariants, preconditions, effects and boolean argument lists capped at 64 each.
- Paths: at most 8 segments; expression depth: at most 16 recursive edges.
- State: at most 10,000 records and 16 MiB serialized state; strings at most 4096 UTF-8 bytes.
- Evaluation: 1,000,000 expression visits shared by a command and its invariants, or by seed invariant checking.
- SQLite lock wait: 5 seconds. No comprehensive wall-time/OS-memory sandbox is claimed by these interpreter limits.

Static diagnostics are `{code,path,message}` arrays with JSON Pointer paths. Runtime errors are `{code,message}` objects. CLI exit 0 means success, 1 means semantic/runtime failure, 2 means usage, file or parsing failure. File/parse diagnostics currently go to stderr as text.

See [the executable fixture](../examples/expenses.sir.json). Create/delete, optional types, enums, quantifiers, query plans, arithmetic, money, timestamps, migrations and semantic patches remain future work.
