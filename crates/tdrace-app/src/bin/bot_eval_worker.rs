//! Bot evaluation worker (spec 083:
//! `specs/083_evorl_bot_driving_style_and_performance_optimization.md`).
//!
//! Reads one JSON race request (`EvalRequest`) per line on stdin and writes one JSON result
//! (`EvalResult`) per line on stdout. Offline calibration (`tools/bot_tuning`) keeps one worker per
//! process for its whole life. Bad input never stops the worker: the result carries `error` and the
//! next line is read. The request and result types live in `bot_harness` so tests run the same code.

use std::io::{BufRead, Write};

use tdrace_app::ai::bot_harness::{eval_race, EvalRequest, EvalResult};

fn main() {
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
