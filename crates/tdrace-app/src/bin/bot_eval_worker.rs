//! Bot evaluation worker (spec 083:
//! `specs/083_evorl_bot_driving_style_and_performance_optimization.md`).
//!
//! Reads one JSON race request (`EvalRequest`) per line on stdin and writes one JSON result
//! (`EvalResult`) per line on stdout. Offline calibration (`tools/bot_tuning`) keeps one worker per
//! process for its whole life. Bad input never stops the worker: the result carries `error` and the
//! next line is read. The request and result types live in `bot_harness` so tests run the same code.
//!
//! `bot_eval_worker --list-launch-circuits` prints the launch circuits as a JSON array of
//! `{"module", "id"}` and exits.
//! `bot_eval_worker --list-all-circuits` prints all closed-loop circuits across all 7 modalities.

use std::io::{BufRead, Write};

use tdrace_app::ai::bot_harness::{all_modality_circuit_cars, eval_race, launch_circuit_cars, EvalRequest, EvalResult};

fn main() {
    if std::env::args().any(|a| a == "--list-launch-circuits") {
        let list: Vec<_> = launch_circuit_cars().iter().map(|c| serde_json::json!({"module": c.module, "id": c.id})).collect();
        println!("{}", serde_json::Value::Array(list));
        return;
    }
    if std::env::args().any(|a| a == "--list-all-circuits") {
        let list: Vec<_> = all_modality_circuit_cars()
            .into_iter()
            .filter(|c| {
                tdrace_core::catalog::find(c.id, Some(c.module))
                    .and_then(|entry| entry.load().ok())
                    .is_some_and(|t| t.spline.closed && t.spline.total_length() > 0.0 && t.checkpoints.len() >= 2)
            })
            .map(|c| serde_json::json!({"module": c.module, "id": c.id}))
            .collect();
        println!("{}", serde_json::Value::Array(list));
        return;
    }
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let result = match serde_json::from_str::<EvalRequest>(&line) {
            Ok(req) => eval_race(&req),
            Err(e) => EvalResult::error(format!("bad request: {e}")),
        };
        let json = serde_json::to_string(&result).unwrap_or_else(|e| format!("{{\"error\":\"cannot encode result: {e}\"}}"));
        if writeln!(out, "{json}").and_then(|_| out.flush()).is_err() {
            break;
        }
    }
}
