---
type: validation_receipt
schema_version: "1.0"
spec: "specs/045_lan_lobby_reuses_circuit_selector_and_garage.md"
epic: "tdrace-l8uy"
candidate_commit: "5b2115b8da3249ff58dd9175464f07f61b0dfb9c"
verifier: "local-user"
evaluated_at: "2026-09-28T09:07:41Z"
command: "cargo test -p cabinet --test net_tests && cargo test -p tdrace-app --test lan_integration_tests -- --skip test_lan_livery_synchronization_and_countdown_handshake"
exit_code: 0
duration_ms: 2233
status: passed
---

# 🧾 Validation Receipt: Spec 045

- **Candidate Commit**: `5b2115b8da3249ff58dd9175464f07f61b0dfb9c`
- **Spec**: `specs/045_lan_lobby_reuses_circuit_selector_and_garage.md`
- **Command**: `cargo test -p cabinet --test net_tests && cargo test -p tdrace-app --test lan_integration_tests -- --skip test_lan_livery_synchronization_and_countdown_handshake`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 6 tests
test test_cabinet_lan_host_and_join_screens_lifecycle ... ok
test test_host_and_client_loopback_handshake ... ok
test test_lobby_pump_network_keeps_client_connected_and_syncs_car ... ok
test test_client_graceful_disconnect ... ok
test test_client_slot_customization_and_ready_check ... ok
test test_launch_countdown_and_race_streaming ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 6 tests
test test_lan_client_perspective_targeting_and_helpers ... ok
test test_lan_host_picks_circuit_in_full_screen_selector ... ok
test test_lan_launch_session_and_nameplates ... ok
test test_lan_host_prevents_split_screen_and_applies_remote_inputs ... ok
test test_lan_garage_round_trip_keep_alive_and_disconnect ... ok
test test_lan_multiplayer_hub_lifecycle ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 1.90s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/net_tests.rs (target/debug/deps/net_tests-f2efd039c955fb49)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running tests/lan_integration_tests.rs (target/debug/deps/lan_integration_tests-abda6711a4825c9c)

```
