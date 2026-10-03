use serde::Serialize;
use serde_json::{json, Value};
use sir_ir::Application;
use sir_runtime::{Principal, Record, Runtime, MAX_PROGRAM_BYTES};
use std::{collections::BTreeMap, env, fs, io::Read, process::ExitCode};

fn read<T: serde::de::DeserializeOwned>(path: &str, limit: usize) -> Result<T, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err("input file exceeds size limit".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn print(value: &impl Serialize) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("serializable output")
    );
}
fn runtime_error(error: sir_runtime::Error) -> ExitCode {
    print(&error);
    ExitCode::from(1)
}
fn inputs(expense: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([("expense".into(), Value::String(expense.into()))])
}
fn principal(id: &str) -> Principal {
    Principal {
        entity: "Employee".into(),
        id: id.into(),
    }
}
fn example_runtime() -> Result<Runtime, Box<dyn std::error::Error>> {
    let app = serde_json::from_str(include_str!("../../../examples/expenses.sir.json"))?;
    let seed = serde_json::from_str(include_str!("../../../examples/expenses.seed.json"))?;
    let mut runtime = Runtime::memory(app)?;
    runtime.seed(seed)?;
    Ok(runtime)
}
fn demo() -> Result<Value, Box<dyn std::error::Error>> {
    let mut runtime = example_runtime()?;
    let initial = runtime.snapshot()?;
    let denied = runtime
        .execute("ApproveExpense", principal("employee"), inputs("expense1"))
        .unwrap_err();
    if denied.code != "UNAUTHORIZED" || runtime.snapshot()? != initial {
        return Err("unauthorized approval modified state".into());
    }
    let changed = runtime.execute("ApproveExpense", principal("manager"), inputs("expense1"))?;
    if changed.changed.len() != 1 || changed.changed[0].fields["status"] != json!("approved") {
        return Err("manager approval failed".into());
    }
    let approved = runtime.snapshot()?;
    let repeated = runtime
        .execute("ApproveExpense", principal("manager"), inputs("expense1"))
        .unwrap_err();
    if repeated.code != "PRECONDITION_FAILED" || runtime.snapshot()? != approved {
        return Err("repeated approval modified state".into());
    }
    let mut runtime = example_runtime()?;
    let before = runtime.snapshot()?;
    let mut input = inputs("expense1");
    input.insert("amount_minor".into(), json!(-1));
    let rejected = runtime
        .execute("SetExpenseAmount", principal("employee"), input)
        .unwrap_err();
    if rejected.code != "INVARIANT_FAILED" || runtime.snapshot()? != before {
        return Err("negative expense was not rolled back".into());
    }
    Ok(json!({
        "demo": "expense-approval", "sir_version": sir_ir::VERSION,
        "checks": [
            {"name":"employee cannot approve own expense", "passed":true},
            {"name":"manager approves pending expense", "passed":true},
            {"name":"repeated approval leaves state unchanged", "passed":true},
            {"name":"negative amount fails invariant with rollback", "passed":true}
        ],
        "approved_record": changed.changed[0]
    }))
}
fn run(args: &[String]) -> ExitCode {
    if args == ["demo"] {
        return match demo() {
            Ok(report) => {
                print(&report);
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("demo failed: {error}");
                ExitCode::from(1)
            }
        };
    }
    let valid_usage = matches!(args.first().map(String::as_str), Some("validate"))
        && args.len() == 2
        || matches!(args.first().map(String::as_str), Some("init")) && args.len() == 4
        || matches!(args.first().map(String::as_str), Some("inspect")) && args.len() == 3
        || matches!(args.first().map(String::as_str), Some("exec")) && args.len() == 7;
    if !valid_usage {
        eprintln!("Usage:\n  sir validate <app.json>\n  sir init <app.json> <db> <seed.json>\n  sir exec <app.json> <db> <command> <actor-entity> <actor-id> <inputs.json>\n  sir inspect <app.json> <db>\n  sir demo\n\nLocal/admin CLI: actor arguments are trusted identity; no remote authentication is provided.");
        return ExitCode::from(2);
    }
    let app: Application = match read(&args[1], MAX_PROGRAM_BYTES) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let diagnostics = sir_validator::validate(&app);
    if args[0] == "validate" || !diagnostics.is_empty() {
        print(&diagnostics);
        return if diagnostics.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }
    let mut runtime = match Runtime::open(app, &args[2]) {
        Ok(runtime) => runtime,
        Err(error) => return runtime_error(error),
    };
    match args[0].as_str() {
        "init" => {
            let seed: Vec<Record> = match read(&args[3], sir_runtime::MAX_STATE_BYTES) {
                Ok(seed) => seed,
                Err(error) => {
                    eprintln!("{error}");
                    return ExitCode::from(2);
                }
            };
            match runtime.seed(seed) {
                Ok(()) => {
                    print(&json!({"initialized":true}));
                    ExitCode::SUCCESS
                }
                Err(error) => runtime_error(error),
            }
        }
        "inspect" => match runtime.snapshot() {
            Ok(records) => {
                print(&records);
                ExitCode::SUCCESS
            }
            Err(error) => runtime_error(error),
        },
        "exec" => {
            let input = match read(&args[6], MAX_PROGRAM_BYTES) {
                Ok(input) => input,
                Err(error) => {
                    eprintln!("{error}");
                    return ExitCode::from(2);
                }
            };
            let actor = Principal {
                entity: args[4].clone(),
                id: args[5].clone(),
            };
            match runtime.execute(&args[3], actor, input) {
                Ok(result) => {
                    print(&result);
                    ExitCode::SUCCESS
                }
                Err(error) => runtime_error(error),
            }
        }
        _ => unreachable!(),
    }
}
fn main() -> ExitCode {
    run(&env::args().skip(1).collect::<Vec<_>>())
}
