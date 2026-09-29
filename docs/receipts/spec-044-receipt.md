---
type: validation_receipt
schema_version: "1.0"
spec: "specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md"
epic: "tdrace-6d87"
candidate_commit: "ff96e0ac469a1e4b5c8738be88203f4e54406e6d"
verifier: "local-user"
evaluated_at: "2026-09-29T14:36:06Z"
command: "cargo test -p tdrace-app --test lan_integration_tests --test lan_sync_tests"
exit_code: 0
duration_ms: 34839
status: passed
---

# 🧾 Validation Receipt: Spec 044

- **Candidate Commit**: `ff96e0ac469a1e4b5c8738be88203f4e54406e6d`
- **Spec**: `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md`
- **Command**: `cargo test -p tdrace-app --test lan_integration_tests --test lan_sync_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 8 tests
test test_lan_client_perspective_targeting_and_helpers ... ok
test test_lan_multiplayer_hub_lifecycle ... ok
test test_lan_host_picks_circuit_in_full_screen_selector ... ok
test test_lan_launch_session_and_nameplates ... ok
test test_lan_livery_synchronization_and_countdown_handshake ... ok
test test_lan_host_prevents_split_screen_and_does_not_simulate_remote_cars ... ok
test test_lan_garage_round_trip_keep_alive_and_disconnect ... ok
test test_d5_slot_gap_maps_every_player_to_its_own_car ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.38s


running 7 tests
test test_solid_mode_pushes_only_the_own_car ... ok
test test_ghost_mode_cars_pass_through_each_other ... ok
test test_host_leaving_mid_race_sends_clients_to_results_then_hub ... ok
test test_host_pause_keeps_everyone_connected_and_racing ... ok
test test_three_player_race_finishes_with_the_same_results_everywhere ... ok
test test_client_leaving_mid_race_is_parked_and_dnf_everywhere ... ok
test test_four_players_with_slot_gap_stay_in_sync_over_a_lossy_link ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.35s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running tests/lan_integration_tests.rs (target/debug/deps/lan_integration_tests-44239084441f2284)
     Running tests/lan_sync_tests.rs (target/debug/deps/lan_sync_tests-350d3831c7c5c817)

```
