use sir_ir::Application;
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "validate" {
        eprintln!("Usage: sir validate <application.json>");
        return ExitCode::from(2);
    }
    let app = match fs::read_to_string(&args[1])
        .map_err(|e| e.to_string())
        .and_then(|text| serde_json::from_str::<Application>(&text).map_err(|e| e.to_string()))
    {
        Ok(app) => app,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let diagnostics = sir_validator::validate(&app);
    println!(
        "{}",
        serde_json::to_string_pretty(&diagnostics).expect("serializable diagnostics")
    );
    if diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
