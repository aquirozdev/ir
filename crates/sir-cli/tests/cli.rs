use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sir"))
}
#[test]
fn demo_exercises_runtime_and_emits_valid_report() {
    let output = bin().arg("demo").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["sir_version"], "0.2-draft");
    assert_eq!(report["checks"].as_array().unwrap().len(), 4);
    assert!(report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["passed"] == true));
}
#[test]
fn missing_arguments_and_missing_files_have_usage_exit_code() {
    assert_eq!(bin().output().unwrap().status.code(), Some(2));
    assert_eq!(
        bin()
            .args(["validate", "not-a-real-file.json"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn valid_program_has_zero_exit_code_and_empty_diagnostics() {
    let example = format!(
        "{}/../../examples/expenses.sir.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = bin().args(["validate", &example]).output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!([])
    );
}
#[test]
fn persistent_cli_flow_checks_permissions_and_repeated_approval() {
    use serde_json::Value;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/test-data")
        .join(format!("cli-{}-{unique}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let temp = Temp(directory);
    let db = temp.0.join("expenses.db");
    let db = db.to_str().unwrap();
    let root = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let program = format!("{root}/examples/expenses.sir.json");
    let seed = format!("{root}/examples/expenses.seed.json");
    let input = format!("{root}/examples/approve.inputs.json");
    assert!(bin()
        .args(["init", &program, db, &seed])
        .output()
        .unwrap()
        .status
        .success());
    let init = bin().args(["init", &program, db, &seed]).output().unwrap();
    assert_eq!(init.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&init.stdout).unwrap()["code"],
        "ALREADY_INITIALIZED"
    );
    let denied = bin()
        .args([
            "exec",
            &program,
            db,
            "ApproveExpense",
            "Employee",
            "employee",
            &input,
        ])
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&denied.stdout).unwrap()["code"],
        "UNAUTHORIZED"
    );
    let approved = bin()
        .args([
            "exec",
            &program,
            db,
            "ApproveExpense",
            "Employee",
            "manager",
            &input,
        ])
        .output()
        .unwrap();
    assert!(approved.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&approved.stdout).unwrap()["changed"][0]["fields"]
            ["status"],
        "approved"
    );
    let repeated = bin()
        .args([
            "exec",
            &program,
            db,
            "ApproveExpense",
            "Employee",
            "manager",
            &input,
        ])
        .output()
        .unwrap();
    assert_eq!(repeated.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&repeated.stdout).unwrap()["code"],
        "PRECONDITION_FAILED"
    );
    let inspected = bin().args(["inspect", &program, db]).output().unwrap();
    assert!(inspected.status.success());
    let records: Value = serde_json::from_slice(&inspected.stdout).unwrap();
    let expense = records
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["entity"] == "Expense")
        .unwrap();
    assert_eq!(expense["fields"]["status"], "approved");
}
