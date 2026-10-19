---
type: validation_receipt
schema_version: "1.0"
spec: "specs/130_fia_european_autocross_10_car_3_2_3_2_grids_timed_qualifying_practice_30_driver_knockout_format_two_phase_scoring_and_expanded_season_calendars.md"
epic: "tdrace-sp130-7kcbs"
candidate_commit: "bfd382cca2e2b562b13860325dce788b71514d47"
verifier: "local-user"
evaluated_at: "2026-10-19T16:48:13Z"
command: "cargo test -p arcade-race-core --test launch_chute_tests && cargo test -p tdrace-core --test ax_rx_circuit_launch_chute_validation && cargo test -p tdrace-app --test track_editor_launch_chute_tests && cargo test -p tdrace-app --test autocross_tournament_tests && cargo test -p tdrace-app --test tournament_bracket_ui_tests"
exit_code: 0
duration_ms: 46614
status: passed
---

# 🧾 Validation Receipt: Spec 130

- **Candidate Commit**: `bfd382cca2e2b562b13860325dce788b71514d47`
- **Spec**: `specs/130_fia_european_autocross_10_car_3_2_3_2_grids_timed_qualifying_practice_30_driver_knockout_format_two_phase_scoring_and_expanded_season_calendars.md`
- **Command**: `cargo test -p arcade-race-core --test launch_chute_tests && cargo test -p tdrace-core --test ax_rx_circuit_launch_chute_validation && cargo test -p tdrace-app --test track_editor_launch_chute_tests && cargo test -p tdrace-app --test autocross_tournament_tests && cargo test -p tdrace-app --test tournament_bracket_ui_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 20 tests
test test_entry_segment_inside_the_loop_is_invalid ... ok
test test_packed_grid_five_three_rows ... ok
test test_packed_grid_patterns_have_their_row_counts ... ok
test test_packed_grid_staggers_the_two_car_rows ... ok
test test_entry_segment_leads_into_the_loop ... ok
test test_packed_grid_stays_inside_the_pad ... ok
test test_packed_grid_three_two_three_two_rows ... ok
test test_launch_chute_config_round_trips_through_json ... ok
test stamp::test_grid_slots_are_inside_the_pad_and_clear_of_walls ... ok
test stamp::test_stamped_chute_merges_tangent_to_the_loop ... ok
test lap_one::test_turning_back_into_the_chute_is_wrong_way ... ok
test lap_one::test_cars_rank_by_place_on_the_chute_then_after_the_merge ... ok
test lap_one::test_race_starts_in_the_chute_and_lap_one_ends_at_the_finish_line ... ok
test stamp::test_stamp_builds_a_valid_chute_on_a_plain_circuit ... ok
test stamp::test_validation_rejects_a_chute_with_an_open_end ... ok
test stamp::test_stamp_refuses_bad_requests_and_leaves_the_track_alone ... ok
test stamp::test_stamp_survives_a_json_round_trip ... ok
test stamp::test_patterns_and_surface_are_applied ... ok
test stamp::test_validation_rejects_bad_grid_merge_and_topology ... ok
test stamp::test_remove_restores_a_plain_circuit ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s


running 4 tests
test test_each_category_has_its_pad_and_grid ... ok
test test_every_chute_is_an_entry_segment_merging_tangent_to_the_loop ... ok
test test_all_43_circuits_have_a_valid_launch_chute ... ok
test test_chute_circuits_round_trip ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.92s


running 8 tests
test test_launch_chute_tool_is_in_the_palette ... ok
test test_refused_clicks_leave_the_circuit_and_explain ... ok
test test_circuit_view_shows_the_chute_grid_instead_of_a_grid_stepper ... ok
test test_left_click_with_the_chute_tool_stamps_and_elsewhere_selects ... ok
test test_saved_circuit_keeps_its_launch_chute ... ok
test test_clicking_a_waypoint_stamps_a_launch_chute ... ok
test test_insert_button_places_a_chute_and_undo_takes_it_back ... ok
test test_inspector_controls_edit_the_chute ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.40s


running 26 tests
test a_driver_who_did_not_finish_is_classified_last_in_the_heat ... ok
test fia_autocross_top_20_odd_even_semis_and_top_5_advancement_to_10_car_final ... ok
test final_ranks_follow_grand_final_then_b_final_then_earlier_exits ... ok
test results_that_do_not_match_the_race_are_refused ... ok
test fia_autocross_best_2_of_3_heats_drops_worst_heat_and_dnf ... ok
test a_64_driver_weekend_runs_8_heats_4_quarterfinals_2_semifinals_and_the_finals ... ok
test fia_autocross_30_driver_timed_practice_and_snake_seeding_into_3_heats_of_10 ... ok
test fia_autocross_continuous_player_racing_in_consolation_semi_and_b_final ... ok
test cancelling_the_latest_round_drops_the_weekend ... ok
test all_32_drivers_are_scored_by_final_rank_in_one_update ... ok
test a_player_who_misses_the_cut_keeps_racing_in_every_stage ... ok
test the_top_4_of_each_semifinal_go_to_the_grand_final_and_the_rest_to_the_b_final ... ok
test world_rx_overall_total_time_intermediate_classification_and_odd_even_semifinals ... ok
test the_top_4_of_each_heat_are_seeded_into_two_semifinals_by_elapsed_time ... ok
test world_rx_configurable_multi_round_qualifying_heats_q1_to_q3 ... ok
test the_32_driver_field_is_drawn_into_4_heats_of_8 ... ok
test world_rx_position_only_semifinals_and_row_by_row_grand_final_pairing ... ok
test the_player_is_in_exactly_one_scheduled_race_at_every_step ... ok
test world_rx_24_driver_superpole_and_snake_seeding_into_4_heats_of_6 ... ok
test world_rx_continuous_player_racing_across_24_driver_weekend ... ok
test a_weekend_replays_bit_for_bit_from_the_same_seed ... ok
test fia_autocross_two_phase_scoring_and_35_point_max_score ... ok
test world_rx_three_stage_cumulative_championship_points ... ok
test a_weekend_in_progress_survives_serialization ... ok
test weekend_keys_are_declared_only_by_tournament_series ... ok
test fia_autocross_all_5_series_tomls_load_with_30_drivers_and_progressive_calendars ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 8 tests
test the_bracket_shows_32_drivers_and_the_players_slot ... ok
test world_rx_bracket_and_hud_badges_for_superpole_q_rounds_and_6_car_stages ... ok
test fia_autocross_bracket_and_hud_badges_for_timed_practice_and_10_car_stages ... ok
test the_bracket_after_the_heats_shows_positions_and_who_advanced ... ok
test a_player_out_of_the_heat_has_a_consolation_slot ... ok
test the_hud_badge_names_the_stage_and_the_laps_follow_it ... ok
test the_bracket_scales_to_64_drivers_and_shows_declared_cars ... ok
test test_spec126_all_32_tournament_drivers_display_valid_car_models_and_teams ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/launch_chute_tests.rs (target/debug/deps/launch_chute_tests-8d251371d0ec5a87)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/ax_rx_circuit_launch_chute_validation.rs (target/debug/deps/ax_rx_circuit_launch_chute_validation-f522eaadfe638fbf)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running tests/track_editor_launch_chute_tests.rs (target/debug/deps/track_editor_launch_chute_tests-7f76bce1e87fd030)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running tests/autocross_tournament_tests.rs (target/debug/deps/autocross_tournament_tests-b62e2e8491d2ac54)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
     Running tests/tournament_bracket_ui_tests.rs (target/debug/deps/tournament_bracket_ui_tests-64f2ae6366bfd73c)

```
