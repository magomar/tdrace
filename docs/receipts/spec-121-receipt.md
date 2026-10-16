---
type: validation_receipt
schema_version: "1.0"
spec: "specs/121_two_stage_lazy_circuit_menu_loading_and_compile_time_card_metadata.md"
epic: "tdrace-sp121-q2ebs"
candidate_commit: "effb65b6be1f83365c6438bb92db2064c6f65bf0"
verifier: "local-user"
evaluated_at: "2026-10-16T13:51:26Z"
command: "cargo test -p tdrace-app --test track_preview_tests -- test_spec121 --test-threads=1"
exit_code: 0
duration_ms: 14959
status: passed
---

# 🧾 Validation Receipt: Spec 121

- **Candidate Commit**: `effb65b6be1f83365c6438bb92db2064c6f65bf0`
- **Spec**: `specs/121_two_stage_lazy_circuit_menu_loading_and_compile_time_card_metadata.md`
- **Command**: `cargo test -p tdrace-app --test track_preview_tests -- test_spec121 --test-threads=1`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 4 tests
test test_spec121_compile_time_embedded_circuit_metadata_and_thumbnail_parity ... ok
test test_spec121_lazy_menu_category_switch_resolves_only_selected_circuit ... ok
test test_spec121_menu_track_cache_returns_shared_arc_without_cloning ... ok
test test_spec121_menu_track_load_preserves_dossier_preview_and_runtime_geometry_parity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 11.07s


stderr:
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.87s
     Running tests/track_preview_tests.rs (target/debug/deps/track_preview_tests-ba4705f857b1b974)

```
