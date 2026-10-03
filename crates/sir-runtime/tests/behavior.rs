use serde_json::{json, Value};
use sir_ir::{Application, Effect, Expr, Literal};
use sir_runtime::{Principal, Record, Runtime};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Barrier,
    },
};

fn app() -> Application {
    serde_json::from_str(include_str!("../../../examples/expenses.sir.json")).unwrap()
}
fn seed() -> Vec<Record> {
    serde_json::from_str(include_str!("../../../examples/expenses.seed.json")).unwrap()
}
fn ready() -> Runtime {
    let mut runtime = Runtime::memory(app()).unwrap();
    runtime.seed(seed()).unwrap();
    runtime
}
fn actor(id: &str) -> Principal {
    Principal {
        entity: "Employee".into(),
        id: id.into(),
    }
}
fn inputs() -> BTreeMap<String, Value> {
    BTreeMap::from([("expense".into(), json!("expense1"))])
}
fn amount(value: Value) -> BTreeMap<String, Value> {
    let mut input = inputs();
    input.insert("amount_minor".into(), value);
    input
}
fn expense(runtime: &Runtime) -> Record {
    runtime
        .snapshot()
        .unwrap()
        .into_iter()
        .find(|r| r.entity == "Expense")
        .unwrap()
}
struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/test-data")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn db(&self) -> PathBuf {
        self.0.join("state.db")
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn manager_can_approve_and_repeated_approval_is_rejected() {
    let mut runtime = ready();
    let changed = runtime
        .execute("ApproveExpense", actor("manager"), inputs())
        .unwrap();
    assert_eq!(changed.changed.len(), 1);
    assert_eq!(expense(&runtime).fields["status"], json!("approved"));
    let before = runtime.snapshot().unwrap();
    assert_eq!(
        runtime
            .execute("ApproveExpense", actor("manager"), inputs())
            .unwrap_err()
            .code,
        "PRECONDITION_FAILED"
    );
    assert_eq!(runtime.snapshot().unwrap(), before);
}
#[test]
fn employee_and_unrelated_principal_are_denied_without_mutation() {
    let mut runtime = ready();
    let before = runtime.snapshot().unwrap();
    for id in ["employee", "stranger"] {
        assert_eq!(
            runtime
                .execute("ApproveExpense", actor(id), inputs())
                .unwrap_err()
                .code,
            "UNAUTHORIZED"
        );
        assert_eq!(runtime.snapshot().unwrap(), before);
    }
}
#[test]
fn identity_cannot_be_overridden_via_inputs() {
    let mut runtime = ready();
    let mut input = inputs();
    input.insert("actor".into(), json!("manager"));
    assert_eq!(
        runtime
            .execute("ApproveExpense", actor("employee"), input)
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
    assert_eq!(expense(&runtime).fields["status"], json!("pending"));
}
#[test]
fn missing_records_wrong_principal_type_and_unknown_command_fail_closed() {
    let mut runtime = ready();
    assert_eq!(
        runtime
            .execute("ApproveExpense", actor("missing"), inputs())
            .unwrap_err()
            .code,
        "NOT_FOUND"
    );
    let wrong = Principal {
        entity: "Expense".into(),
        id: "expense1".into(),
    };
    assert_eq!(
        runtime
            .execute("ApproveExpense", wrong, inputs())
            .unwrap_err()
            .code,
        "UNAUTHORIZED"
    );
    assert_eq!(
        runtime
            .execute("Unknown", actor("manager"), inputs())
            .unwrap_err()
            .code,
        "UNKNOWN_COMMAND"
    );
    let mut input = inputs();
    input.insert("expense".into(), json!("missing"));
    assert_eq!(
        runtime
            .execute("ApproveExpense", actor("manager"), input)
            .unwrap_err()
            .code,
        "NOT_FOUND"
    );
}
#[test]
fn missing_extra_and_incorrectly_typed_inputs_fail() {
    let mut runtime = ready();
    for input in [
        BTreeMap::new(),
        BTreeMap::from([("other".into(), json!("expense1"))]),
    ] {
        assert_eq!(
            runtime
                .execute("ApproveExpense", actor("manager"), input)
                .unwrap_err()
                .code,
            "INVALID_INPUT"
        );
    }
    for value in [
        json!("100"),
        json!(1.5),
        json!(null),
        json!(true),
        json!(9223372036854775808_u64),
    ] {
        assert_eq!(
            runtime
                .execute("SetExpenseAmount", actor("employee"), amount(value))
                .unwrap_err()
                .code,
            "TYPE_MISMATCH"
        );
    }
}
#[test]
fn amount_boundaries_and_negative_values_preserve_invariants() {
    let mut runtime = ready();
    for value in [0, 1, i64::MAX] {
        runtime
            .execute("SetExpenseAmount", actor("employee"), amount(json!(value)))
            .unwrap();
        assert_eq!(expense(&runtime).fields["amount_minor"], json!(value));
    }
    let before = runtime.snapshot().unwrap();
    for value in [-1, i64::MIN] {
        assert_eq!(
            runtime
                .execute("SetExpenseAmount", actor("employee"), amount(json!(value)))
                .unwrap_err()
                .code,
            "INVARIANT_FAILED"
        );
        assert_eq!(runtime.snapshot().unwrap(), before);
    }
}
#[test]
fn late_invariant_failure_reverts_every_effect() {
    let mut program = app();
    program.commands[1].effects.insert(
        0,
        Effect::Set {
            target: vec!["expense".into(), "status".into()],
            value: Expr::Literal {
                value: Literal::String("approved".into()),
            },
        },
    );
    let mut runtime = Runtime::memory(program).unwrap();
    runtime.seed(seed()).unwrap();
    let before = runtime.snapshot().unwrap();
    assert_eq!(
        runtime
            .execute("SetExpenseAmount", actor("employee"), amount(json!(-1)))
            .unwrap_err()
            .code,
        "INVARIANT_FAILED"
    );
    assert_eq!(runtime.snapshot().unwrap(), before);
}
#[test]
fn invalid_seed_is_atomic_and_dangling_references_are_rejected() {
    let mut runtime = Runtime::memory(app()).unwrap();
    let mut rows = seed();
    rows.last_mut()
        .unwrap()
        .fields
        .insert("amount_minor".into(), json!(-1));
    assert_eq!(runtime.seed(rows).unwrap_err().code, "INVARIANT_FAILED");
    assert!(runtime.snapshot().unwrap().is_empty());
    let mut rows = seed();
    rows.last_mut()
        .unwrap()
        .fields
        .insert("employee".into(), json!("missing"));
    assert_eq!(runtime.seed(rows).unwrap_err().code, "NOT_FOUND");
    assert!(runtime.snapshot().unwrap().is_empty());
    runtime.seed(seed()).unwrap();
    assert_eq!(
        runtime.seed(seed()).unwrap_err().code,
        "ALREADY_INITIALIZED"
    );
}
#[test]
fn duplicate_and_malformed_seed_records_fail() {
    let mut runtime = Runtime::memory(app()).unwrap();
    let mut rows = seed();
    rows.push(rows[0].clone());
    assert_eq!(runtime.seed(rows).unwrap_err().code, "DUPLICATE_RECORD");
    let mut rows = seed();
    rows[0].fields.insert("extra".into(), json!(1));
    assert_eq!(runtime.seed(rows).unwrap_err().code, "INVALID_RECORD");
    let mut rows = seed();
    rows.last_mut().unwrap().id = "invalid id".into();
    assert_eq!(runtime.seed(rows).unwrap_err().code, "INVALID_ID");
    assert!(runtime.snapshot().unwrap().is_empty());
}
#[test]
fn invalid_program_cannot_be_installed() {
    let mut program = app();
    program.commands[0].policy = Expr::Literal {
        value: Literal::String("allow".into()),
    };
    let error = match Runtime::memory(program) {
        Ok(_) => panic!("invalid program accepted"),
        Err(e) => e,
    };
    assert_eq!(error.code, "INVALID_PROGRAM");
}
#[test]
fn persistence_survives_reopening_and_program_mismatch_is_rejected() {
    let temp = TempDir::new();
    {
        let mut runtime = Runtime::open(app(), temp.db()).unwrap();
        runtime.seed(seed()).unwrap();
        runtime
            .execute("ApproveExpense", actor("manager"), inputs())
            .unwrap();
    }
    let runtime = Runtime::open(app(), temp.db()).unwrap();
    assert_eq!(expense(&runtime).fields["status"], json!("approved"));
    let mut different = app();
    different.name = "DifferentApp".into();
    let error = match Runtime::open(different, temp.db()) {
        Ok(_) => panic!("mismatched program accepted"),
        Err(e) => e,
    };
    assert_eq!(error.code, "PROGRAM_MISMATCH");
}
#[test]
fn sqlite_failure_on_second_write_rolls_back_first_write() {
    let temp = TempDir::new();
    let mut program = app();
    program.commands[1].effects.push(Effect::Set {
        target: vec!["other".into(), "amount_minor".into()],
        value: Expr::Literal {
            value: Literal::Int(100),
        },
    });
    program.commands[1].inputs.push(sir_ir::Field {
        name: "other".into(),
        field_type: sir_ir::FieldType::Ref {
            entity: "Expense".into(),
        },
    });
    let mut runtime = Runtime::open(program, temp.db()).unwrap();
    let mut rows = seed();
    let mut second = rows.last().unwrap().clone();
    second.id = "expense2".into();
    rows.push(second);
    runtime.seed(rows).unwrap();
    let before = runtime.snapshot().unwrap();
    let conn = rusqlite::Connection::open(temp.db()).unwrap();
    conn.execute_batch("CREATE TRIGGER fail_second_write BEFORE UPDATE ON sir_records WHEN NEW.id = 'expense2' BEGIN SELECT RAISE(ABORT, 'injected storage failure'); END;").unwrap();
    let mut input = amount(json!(100));
    input.insert("other".into(), json!("expense2"));
    assert_eq!(
        runtime
            .execute("SetExpenseAmount", actor("employee"), input)
            .unwrap_err()
            .code,
        "STORAGE_ERROR"
    );
    assert_eq!(runtime.snapshot().unwrap(), before);
}
#[test]
fn concurrent_approvals_serialize_and_only_one_succeeds() {
    let temp = TempDir::new();
    let mut runtime = Runtime::open(app(), temp.db()).unwrap();
    runtime.seed(seed()).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let db = temp.db();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            let mut runtime = Runtime::open(app(), db).unwrap();
            barrier.wait();
            runtime
                .execute("ApproveExpense", actor("manager"), inputs())
                .map(|_| ())
                .map_err(|e| e.code)
        }));
    }
    let outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(outcomes.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|r| matches!(r, Err("PRECONDITION_FAILED")))
            .count(),
        1
    );
    assert_eq!(expense(&runtime).fields["status"], json!("approved"));
}
#[test]
fn identical_inputs_and_state_have_identical_results() {
    let mut a = ready();
    let mut b = ready();
    a.execute("ApproveExpense", actor("manager"), inputs())
        .unwrap();
    b.execute("ApproveExpense", actor("manager"), inputs())
        .unwrap();
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
}
#[test]
fn comparison_and_boolean_operators_have_defined_behavior() {
    for (expression, expected) in [
        (
            json!({"op":"gt","left":{"op":"literal","value":{"kind":"int","value":2}},"right":{"op":"literal","value":{"kind":"int","value":1}}}),
            true,
        ),
        (
            json!({"op":"lt","left":{"op":"literal","value":{"kind":"int","value":1}},"right":{"op":"literal","value":{"kind":"int","value":2}}}),
            true,
        ),
        (
            json!({"op":"lte","left":{"op":"literal","value":{"kind":"int","value":2}},"right":{"op":"literal","value":{"kind":"int","value":1}}}),
            false,
        ),
        (
            json!({"op":"ne","left":{"op":"literal","value":{"kind":"string","value":"a"}},"right":{"op":"literal","value":{"kind":"string","value":"a"}}}),
            false,
        ),
        (
            json!({"op":"and","args":[{"op":"literal","value":{"kind":"bool","value":true}},{"op":"not","arg":{"op":"literal","value":{"kind":"bool","value":false}}}]}),
            true,
        ),
    ] {
        let mut program = app();
        program.commands[0].requires = vec![serde_json::from_value(expression).unwrap()];
        let mut runtime = Runtime::memory(program).unwrap();
        runtime.seed(seed()).unwrap();
        let result = runtime.execute("ApproveExpense", actor("manager"), inputs());
        assert_eq!(result.is_ok(), expected);
        if !expected {
            assert_eq!(result.unwrap_err().code, "PRECONDITION_FAILED");
        }
    }
}
#[test]
fn resource_limits_fail_without_seeding_partial_state() {
    let mut runtime = Runtime::memory(app()).unwrap();
    let rows = vec![seed()[0].clone(); sir_runtime::MAX_RECORDS + 1];
    assert_eq!(runtime.seed(rows).unwrap_err().code, "LIMIT_EXCEEDED");
    assert!(runtime.snapshot().unwrap().is_empty());
    let mut program = app();
    program.invariants = (0..64)
        .map(|i| sir_ir::Invariant {
            name: format!("Budget{i}"),
            entity: "Employee".into(),
            expression: Expr::And {
                args: vec![
                    Expr::Literal {
                        value: Literal::Bool(true)
                    };
                    64
                ],
            },
        })
        .collect();
    let mut runtime = Runtime::memory(program).unwrap();
    let mut rows = seed();
    for i in 0..1000 {
        let mut row = rows[0].clone();
        row.id = format!("user{i}");
        rows.push(row);
    }
    assert_eq!(runtime.seed(rows).unwrap_err().code, "LIMIT_EXCEEDED");
    assert!(runtime.snapshot().unwrap().is_empty());
}
