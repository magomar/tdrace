---
type: validation_receipt
schema_version: "1.0"
spec: "specs/065_vehiclegeneric_bot_ai_and_effects.md"
epic: "tdrace-lt1c"
candidate_commit: "9b2c8573215190b453bcdab0bb1927365c9979ac"
verifier: "local-user"
evaluated_at: "2026-09-29T17:59:31Z"
command: "cargo test -p race-kit --test bot_vehicle_tests --test golden_world && cargo test -p race-ui --test fx_vehicle_tests && cargo check -p cabinet --no-default-features && cargo test -p tdrace-app --test fx_tests --test golden_session"
exit_code: 0
duration_ms: 21481
status: passed
---

# 🧾 Validation Receipt: Spec 065

- **Candidate Commit**: `9b2c8573215190b453bcdab0bb1927365c9979ac`
- **Spec**: `specs/065_vehiclegeneric_bot_ai_and_effects.md`
- **Command**: `cargo test -p race-kit --test bot_vehicle_tests --test golden_world && cargo test -p race-ui --test fx_vehicle_tests && cargo check -p cabinet --no-default-features && cargo test -p tdrace-app --test fx_tests --test golden_session`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 1 test
test bot_drives_a_non_car_vehicle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s


running 2 tests
test golden_oval ... ok
test golden_figure_eight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.69s


running 1 test
test effects_follow_a_non_car_vehicle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 16 tests
test test_debris_roost_particle_emission_by_surface ... ok
test test_dirt_contamination_deposit_on_pavement ... ok
test test_drift_popup_manager ... ok
test test_dirt_roost_is_ground_layer_and_smoke_is_not ... ok
test test_particle_system_emission_and_updates ... ok
test test_no_particles_while_car_is_on_the_air ... ok
test test_multi_surface_skidmark_distinct_palettes ... ok
test test_dirt_roost_count_scales_with_intensity ... ok
test test_gravel_rolling_rut_without_slip ... ok
test test_skidmarks_buffer_lifecycle ... ok
test test_skidmarks_dual_tread_and_irregularity ... ok
test test_effects_manager_integration ... ok
test test_slow_speed_hairpin_distance_accumulation ... ok
test test_persistent_skidmarks_across_multiple_laps ... ok
test test_skidmarks_uv_mapping_and_persistent_capacity ... ok
test test_skidmarks_viewport_culling ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s


running 1 test
test golden_session ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.48s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/bot_vehicle_tests.rs (target/debug/deps/bot_vehicle_tests-c275dcb58e344dfb)
     Running tests/golden_world.rs (target/debug/deps/golden_world-0d88c7283aaceb24)
   Compiling race-ui v0.1.0 (/Users/mario.gomez/workspace/games/tdrace/.claude/worktrees/racing-architecture-analysis-387824/crates/race-ui)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.37s
     Running tests/fx_vehicle_tests.rs (target/debug/deps/fx_vehicle_tests-d1fad09044b46f8e)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/fx_tests.rs (target/debug/deps/fx_tests-7f15c9b553e5cf43)
     Running tests/golden_session.rs (target/debug/deps/golden_session-21782ab6ef64597d)

```
