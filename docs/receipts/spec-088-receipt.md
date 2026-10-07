---
type: validation_receipt
schema_version: "1.0"
spec: "specs/088_rallycross_jokers_built_from_openstreetmap_joker_ways.md"
epic: "tdrace-4ven"
candidate_commit: "32c39e3cfd105547d4b6a2d32e32e125d7c3aa0a"
verifier: "local-user"
evaluated_at: "2026-10-05T15:08:48Z"
command: "cargo test -p tdrace-app --test rally_tracks_tests --test ai_tests && ruff check scripts/osm_importer.py"
exit_code: 0
duration_ms: 152173
status: passed
---

# 🧾 Validation Receipt: Spec 088

- **Candidate Commit**: `32c39e3cfd105547d4b6a2d32e32e125d7c3aa0a`
- **Spec**: `specs/088_rallycross_jokers_built_from_openstreetmap_joker_ways.md`
- **Command**: `cargo test -p tdrace-app --test rally_tracks_tests --test ai_tests && ruff check scripts/osm_importer.py`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 11 tests
test test_bot_profiles_creation ... ok
test test_bot_ai_strategic_joker_lap_decision ... ok
test test_bot_ai_slipstream_drafting_and_slingshot_pack_racing ... ok
test test_bot_ai_collision_avoidance ... ok
test test_bot_ai_dynamic_traffic_avoidance ... ok
test test_bot_ai_multi_route_split_junction_navigation ... ok
test test_multi_bot_branching_race_simulation ... ok
test test_bot_ai_cornering_slowdown ... ok
test test_bot_ai_steering_and_throttle_on_straight ... ok
test test_bot_ai_on_kart_grid_positions ... ok
test test_bot_ai_strategic_joker_rx_race_compliance has been running for over 60 seconds
test test_bot_ai_strategic_joker_rx_race_compliance ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.19s


running 22 tests
test test_dirt_figure_eight_horizontal_flat_dirt_arena ... ok
test test_dirt_figure_eight_jump_ramps_proportional_trajectory ... ok
test test_blyton_ids_resolve_to_croft ... ok
test test_holjes_rx_joker_lap_time_delta_simulation ... ok
test test_silverstone_and_yas_marina_ids_resolve_to_their_replacements ... ok
test test_rx_race_ignores_the_joker_layout_pick ... ok
test test_export_and_save_rally_tracks_to_disk ... ok
test test_world_rx_jump_ramps_dirt_surface_and_containment_landing ... ok
test test_world_rx_tracks_jump_ramps_and_mixed_surfaces ... ok
test test_rally_tracks_centerline_driving_and_no_wall_obstructions ... ok
test test_all_20_world_rx_circuits_have_valid_joker_track_networks ... ok
test test_rx_joker_branch_never_turns_tighter_than_3_m ... ok
test test_rx_races_get_the_joker_rule_and_other_races_do_not ... ok
test test_rx_joker_costs_lap_time ... ok
test test_rx_layout_checkpoints_lie_on_their_route_in_driving_order ... ok
test test_rx_joker_road_reads_as_its_own_surface_not_runoff ... ok
test test_rally_race_session_simulation_on_new_tracks ... ok
test test_rally_module_tracks_integrity_and_validation ... ok
test test_rx_joker_branch_has_walls_that_stay_off_every_road ... ok
test test_famous_rally_tracks_in_track_manager_and_menu_resolution ... ok
test test_rx_joker_walls_are_rebuilt_on_load_and_never_saved ... ok
test test_rx_car_driving_the_joker_route_gets_its_lap_and_its_joker has been running for over 60 seconds
test test_rx_car_driving_the_joker_route_gets_its_lap_and_its_joker ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 83.10s

All checks passed!

stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 12.1 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.37s
     Running tests/ai_tests.rs (target/debug/deps/ai_tests-1df1b69323a86663)
     Running tests/rally_tracks_tests.rs (target/debug/deps/rally_tracks_tests-6d7fb059065c5731)
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
