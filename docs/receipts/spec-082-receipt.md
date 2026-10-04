---
type: validation_receipt
schema_version: "1.0"
spec: "specs/082_mandatory_rallycross_joker_lap_tracking_and_penalty_enforcement.md"
epic: "tdrace-eo64"
candidate_commit: "604a898f3fb81932dd01e2b3d0f2960f73a36e71"
verifier: "local-user"
evaluated_at: "2026-10-04T17:57:32Z"
command: "cargo test --release -p race-kit --test joker_rule_tests && cargo test --release -p arcade-race-core --test multi_route_progress_tests && cargo test --release -p tdrace-app --test rally_tracks_tests --test ai_tests --test joker_ui_tests --test ui_table_migration_tests && cargo test --release -p cabinet --test net_tests"
exit_code: 0
duration_ms: 124654
status: passed
---

# 🧾 Validation Receipt: Spec 082

- **Candidate Commit**: `604a898f3fb81932dd01e2b3d0f2960f73a36e71`
- **Spec**: `specs/082_mandatory_rallycross_joker_lap_tracking_and_penalty_enforcement.md`
- **Command**: `cargo test --release -p race-kit --test joker_rule_tests && cargo test --release -p arcade-race-core --test multi_route_progress_tests && cargo test --release -p tdrace-app --test rally_tracks_tests --test ai_tests --test joker_ui_tests --test ui_table_migration_tests && cargo test --release -p cabinet --test net_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 3 tests
test default_rules_never_penalise ... ok
test driver_who_took_the_joker_gets_no_penalty ... ok
test missed_joker_adds_penalty_and_drops_the_driver_behind_a_slower_finisher ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s


running 5 tests
test test_track_progress_tracker_update_network_sync ... ok
test test_wrong_way_detection_on_divergent_branch ... ok
test test_multi_car_free_choice_routes_and_joker_counting ... ok
test test_joker_counts_when_the_car_runs_wide_past_the_gate_end ... ok
test test_no_joker_for_a_car_in_the_infield_next_to_the_joker ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 11 tests
test test_bot_profiles_creation ... ok
test test_bot_ai_strategic_joker_lap_decision ... ok
test test_bot_ai_dynamic_traffic_avoidance ... ok
test test_bot_ai_multi_route_split_junction_navigation ... ok
test test_bot_ai_slipstream_drafting_and_slingshot_pack_racing ... ok
test test_bot_ai_collision_avoidance ... ok
test test_multi_bot_branching_race_simulation ... ok
test test_bot_ai_steering_and_throttle_on_straight ... ok
test test_bot_ai_cornering_slowdown ... ok
test test_bot_ai_on_kart_grid_positions ... ok
test test_bot_ai_strategic_joker_rx_race_compliance ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.70s


running 2 tests
test joker_pill_is_amber_until_taken_red_on_the_last_lap_and_off_without_the_rule ... ok
test results_show_the_joker_column_only_in_joker_races ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 19 tests
test test_dirt_figure_eight_horizontal_flat_dirt_arena ... ok
test test_dirt_figure_eight_jump_ramps_proportional_trajectory ... ok
test test_holjes_rx_joker_lap_time_delta_simulation ... ok
test test_blyton_ids_resolve_to_croft ... ok
test test_silverstone_and_yas_marina_ids_resolve_to_their_replacements ... ok
test test_world_rx_jump_ramps_dirt_surface_and_containment_landing ... ok
test test_export_and_save_rally_tracks_to_disk ... ok
test test_world_rx_tracks_jump_ramps_and_mixed_surfaces ... ok
test test_all_20_world_rx_circuits_have_valid_joker_track_networks ... ok
test test_rx_joker_branch_never_turns_tighter_than_3_m ... ok
test test_rx_races_get_the_joker_rule_and_other_races_do_not ... ok
test test_rally_tracks_centerline_driving_and_no_wall_obstructions ... ok
test test_rx_race_ignores_the_joker_layout_pick ... ok
test test_rx_layout_checkpoints_lie_on_their_route_in_driving_order ... ok
test test_rx_joker_road_reads_as_its_own_surface_not_runoff ... ok
test test_rally_module_tracks_integrity_and_validation ... ok
test test_famous_rally_tracks_in_track_manager_and_menu_resolution ... ok
test test_rally_race_session_simulation_on_new_tracks ... ok
test test_rx_car_driving_the_joker_route_gets_its_lap_and_its_joker ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.14s


running 2 tests
test results_columns_preserve_order_and_racing_formatting ... ok
test pause_focus_rows_and_setting_actions_preserve_paused_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s


running 11 tests
test test_d1_eight_car_world_state_fits_one_datagram ... ok
test test_v1_client_gets_a_v1_version_mismatch_reply ... ok
test test_lan_standings_order_by_finish_time_plus_penalty ... ok
test test_d5_lost_state_sync_does_not_change_the_client_roster ... ok
test test_d4_client_in_slot_three_keeps_its_slot_after_launch ... ok
test test_cabinet_lan_host_and_join_screens_lifecycle ... ok
test test_client_graceful_disconnect ... ok
test test_host_and_client_loopback_handshake ... ok
test test_client_slot_customization_and_ready_check ... ok
test test_lobby_pump_network_keeps_client_connected_and_syncs_car ... ok
test test_launch_start_and_car_state_relay ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s


stderr:
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling zmij v1.0.23
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling memchr v2.8.3
   Compiling itoa v1.0.18
   Compiling syn v3.0.6
   Compiling serde_derive v1.0.229
   Compiling glam v0.29.3
   Compiling wheelbase v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/wheelbase)
   Compiling arcade-race-core v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/arcade-race-core)
   Compiling race-kit v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/race-kit)
    Finished `release` profile [optimized] target(s) in 20.44s
     Running tests/joker_rule_tests.rs (target/release/deps/joker_rule_tests-e014a74087cf9968)
   Compiling arcade-race-core v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/arcade-race-core)
    Finished `release` profile [optimized] target(s) in 4.19s
     Running tests/multi_route_progress_tests.rs (target/release/deps/multi_route_progress_tests-15ff5458226b994f)
   Compiling bitflags v2.13.2
   Compiling autocfg v1.5.1
   Compiling libc v0.2.190
   Compiling objc2 v0.6.4
   Compiling objc2-encode v4.1.0
   Compiling log v0.4.34
   Compiling bytemuck v1.25.2
   Compiling smallvec v1.16.2
   Compiling lazy_static v1.5.1
   Compiling cfg-if v1.0.5
   Compiling regex-lite v0.1.9
   Compiling simd-adler32 v0.3.10
   Compiling adler2 v2.0.1
   Compiling crc32fast v1.5.2
   Compiling equivalent v1.0.2
   Compiling libm v0.2.16
   Compiling serde_core v1.0.229
   Compiling zerocopy v0.8.59
   Compiling crossbeam-utils v0.8.23
   Compiling version_check v0.9.5
   Compiling miniz_oxide v0.9.1
   Compiling miniz_oxide v0.8.9
   Compiling fdeflate v0.3.7
   Compiling zmij v1.0.23
   Compiling allocator-api2 v0.2.21
   Compiling num-traits v0.2.19
   Compiling ahash v0.8.12
   Compiling serde v1.0.229
   Compiling foldhash v0.1.5
   Compiling find-msvc-tools v0.1.14
   Compiling cpal v0.18.2
   Compiling shlex v2.0.1
   Compiling extended v0.1.0
   Compiling bitflags v1.3.2
   Compiling miniquad v0.4.11
   Compiling cc v1.6.0
   Compiling hashbrown v0.15.5
   Compiling serde_json v1.0.151
   Compiling vcpkg v0.2.15
   Compiling flate2 v1.1.10
   Compiling core-foundation-sys v0.8.7
   Compiling pkg-config v0.3.34
   Compiling block2 v0.6.2
   Compiling dispatch2 v0.3.1
   Compiling objc2-foundation v0.3.2
   Compiling objc2-core-audio-types v0.3.2
   Compiling png v0.17.16
   Compiling itoa v1.0.18
   Compiling color_quant v1.1.0
   Compiling once_cell v1.21.4
   Compiling byteorder v1.5.0
   Compiling memchr v2.8.3
   Compiling gilrs v0.11.2
   Compiling vec_map v0.8.2
   Compiling objc2-core-foundation v0.3.2
   Compiling malloc_buf v0.0.6
   Compiling mach2 v0.6.0
   Compiling objc-rs v0.2.8
   Compiling mint v0.5.9
   Compiling dasp_sample v0.11.0
   Compiling uuid v1.27.0
   Compiling glam v0.33.12
   Compiling triple_buffer v9.0.0
   Compiling libsqlite3-sys v0.30.1
   Compiling core_maths v0.1.1
   Compiling ttf-parser v0.25.1
   Compiling iana-time-zone v0.1.65
   Compiling rtrb v0.4.0
   Compiling glam v0.27.0
   Compiling hashbrown v0.17.1
   Compiling pastey v0.2.3
   Compiling macroquad_macro v0.1.8
   Compiling fnv v1.0.7
   Compiling quad-rand v0.2.3
   Compiling atomic-arena v0.1.2
   Compiling serde_spanned v0.6.9
   Compiling toml_datetime v0.6.11
   Compiling indexmap v2.14.2
   Compiling winnow v0.7.15
   Compiling toml_write v0.1.2
   Compiling objc2-core-audio v0.3.2
   Compiling num-complex v0.4.6
   Compiling objc2-io-kit v0.3.2
   Compiling objc2-audio-toolbox v0.3.2
   Compiling fontdue v0.9.4
   Compiling image v0.24.9
   Compiling chrono v0.4.45
   Compiling symphonia-core v0.6.1
   Compiling gilrs-core v0.6.8
   Compiling coreaudio-rs v0.14.2
   Compiling fallible-streaming-iterator v0.1.9
   Compiling fallible-iterator v0.3.0
   Compiling toml_edit v0.22.27
   Compiling macroquad v0.4.16
   Compiling hashbrown v0.14.5
   Compiling tdrace-core v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/tdrace-core)
   Compiling symphonia-metadata v0.6.1
   Compiling symphonia-bundle-mp3 v0.6.1
   Compiling symphonia-codec-pcm v0.6.1
   Compiling hashlink v0.9.1
   Compiling symphonia-common v0.6.1
   Compiling symphonia-format-riff v0.6.1
   Compiling symphonia-bundle-flac v0.6.1
   Compiling symphonia-format-ogg v0.6.1
   Compiling symphonia-codec-vorbis v0.6.1
   Compiling toml v0.8.23
   Compiling symphonia v0.6.1
   Compiling kira v0.12.5
   Compiling cabinet v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/cabinet)
   Compiling race-ui v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/race-ui)
warning: tdrace-core@0.1.0: embedded official circuits are 12.0 MB (target <= 8 MB)
   Compiling rusqlite v0.32.1
   Compiling tdrace-app v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/tdrace-app)
    Finished `release` profile [optimized] target(s) in 1m 23s
     Running tests/ai_tests.rs (target/release/deps/ai_tests-1a0b1c267b4de976)
     Running tests/joker_ui_tests.rs (target/release/deps/joker_ui_tests-07289db226aa5ab6)
     Running tests/rally_tracks_tests.rs (target/release/deps/rally_tracks_tests-527286f9adb09ef4)
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
     Running tests/ui_table_migration_tests.rs (target/release/deps/ui_table_migration_tests-0746a7a6ab564109)
   Compiling cabinet v0.1.0 (/Users/mario.gomez/workspace/games/workspace-specs/rallycross-joker-enforcement/crates/cabinet)
    Finished `release` profile [optimized] target(s) in 6.76s
     Running tests/net_tests.rs (target/release/deps/net_tests-a51af848c0eaed0a)

```
