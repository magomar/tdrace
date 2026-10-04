---
type: validation_receipt
schema_version: "1.0"
spec: "specs/084_highthroughput_physics_zeroallocation_lidar_and_precomputed_junction_render_caching.md"
epic: "tdrace-vgu7"
candidate_commit: "6bdeae73093c9c1ea4516abc91bd86478f3ec753"
verifier: "local-user"
evaluated_at: "2026-10-04T07:06:45Z"
command: "cargo bench -p wheelbase --bench physics_bench && cargo bench -p arcade-race-core --bench lidar_bench && cargo bench -p arcade-race-core --bench collision_bench && cargo test -p race-kit --test bot_vehicle_tests"
exit_code: 0
duration_ms: 5624
status: passed
---

# 🧾 Validation Receipt: Spec 084

- **Candidate Commit**: `6bdeae73093c9c1ea4516abc91bd86478f3ec753`
- **Spec**: `specs/084_highthroughput_physics_zeroallocation_lidar_and_precomputed_junction_render_caching.md`
- **Command**: `cargo bench -p wheelbase --bench physics_bench && cargo bench -p arcade-race-core --bench lidar_bench && cargo bench -p arcade-race-core --bench collision_bench && cargo test -p race-kit --test bot_vehicle_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
============================================================
🏁 TDRace Physics Throughput Benchmark
============================================================
Warming up with 100000 steps...
Running 2000000 physics steps...
------------------------------------------------------------
Elapsed Time:        1.0780 s
Throughput:          1855269.08 steps/second
Latency per step:    539.01 ns/step
Regression floor:    1500000 steps/second
Roadmap target:      4000000 steps/second (not met)
Status:              ✅ PASS (floor)
============================================================
============================================================
📡 TDRace High-Speed LIDAR Raycasting Benchmark
============================================================
Warming up LIDAR scanner...
Running 100000 LIDAR sweeps (3200000 total rays)...
------------------------------------------------------------
Elapsed Time:        0.2121 s
LIDAR Sweeps/sec:    471373.46 sweeps/second
Throughput:          15083950.62 rays/second
Latency per ray:     66.30 ns/ray
Regression floor:    14000000 rays/second
Status:              ✅ PASS (floor)
============================================================
============================================================
💥 TDRace Collision & Multi-Body Resolution Benchmark
============================================================
Benchmarking SAT OBB vs OBB (5000000 iterations)...
SAT Overlap Checks:  22649779.49 checks/second (44.15 ns/check)
Benchmarking Multi-Car Solver (8 cars, 100000 steps)...
8-Car Multi-Body Steps: 120353.38 steps/second (8.31 us/step)
Benchmarking Track Wall Collisions (500000 steps)...
Track Barrier Checks:   1445728.85 checks/second (691.69 ns/check)
------------------------------------------------------------
SAT regression floor: 20000000 checks/second
SAT roadmap target:   22000000 checks/second (met)
Status:     ✅ PASS (floor)
============================================================

running 5 tests
test test_bot_parallel_straight_immunity_and_throat_lateral_repulsion ... ok
test test_bot_ai_pit_tactics_and_stall_stopping ... ok
test test_bot_ai_lap1_pit_gating_stall_assignment_and_cooldown ... ok
test test_bot_tactical_pit_stop_entry_service_and_rejoin_pipeline ... ok
test bot_drives_a_non_car_vehicle ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s


stderr:
    Finished `bench` profile [optimized] target(s) in 0.02s
     Running benches/physics_bench.rs (target/release/deps/physics_bench-14d08b853bf989d5)
    Finished `bench` profile [optimized] target(s) in 0.02s
     Running benches/lidar_bench.rs (target/release/deps/lidar_bench-af4ab3fdaaa6b6ed)
    Finished `bench` profile [optimized] target(s) in 0.02s
     Running benches/collision_bench.rs (target/release/deps/collision_bench-8a6c6f4ba8dee072)
   Compiling wheelbase v0.1.0 (/home/mario/workspace/games/tdrace/crates/wheelbase)
   Compiling arcade-race-core v0.1.0 (/home/mario/workspace/games/tdrace/crates/arcade-race-core)
   Compiling race-kit v0.1.0 (/home/mario/workspace/games/tdrace/crates/race-kit)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.03s
     Running tests/bot_vehicle_tests.rs (target/debug/deps/bot_vehicle_tests-8c477fe464b35a37)

```
