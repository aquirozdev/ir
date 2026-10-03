# First behavioral specification: expenses

Employees have an email and a reference to their manager. Managers may refer to themselves. An expense has an employee, an integer amount in illustrative minor units and a status of `pending` or `approved`. Amounts must never be negative.

`ApproveExpense(expense)`:

1. Only the expense employee's manager may execute it.
2. The expense must currently be pending.
3. Its status becomes approved atomically.

`SetExpenseAmount(expense, amount_minor)`:

1. Only the expense employee may execute it.
2. The expense must currently be pending.
3. Its amount becomes the supplied integer; any resulting invariant failure leaves all state unchanged.

Initialization is administrative and must reject malformed records, duplicate IDs within an entity, dangling references and violated invariants without inserting partial state. The command interface rejects missing/extra keys and incorrect types. Concurrent approvals of one pending expense must serialize: one succeeds and the other fails its precondition.

No currency, rounding, accounting rules, remote authentication or arbitrary creation API is implied by this fixture. It is a minimal interpreter test, not a complete expense-management product. The public regression cases are development tests, not a hidden evaluation benchmark.
