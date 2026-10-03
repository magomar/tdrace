---
type: validation_receipt
schema_version: "1.0"
spec: "specs/081_rallycross_joker_lap_segments_for_classic_and_openstreetmap_circuits.md"
epic: "tdrace-vbaq"
candidate_commit: "ec0956629c4a858e60a49474ea70233539aa6eb8"
verifier: "local-user"
evaluated_at: "2026-10-03T15:47:00Z"
command: "cargo test -p tdrace-core --test classic_circuits_tests"
exit_code: 0
duration_ms: 818
status: passed
---

# 🧾 Validation Receipt: Spec 081

- **Candidate Commit**: `ec0956629c4a858e60a49474ea70233539aa6eb8`
- **Spec**: `specs/081_rallycross_joker_lap_segments_for_classic_and_openstreetmap_circuits.md`
- **Command**: `cargo test -p tdrace-core --test classic_circuits_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 20 tests
test test_stock_thunder_bowl_design_rules ... ok
test test_clay_bowl_has_launch_pad_and_crest ... ok
test test_all_terrain_mudbath_valley_design_rules ... ok
test test_velocity_park_has_two_long_straights_and_chicanes ... ok
test test_stock_tri_oval_speedway_design_rules ... ok
test test_all_terrain_dune_sea_design_rules ... ok
test test_hillside_hammer_has_off_camber_and_summit_climb ... ok
test test_all_terrain_frostbite_pass_design_rules ... ok
test test_ridge_ring_has_ridge_climb_and_deepsand_traps ... ok
test test_stock_roval_design_rules ... ok
test test_kart_circuits_are_outdoor_standard_circuits ... ok
test test_coastal_grand_prix_has_400m_straight_carousel_and_plateau ... ok
test test_kart_circuits_have_aligned_walls_and_a_flat_grid ... ok
test test_autocross_circuits_are_unpaved_without_ramps ... ok
test test_gt_circuits_have_speed_braking_and_runoff ... ok
test test_canyon_flyer_has_water_gap_and_whoops ... ok
test test_hilltop_leap_has_crest_and_gravel_hairpin ... ok
test test_vault_quarantined_indoor_kart_circuits_have_bridges ... ok
test test_rallycross_circuits_have_jumps_and_mixed_surfaces ... ok
test test_classic_rallycross_circuits_have_joker_track_networks ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 11.0 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/classic_circuits_tests.rs (target/debug/deps/classic_circuits_tests-37602c5f8d4fb7c3)

```
