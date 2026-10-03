---
type: validation_receipt
schema_version: "1.0"
spec: "specs/081_rallycross_joker_lap_segments_for_classic_and_openstreetmap_circuits.md"
epic: "tdrace-vbaq"
candidate_commit: "577b7af8122e3b1f0219b195901de084ce06bd9d"
verifier: "local-user"
evaluated_at: "2026-10-03T22:37:24Z"
command: "cargo test -p tdrace-app --test rally_tracks_tests"
exit_code: 0
duration_ms: 7654
status: passed
---

# 🧾 Validation Receipt: Spec 081

- **Candidate Commit**: `577b7af8122e3b1f0219b195901de084ce06bd9d`
- **Spec**: `specs/081_rallycross_joker_lap_segments_for_classic_and_openstreetmap_circuits.md`
- **Command**: `cargo test -p tdrace-app --test rally_tracks_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 13 tests
test test_dirt_figure_eight_horizontal_flat_dirt_arena ... ok
test test_dirt_figure_eight_jump_ramps_proportional_trajectory ... ok
test test_holjes_rx_joker_lap_time_delta_simulation ... ok
test test_blyton_ids_resolve_to_croft ... ok
test test_silverstone_and_yas_marina_ids_resolve_to_their_replacements ... ok
test test_world_rx_jump_ramps_dirt_surface_and_containment_landing ... ok
test test_export_and_save_rally_tracks_to_disk ... ok
test test_world_rx_tracks_jump_ramps_and_mixed_surfaces ... ok
test test_all_20_world_rx_circuits_have_valid_joker_track_networks ... ok
test test_rally_tracks_centerline_driving_and_no_wall_obstructions ... ok
test test_rally_module_tracks_integrity_and_validation ... ok
test test_famous_rally_tracks_in_track_manager_and_menu_resolution ... ok
test test_rally_race_session_simulation_on_new_tracks ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.58s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 12.0 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/rally_tracks_tests.rs (target/debug/deps/rally_tracks_tests-0c6163381582b64f)
Checking track: holjes_rx
Checking track: lydden_hill
Checking track: hell_rx
Checking track: loheac_rx
Checking track: estering_rx
Checking track: montalegre_rx
Checking track: nyirad_rx
Checking track: kouvola_rx
Checking track: catalunya_rx
Checking track: mettet_rx
Checking track: lavare_rx
Checking track: riga_rx
Checking track: killarney_rx
Checking track: lessay_rx
Checking track: essay_rx
Checking track: dreux_rx
Checking track: croft_rx
Checking track: spa_rx
Checking track: silverstone_rx
Checking track: erx_motor_park

```
